use std::convert::TryFrom;
use std::collections::HashSet;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::process;
use std::time::{Duration, Instant};

use fehler::throws;
use futures::{Poll, Sink, Stream};
use futures::task::Context;
use log::{trace, info, warn};
use tokio::timer::{self, Delay, Interval};

use crate::TestCase;
use crate::msg::Message;
use crate::net::Nodes;

/// An internal entry for tracking received view changes.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct VC(u32, u32);

/// A configuration for constructing a new instance of Paxos.
pub struct PaxosConfig {
    /// the process id of the current node
    pub pid: usize,
    /// all the nodes in the system
    pub nodes: Nodes,
    /// the current test case being executed
    pub test_case: TestCase,
    /// the duration of the progress timer in seconds
    pub progress_timer_length: u64,
    /// the duration of the vc proof timer in seconds
    pub vc_proof_timer_length: u64,
}

/// An asynchronous implementation of Paxos.
pub struct Paxos {
    /// the process id of the current node
    pid: u32,
    /// all the nodes in the system
    nodes: Nodes,
    /// the current test case being executed
    test_case: TestCase,
    /// the length of the progress timer
    progress_length: Duration,
    /// a delay until the progress timer is finished
    progress_timer: Delay,
    /// an interval for sending vcproof messages every so often
    vc_proof_timer: Interval,
    /// the last view we attempted to install
    last_attempted_view: u32,
    /// the current view that we have installed
    current_view: u32,
    /// a set of all the current view change messages received.
    view_change_state: HashSet<VC>,
}

impl Paxos {
    /// Creates a new instance of Paxos.
    #[throws]
    pub fn new(config: PaxosConfig) -> Paxos {
        let PaxosConfig {
            pid, nodes, test_case, progress_timer_length, vc_proof_timer_length
        } = config;
        let progress_length = Duration::from_secs(progress_timer_length);
        let proof_length = Duration::from_secs(vc_proof_timer_length);
        Paxos {
            pid: u32::try_from(pid)?,
            nodes, test_case, progress_length,
            progress_timer: timer::delay_for(progress_length),
            vc_proof_timer: Interval::new_interval(proof_length),
            last_attempted_view: 0,
            current_view: 0,
            view_change_state: HashSet::new(),
        }
    }

    /// Computes the id of the current leader according to the installed view
    pub fn current_leader(&self) -> u32 {
        match u32::try_from(self.nodes.len()) {
            Ok(num_nodes) => self.current_view % num_nodes,
            // if the length (usize) can't be converted into a u32, then there are more nodes than
            // could possibly fit into the current view counter. Thus, just the view will suffice.
            Err(_) => self.current_view,
        }
    }

    /// Starts a view change to the given view by sending out view change messages
    /// invariant: a node should only ever try to install larger views than what it has installed
    #[throws(io::Error)]
    fn start_view_change(&mut self, new_view: u32) {
        info!("start view change to new view: {}", new_view);
        assert!(new_view > self.current_view);

        // clear the current view change state
        self.view_change_state.clear();

        // set the last attempted view to this new view
        self.last_attempted_view = new_view;

        // send view change to all the servers
        self.nodes.multicast_send(Message::ViewChange {
            server_id: self.pid,
            attempted: new_view,
        })?;

        // resets the progress timer
        self.reset_progress_timer();
    }

    /// Installs the last attempted view if we have seen a majority attempting to install it
    #[throws(io::Error)]
    fn install_view_if_possible(&mut self) {
        let vc_received = self.view_change_state.iter()
            .filter(|vc| vc.1 == self.last_attempted_view)
            .count();
        // if we have a majority attempting to install the last_attempted_view, then
        if vc_received >= (self.nodes.len() / 2) + 1 {
            // first, invoke test case hook to see if we should crash
            self.test_case_crash_hook();
            // then, we can go ahead and install the view (since we have no reconciliation phase)
            self.install_view()?;
        } else {
            info!("insufficient proof to install view {}: {}",
                  self.last_attempted_view, vc_received);
        }
    }

    /// Installs the last attempted view unconditionally
    /// invariant: a view can only be installed with a proof in the form of either view changes from
    /// a majority of nodes or a vc proof message from another node
    #[throws(io::Error)]
    fn install_view(&mut self) {
        // we should never install a view that is smaller than the one we already had
        assert!(self.last_attempted_view >= self.current_view);

        self.current_view = self.last_attempted_view;
        info!("installed view {}", self.current_view);
        self.output_leader();
        self.test_case_exit_hook();

        // send a VC proof immediately (not strictly necessary though)
        self.nodes.multicast_send(Message::VCProof {
            server_id: self.pid,
            installed: self.current_view,
        })?;
    }

