use std::net::SocketAddr;
use utils::impl_ByteRep;
use utils::utils::ByteRep;
use serde::{Serialize, Deserialize};

impl_ByteRep!(for GossipMessage);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipMessage {
    pub id: [u8; 32],
    pub data: Vec<u8>,
    pub sender: SocketAddr,
}