use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use fehler::throws;
use futures::select;
use futures::stream::StreamExt;
use tokio::net::{UdpFramed, UdpSocket};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::TestCase;
use crate::msg::{Message, MessageCodec};
use crate::paxos::{Paxos, PaxosConfig};

pub type ProtocolSocket = UdpFramed<MessageCodec>;

pub const PORT_NUMBER: u16 = 42069;

#[throws(io::Error)]
async fn make_proc_socket(port: u16) -> ProtocolSocket {
    UdpFramed::new(UdpSocket::bind(format!("0.0.0.0:{}", port)).await?, MessageCodec)
}

#[throws(io::Error)]
pub async fn incoming_socket() -> ProtocolSocket {
    make_proc_socket(PORT_NUMBER).await?
}

#[throws(io::Error)]
pub async fn outgoing_socket() -> ProtocolSocket {
    make_proc_socket(PORT_NUMBER + 1).await?
}

pub struct Node {
    addr: SocketAddr,
}

impl Node {
    /// Attempt to resolve the given hostname repeatedly until success.
    #[throws(io::Error)]
    fn resolve_from_hostname<S: AsRef<str>>(hostname: S) -> Node {
        while let Err(e) = format!("{}:{}", hostname.as_ref(), PORT_NUMBER).to_socket_addrs() {
            eprintln!("{:?}", e);
            // really ought to use exponential backoff here and eventually throw the error
            thread::sleep(Duration::from_millis(500));
        }

        let addr =
            format!("{}:{}", hostname.as_ref(), PORT_NUMBER).to_socket_addrs()?.next().unwrap();
        Node { addr }
    }
}

#[derive(Clone)]
pub struct Nodes(UnboundedSender<(Message, SocketAddr)>, Arc<Vec<Node>>);

impl Nodes {
    pub fn len(&self) -> usize {
        self.1.len()
    }

    #[throws(io::Error)]
    pub fn multicast_send(&mut self, msg: Message) -> () {
        for node in self.1.iter() {
            self.0.try_send((msg, node.addr)).unwrap();
        }
    }
}

pub struct System {
    pid: usize,
    incoming: ProtocolSocket,
    opt_rx: Option<UnboundedReceiver<(Message, SocketAddr)>>,
    nodes: Nodes,
}

impl System {
    #[throws(io::Error)]
    pub async fn from_hosts(hosts: Vec<String>, hostname: &str) -> System {
        let pid = hosts.iter().take_while(|curr_host| curr_host != &hostname).count();
        let nodes: io::Result<Vec<_>> = hosts.iter().map(Node::resolve_from_hostname).collect();
        let incoming = incoming_socket().await?;
        let (tx, rx) = mpsc::unbounded_channel();
        System {
            pid, incoming,
            opt_rx: Some(rx),
            nodes: Nodes(tx, Arc::new(nodes?))
        }
    }

    /// gets the outgoing receiver from this system, fails on subsequent attempts
    fn take_outgoing(&mut self) -> UnboundedReceiver<(Message, SocketAddr)> {
        self.opt_rx.take().unwrap()
    }

    #[throws]
    #[allow(unreachable_code)]
    pub async fn paxos(
        mut self, test_case: TestCase, progress_timer_length: u64, vc_proof_timer_length: u64
    ) -> ! {
        // create an outgoing socket to actually forward sent messages along
        let outgoing_socket = outgoing_socket().await?;
        let mut outgoing_future = self.take_outgoing().map(|m| Ok(m)).forward(outgoing_socket);

        // create a new instance of the Paxos protocol
        let paxos = Paxos::new(PaxosConfig {
            pid: self.pid,
            nodes: self.nodes.clone(),
            test_case, progress_timer_length, vc_proof_timer_length
        })?;

        // split paxos into a separate sink and stream
        let (paxos_inc, paxos_out) = paxos.split();

        // forward received messages to the protocol implementation
        let mut incoming_future = self.incoming
            .map(|result| result.map(|msg_with_addr| msg_with_addr.0))
            .forward(paxos_inc);

        let mut paxos_out = paxos_out.fuse();

        loop {
            select! {
                res = outgoing_future => res?,
                res = incoming_future => res?,
                opt_res = paxos_out.next() => match opt_res {
                    Some(res) => res?,
                    None => (),
                },
            }
        }
    }
}