    /// Resets the progress timer to its full length from now.
    fn reset_progress_timer(&mut self) {
        self.progress_timer.reset(Instant::now() + self.progress_length);
        info!("progress timer reset!");
    }

    /// Outputs the current leader and the new view.
    fn output_leader(&self) {
        println!("{}: Server {} is the new leader of view {}",
                 self.pid, self.current_leader(), self.current_view);
    }

    /// Either crash or do nothing, depending on the pid and test case.
    ///
    /// The behavior is defined as follows:
    /// ```
    /// /------------------------------\
    /// | pid | test case  | behavior  |
    /// |------------------------------|
    /// | 1   | 1, 2       | nop       |
    /// | 1   | 3, 4, 5    | crash     |
    /// |------------------------------|
    /// | 2   | 1, 2, 3    | nop       |
    /// | 2   | 4, 5       | crash     |
    /// |------------------------------|
    /// | 3   | 1, 2, 3, 4 | nop       |
    /// | 3   | 5          | crash     |
    /// |------------------------------|
    /// | 4   | *          | nop       |
    /// |------------------------------|
    /// | 5   | *          | nop       |
    /// \------------------------------/
    /// ```
    fn test_case_crash_hook(&self) {
        trace!("crash hook invoked");
        use TestCase::*;

        match self.test_case {
            SingleCrash if self.pid == 1 => panic!("crashing"),
            TwoCrashes if self.pid < 3 && self.pid > 0 => panic!("crashing"),
            ThreeCrashes if self.pid < 4 && self.pid > 0 => panic!("crashing"),
            _ => (),
        }
    }

    /// Either exits the program or does nothing, depending on the pid and test case.
    fn test_case_exit_hook(&self) -> () {
        trace!("exit hook invoked");
        use TestCase::*;

        match self.test_case {
            NormalCase if self.current_view == 1 => process::exit(0),
            FullRotation if self.current_view != 0 && self.current_leader() == 0 =>
                process::exit(0),
            SingleCrash if self.current_view == 2 => process::exit(0),
            TwoCrashes if self.current_view == 3 => process::exit(0),
            ThreeCrashes if self.current_view == 4 => process::exit(0),
            _ => (),
        }
    }
}

impl Sink<Message> for Paxos {
    type Error = io::Error;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[throws(io::Error)]
    fn start_send(mut self: Pin<&mut Self>, msg: Message) -> () {
        trace!("processing message: {:?}", msg);
        match msg {
            Message::ViewChange { server_id, attempted } => {
                // this view change message is stale
                if attempted < self.last_attempted_view {
                    warn!("stale view change message received: {}", attempted);
                    return
                }

                // there's an ongoing view change to a higher view
                if attempted > self.last_attempted_view {
                    return self.start_view_change(attempted)?
                }

                // this message is for the view we want to install
                self.view_change_state.insert(VC(server_id, attempted));
                self.install_view_if_possible()?;
            }

            Message::VCProof { server_id, installed } => {
                if installed == self.last_attempted_view && installed > self.current_view {
                    info!("installing view {} based on VC Proof from {}", installed, server_id);
                    // someone installed this view before us, so we can too!
                    self.install_view()?;
                }
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl Stream for Paxos {
    type Item = io::Result<()>;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // note: we have to ensure we poll both futures each time!
        let poll_progress_timer = Future::poll(Pin::new(&mut self.progress_timer), ctx);
        trace!("polled progress timer");
        let poll_vc_proof_timer = Stream::poll_next(Pin::new(&mut self.vc_proof_timer), ctx);
        trace!("polled vc proof timer");

        // if progress timer expired,
        if let Poll::Ready(()) = poll_progress_timer {
            trace!("progress timer expired");
            // then we'll start a view change to the next view
            let new_view = self.last_attempted_view + 1;
            return Poll::Ready(Some(self.start_view_change(new_view)))
        }

        // if vc proof timer fired,
        if let Poll::Ready(Some(_)) = poll_vc_proof_timer {
            trace!("vc proof timer fired");
           // then we'll multicast a vc proof to everyone 
            let server_id = self.pid;
            let installed = self.current_view;
            return Poll::Ready(Some(self.nodes.multicast_send(
                Message::VCProof { server_id, installed }
            )));
        }

        trace!("both timers pending");
        Poll::Pending
    }
}
