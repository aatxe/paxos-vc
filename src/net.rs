use std::convert::TryFrom;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tokio::net::{UdpFramed, UdpSocket};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::msg::{Message, MessageCodec};

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
    #[throws(io::Error)]
    pub fn send(&mut self, pid: u32, msg: Message) -> () {
        let node = self.1.iter().nth(usize::try_from(pid).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidInput, e)
        })?).ok_or_else(|| -> io::Error { io::ErrorKind::InvalidInput.into() })?;
        self.0.try_send((msg, node.addr)).unwrap();
    }

    #[throws(io::Error)]
    pub fn multicast_send(&mut self, msg: Message) -> () {
        for node in self.1.iter() {
            self.0.try_send((msg, node.addr)).unwrap();
        }
    }
}

pub struct System(ProtocolSocket, Option<UnboundedReceiver<(Message, SocketAddr)>>, Nodes);

impl System {
    #[throws(io::Error)]
    pub async fn from_hosts(hosts: Vec<String>) -> System {
        let nodes: io::Result<Vec<_>> = hosts.iter().map(Node::resolve_from_hostname).collect();
        let incoming = incoming_socket().await?;
        let (tx, rx) = mpsc::unbounded_channel();
        System(incoming, Some(rx), Nodes(tx, Arc::new(nodes?)))
    }

    pub fn take_outgoing(&mut self) -> UnboundedReceiver<(Message, SocketAddr)> {
        self.1.take().unwrap()
    }
}
