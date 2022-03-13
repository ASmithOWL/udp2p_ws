use std::net::SocketAddr;
use udp2p_utils::impl_ByteRep;
use udp2p_utils::utils::ByteRep;
use serde::{Serialize, Deserialize};
use udp2p_protocol::protocol::{InnerKey, MessageData};

impl_ByteRep!(for GossipMessage);

/// The gossip message 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipMessage {
    pub id: InnerKey,
    pub data: MessageData,
    pub sender: SocketAddr,
}