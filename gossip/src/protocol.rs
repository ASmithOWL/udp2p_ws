use std::net::SocketAddr;
use utils::impl_ByteRep;
use utils::utils::ByteRep;
use serde::{Serialize, Deserialize};
use protocol::protocol::{InnerKey, MessageData};

impl_ByteRep!(for GossipMessage);

/// The gossip message 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipMessage {
    pub id: InnerKey,
    pub data: MessageData,
    pub sender: SocketAddr,
}