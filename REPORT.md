# paxos-vc report

## System Architecture

paxos-vc is an implementation of the view change algorithm from Paxos. The system consists of a number of nodes, given by their respective hostnames in the input hostfile. Each node runs with two sockets, one for incoming messages nad one for outgoing messages. Each node has two timers: one for keeping track of progress and inducing view changes, and the other for indicating when to send VC proof messages of their currently installed view. When the nodes send these respective messages, they always multicast them to all the nodes, including themselves.

## Design Decisions

I once again continued to use type-based abstractions to define explicit encoder and decoders for message types, and wrap the socket interface with a typed notion of messages. Further, I built the implementation of Paxos using the Stream and Sink abstractions from Rust's futures library. The Stream ensures that the Paxos implementation continually polls the timers, and defines the appropriate behavior for when the timers fire (i.e. either starting a view change, or sending a VC proof message). The Sink defines how the system should respond to incoming messages according to the protocol. This design decouples the actual networking components from the underlying implementation of the protocol.

## Implementation Issues

This time around, there weren't really a lot of implementation issues. I started immediately in Rust, rather than using C. So, dynamic memory management didn't give me a hard time. I spent some of the time learning a bit more about the new asynchronous networking support in Rust, and was able to learn about some useful features that made it easier to structure my code nicely (namely, I found these new macros for polling a number of futures simultaneously which let you describe what to do after getting each result).
