use crate::KAD_MESSAGE_LEN;
use node::peer_key::Key;
use node::peer_info::PeerInfo;
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use udp2p::routable::Routable;
use utils::utils::ByteRep;
use utils::impl_ByteRep;
use rand::{thread_rng, Rng};
use std::net::SocketAddr;
use std::collections::HashSet;
use std::marker::PhantomData;
use protocol::protocol::{Message, KadMessage, MessageKey};

impl_ByteRep!(for RPC, Req, Resp, Fwd);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RPC {
    Ping,
    // PeerInfo
    NewPeer(Vec<u8>),
    // Key -> Value
    Store([u8; 32], Vec<u8>),
    // PeerInfo
    FindNode(Vec<u8>),
    // PeerInfo
    FindValue(Vec<u8>),
    // Vector of u8 PeerInfo::as_bytes()
    Nodes(Vec<Vec<u8>>),
    // Vector of value in bytes
    Value(Vec<u8>),
    // Key of value saved. 
    // Either stored value key or
    // PeerKey
    Saved([u8; 32]),
    Pong(Vec<u8>),

}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Req {
    // MessageKey::inner()
    pub id: [u8; 32],
    // PeerInfo::as_bytes()
    pub sender: Vec<u8>,
    // RPC::as_bytes()
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resp {
    // Req::as_bytes()
    pub request: Vec<u8>,
    // PeerInfo::as_bytes()
    pub receiver: Vec<u8>,
    // RPCResponse::as_bytes()
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fwd {
    // Req::as_bytes()
    pub request: Vec<u8>,
    pub n_fwd: usize,
    pub n_resp: usize,
    pub timeout: u128,
    pub resps: Vec<Resp>,
    pub discovered: HashSet<PeerInfo>,
}

impl Req {
    pub fn to_components(&self) -> (MessageKey, PeerInfo, RPC) {
        let rpc = RPC::from_bytes(&self.payload);
        let sender = PeerInfo::from_bytes(&self.sender);
        let id = MessageKey::from_inner(self.id);
        (id, sender, rpc)
    }
}

impl Resp {
    pub fn to_components(&self) -> (Req, PeerInfo, RPC) {
        let rpc = RPC::from_bytes(&self.payload);
        let receiver = PeerInfo::from_bytes(&self.receiver);
        let req = Req::from_bytes(&self.request);

        (req, receiver, rpc)
    }
}

pub trait Request {}
pub trait Response {}

#[macro_export]
macro_rules! impl_Request {
    (for $($t:ty), +) => {
        $(impl Request for $t {})*
    };
}

#[macro_export]
macro_rules! impl_Response {
    (for $($t:ty), +) => {
        $(impl Response for $t {})*
    };
}

impl_Request!(for Req, RPC, KadMessage);
impl_Response!(for Resp, RPC, KadMessage);

