use std::io;

use bytes::{Buf, BufMut, BytesMut, IntoBuf};
use fehler::{throw, throws};
use tokio::codec::{Decoder, Encoder};

#[derive(Clone, Copy, Debug)]
pub enum Message {
    /// A message indicating that the given node is attempting to change to the given view.
    ViewChange {
        /// the id of the node attempting to change views
        server_id: u32,
        /// the id of the view the node is attempting to adopt
        attempted: u32,
    },

    /// A proof that the given view is installed by the specified node.
    VCProof {
        /// the id of the node sending the proof
        server_id: u32,
        /// the view installed by the node
        installed: u32,
    },
}

pub struct MessageCodec;

impl Decoder for MessageCodec {
    type Item = Message;
    type Error = io::Error;

    #[throws(io::Error)]
    fn decode(&mut self, src: &mut BytesMut) -> Option<Message> {
        let mut buf = src.clone().into_buf();
        if buf.remaining() < 4 { return None }
        match buf.get_u32_be() {
            // ViewChange
            2 => {
                if buf.remaining() < 8 { return None }
                Some(Message::ViewChange {
                    server_id: buf.get_u32_be(),
                    attempted: buf.get_u32_be(),
                })
            },
            // VCProof
            3 => {
                if buf.remaining() < 8 { return None }
                Some(Message::VCProof {
                    server_id: buf.get_u32_be(),
                    installed: buf.get_u32_be(),
                })
            },
            // default case: unknown message type
            n => {
                eprintln!("unknown message type: {}", n);
                throw!(io::ErrorKind::InvalidData)
            },
        }
    }
}

impl Encoder for MessageCodec {
    type Item = Message;
    type Error = io::Error;

    #[throws(io::Error)]
    fn encode(&mut self, msg: Message, dst: &mut BytesMut) -> () {
        match msg {
            Message::ViewChange { server_id, attempted } => {
                dst.put_u32_be(2);
                dst.put_u32_be(server_id);
                dst.put_u32_be(attempted);
            },
            Message::VCProof { server_id, installed } => {
                dst.put_u32_be(3);
                dst.put_u32_be(server_id);
                dst.put_u32_be(installed);
            },
        }
    }
}
