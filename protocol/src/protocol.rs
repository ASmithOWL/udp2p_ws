use utils::utils::ByteRep;
use utils::impl_ByteRep;
use serde::{Deserialize, Serialize};

impl_ByteRep!(for Packet, AckMessage, Message, MessageKey, Header, KadMessage);

/// There are many instances where a byte
/// representation of a given struct or enu
/// is passed through a function or used for something
/// To make it easier to distinguish what the byte vector
/// is intending to represent, we have some custom types
/// used instead of passsing in a Vec<u8> alone.
pub type Peer = Vec<u8>;
pub type RequestBytes = Vec<u8>;
pub type ResponseBytes = Vec<u8>;
pub type Value = Vec<u8>;
pub type Nodes = Vec<Vec<u8>>;
pub type StoreKey = [u8; 32];
pub type InnerKey = [u8; 32];
pub type RPCBytes = Vec<u8>;
pub type MessageData = Vec<u8>;
pub type ReturnReceipt = u8;
pub type AddressBytes = Vec<u8>;
pub type Packets = Vec<Packet>;

pub trait Protocol {}

/// A trait implemented on objects that need to be sent across
/// the network, whether kademlia, gossip, or some other protocol
pub trait Packetize<'a>: ByteRep<'a> {
    fn packetize(&self) -> Vec<Packet>;
}

/// A function that returns a vector of *n* Packet(s) based on the size of
/// the MessageData passed to it.
/// 
/// # Arguments
/// 
/// * bytes - a vector of u8 bytes representing the message data to be split up into packets
/// * id - a common id shared by all packets derived from the same message for reassembly by the receiver
/// * ret - a 0 or 1 representing whether a return receipt is required of the sender
/// 
/// TODO:
/// 
/// Build a macro for this
pub fn packetize(bytes: MessageData, id: InnerKey, ret: ReturnReceipt) -> Packets {
    if bytes.len() < 32500 {
        let hex_string = hex::encode(&bytes);
        let packet = Packet {
                id,
                n: 1,
                total_n: 1,
                bytes: hex_string,
                ret
        };
        return vec![packet]
    }
    let mut n_packets = bytes.len() / 32500;
    if n_packets % 32500 != 0 {
        n_packets += 1;
    }
    let mut start = 0;
    let mut end = 32500;
    let mut packets = vec![];

    for n in 0..n_packets {
        if n == n_packets-1 {
            packets.push(bytes[start..].to_vec());
        } else {
            packets.push(bytes[start..end].to_vec());
            start = end;
            end += 32500;
        }
    }
    
    let packets: Packets = packets.iter().enumerate().map(|(idx, packet)| {
        let hex_string = hex::encode(&packet);
        Packet {
            id,
            n: idx + 1,
            total_n: packets.len(),
            bytes: hex_string,
            ret
        }
    }).collect();

    packets
}

/// Packet contains a common id derived from the message
/// n is the packet number, total_n is the total number of packets
/// generated by the message, bytes is a hexadecimal string representaiton
/// of the MessageData broken down into packet(s)
/// ret is a 0 or 1 representing a return receipt.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Packet {
    pub id: InnerKey,
    pub n: usize,
    pub total_n: usize,
    pub bytes: String,
    pub ret: ReturnReceipt,
}

/// An acknowledge message sent back to the sender of a packet
/// in response to a return receipt being required by a given packet
/// Ack messages contain the packet's common, derived id that identifies
/// which message the packet was derived from, the packet number of the
/// packet being acknowledged and src a byte representation of a socket address.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AckMessage {
    pub packet_id: InnerKey,
    pub packet_number: usize,
    pub src: AddressBytes
}

/// Headers to identify and route messages to the proper component
/// Request, Response, Gossip and Ack allows for easy routing once
/// the packets have been received and aggregated into a message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Header {
    Request,
    Response,
    Gossip,
    Ack,
}

/// A message struct contains a header and message data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub head: Header,
    pub msg: MessageData,
}

/// A tuple struct containing a byte representation of a 256 bit key
#[derive(Ord, PartialOrd, PartialEq, Eq, Clone, Hash, Serialize, Deserialize, Default, Copy, Debug)]
pub struct MessageKey(InnerKey);

/// TODO: Create a trait called Key to apply to all the different types of keys with the same required
/// functionality
impl MessageKey {
    /// generates a new Message Key from a 32 byte array representing a 256 bit key
    pub fn new(v: InnerKey) -> Self {
        MessageKey(v)
    }

    /// Generate a random key
    pub fn rand() -> Self {
        let mut ret = MessageKey([0; 32]);
        ret.0.iter_mut().for_each(|k| {
            *k = rand::random::<u8>();
        });

        ret
    }

    /// generate a random key within a given range
    pub fn rand_in_range(idx: usize) -> Self {
        let mut ret = MessageKey::rand();
        let bytes = idx / 8;
        let bit = idx % 8;
        (0..bytes).into_iter().for_each(|i| {
            ret.0[i] = 0;
        });
        ret.0[bytes] &= 0xFF >>(bit);
        ret.0[bytes] |= 1 << (8 - bit - 1);

        ret
    }

    /// Return the inner key
    pub fn inner(&self) -> InnerKey {
        self.0
    }

    /// Return the Message Key given an Inner key
    pub fn from_inner(v: InnerKey) -> MessageKey {
        MessageKey(v)
    }
}

/// A message for the Kademlia DHT protocol
/// 3 different variants, Request, Response, and Kill
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum KadMessage {
    Request(RequestBytes),
    Response(ResponseBytes),
    Kill,
}