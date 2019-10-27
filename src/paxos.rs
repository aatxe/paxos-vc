use std::convert::TryFrom;
use std::collections::HashSet;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use futures::{Poll, Sink};
use futures::task::Context;

use crate::TestCase;
use crate::msg::Message;
use crate::net::Nodes;

/// An internal entry for tracking received view changes.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct VC(u32, u32);

pub struct Paxos {
    /// pid of the current node
    pid: u32,
    /// all the nodes in the system
    nodes: Nodes,
    /// the current test case being executed
    test_case: TestCase,
    /// the last view we attempted to install
    last_attempted_view: u32,
    /// the current view that we have installed
    current_view: Arc<AtomicU32>,
    /// a set of all the current view change messages received.
    view_change_state: HashSet<VC>,
}

impl Paxos {
    #[throws]
    pub fn new(pid: usize, nodes: Nodes, test_case: TestCase) -> Paxos {
        Paxos {
            pid: u32::try_from(pid)?,
            nodes, test_case,
            last_attempted_view: 0,
            current_view: Arc::new(AtomicU32::new(0)),
            view_change_state: HashSet::new(),
        }
    }

    /// gets a reference to the view for this instance of Paxos
    /// note: if you simply need the value right now, use `Paxos::current_view(...)` instead.
    pub fn view(&self) -> Arc<AtomicU32> {
        self.current_view.clone()
    }

    /// gets the current view for this instance of Paxos
    /// note: if you need to keep track of the view as it changes, use `Paxos::view(...)` instead.
    pub fn current_view(&self) -> u32 {
        self.current_view.load(Ordering::SeqCst)
    }

    /// computes the id of the current leader according to the installed view
    pub fn current_leader(&self) -> u32 {
        match u32::try_from(self.nodes.len()) {
            Ok(num_nodes) => self.current_view() % num_nodes,
            // if the length (usize) can't be converted into a u32, then there are more nodes than
            // could possibly fit into the current view counter. Thus, just the view will suffice.
            Err(_) => self.current_view(),
        }
    }

    /// starts a view change to the given view by sending out view change messages
    /// invariant: a node should only ever try to install larger views than what it has installed
    #[throws(io::Error)]
    fn start_view_change(&mut self, new_view: u32) {
        assert!(new_view > self.current_view());

        // clear the current view change state
        self.view_change_state.clear();

        // set the last attempted view to this new view
        self.last_attempted_view = new_view;

        // send view change to all the servers
        self.nodes.multicast_send(Message::ViewChange {
            server_id: self.pid,
            attempted: new_view,
        })?;
    }

    /// installs the last attempted view if we have seen a majority attempting to install it
    fn install_view_if_possible(&self) {
        let vc_received = self.view_change_state.iter()
            .filter(|vc| vc.1 == self.last_attempted_view)
            .count();
        // if we have a majority attempting to install the last_attempted_view, then
        if vc_received >= (self.nodes.len() / 2) + 1 {
            // we can go ahead and install the view (since we have no reconciliation phase)
            self.install_view()
        }
    }

    /// installs the last attempted view unconditionally
    /// invariant: a view can only be installed with a proof in the form of either view changes from
    /// a majority of nodes or a vc proof message from another node
    fn install_view(&self) {
        let last_installed = self.current_view.swap(self.last_attempted_view, Ordering::SeqCst);
        // we should never install a view that is smaller than the one we already had
        assert!(self.last_attempted_view >= last_installed);
    }
}

impl Sink<Message> for Paxos {
    type Error = io::Error;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[throws(io::Error)]
    fn start_send(mut self: Pin<&mut Self>, msg: Message) -> () {
        use Message::*;

        match msg {
            ViewChange { server_id, attempted } => {
                // this view change message is stale
                if attempted < self.last_attempted_view { return }

                // there's an ongoing view change to a higher view
                if attempted > self.last_attempted_view {
                    return self.start_view_change(attempted)?
                }

                // this message is for the view we want to install
                self.view_change_state.insert(VC(server_id, attempted));
                self.install_view_if_possible();
            }

            VCProof { installed, .. } => {
                if installed == self.last_attempted_view {
                    // someone installed this view before us, so we can too!
                    self.install_view();
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
