use utils::utils::ByteRep;
use utils::impl_ByteRep;
use serde::{Deserialize, Serialize};

impl_ByteRep!(for Packet, AckMessage, Message, MessageKey, Header, KadMessage);

pub trait Protocol {}

pub trait Packetize<'a>: ByteRep<'a> {
    fn packetize(&self) -> Vec<Packet>;
}

pub fn packetize(bytes: Vec<u8>, id: [u8; 32], ret: u8) -> Vec<Packet> {
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
    
    let packets: Vec<Packet> = packets.iter().enumerate().map(|(idx, packet)| {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Packet {
    pub id: [u8; 32],
    pub n: usize,
    pub total_n: usize,
    pub bytes: String,
    pub ret: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AckMessage {
    pub packet_id: [u8; 32],
    pub packet_number: usize,
    // SocketAddr;
    pub src: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Header {
    Request,
    Response,
    Gossip,
    Ack,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub head: Header,
    pub msg: Vec<u8>,
}

#[derive(Ord, PartialOrd, PartialEq, Eq, Clone, Hash, Serialize, Deserialize, Default, Copy, Debug)]
pub struct MessageKey([u8; 32]);

impl MessageKey {
    pub fn new(v: [u8; 32]) -> Self {
        MessageKey(v)
    }

    pub fn rand() -> Self {
        let mut ret = MessageKey([0; 32]);
        ret.0.iter_mut().for_each(|k| {
            *k = rand::random::<u8>();
        });

        ret
    }

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

    pub fn inner(&self) -> [u8; 32] {
        self.0
    }

    pub fn from_inner(v: [u8; 32]) -> MessageKey {
        MessageKey(v)
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum KadMessage {
    //Req::as_bytes()
    Request(Vec<u8>),
    //Resp::as_bytes()
    Response(Vec<u8>),
    Kill,
}