#![allow(dead_code)]
use std::sync::mpsc::Sender;
use protocol::protocol::{AckMessage, Message, KadMessage, Packet, Header};
use std::net::{SocketAddr, UdpSocket};
use gd_udp::gd_udp::GDUdp;
use std::collections::HashMap;
use utils::utils::ByteRep;

pub struct MessageHandler {
    om_tx: Sender<(SocketAddr, Message)>,
    ia_tx: Sender<AckMessage>,
    pending: HashMap<[u8; 32], HashMap<usize, Packet>>,
    kad_tx: Sender<(SocketAddr, KadMessage)>,
    gossip_tx: Sender<(SocketAddr, Message)>,
}

impl MessageHandler {
    pub fn new(
        om_tx: Sender<(SocketAddr, Message)>,
        ia_tx: Sender<AckMessage>,
        pending: HashMap<[u8; 32], HashMap<usize, Packet>>,
        kad_tx: Sender<(SocketAddr, KadMessage)>,
        gossip_tx: Sender<(SocketAddr, Message)>
    ) -> MessageHandler {
        MessageHandler {
            om_tx,
            ia_tx,
            pending,
            kad_tx,
            gossip_tx,
        }

    }
    pub fn recv_msg(&mut self, sock: &UdpSocket, buf: &mut [u8], local: SocketAddr) {
        let res = sock.recv_from(buf);
        match res {
            Ok((amt, src)) => {
                let packet = self.process_packet(local, buf.to_vec(), amt, src);
                self.insert_packet(packet, src)
            }
            Err(_) => {}
        }
    }

    pub fn process_packet(&self, local: SocketAddr, buf: Vec<u8>, amt: usize, src: SocketAddr) -> Packet {
        let packet = Packet::from_bytes(&buf[..amt]);
        if packet.ret == GDUdp::RETURN_RECEIPT {
            let ack = AckMessage {
                packet_id: packet.id,
                packet_number: packet.n,
                src: local.to_string().as_bytes().to_vec()
            };
            let header = Header::Ack;
            let message = Message {
                head: header,
                msg: ack.as_bytes()
            };

            if let Err(_) = self.om_tx.clone().send((src, message)) {
                println!("Error sending ack message to transport thread");
            }
        }

        return packet
    }

    pub fn insert_packet(&mut self, packet: Packet, src: SocketAddr) {
        if let Some(map) = self.pending.clone().get_mut(&packet.id) {
            map.entry(packet.n).or_insert(packet.clone());
            self.pending.insert(packet.id, map.clone());
            if map.len() == packet.total_n {
                let message = self.assemble_packets(packet.clone(), map.clone());
                self.handle_message(message, src);
            }
        } else {
            if packet.total_n == 1 {
                let bytes = hex::decode(&packet.bytes).unwrap();
                let message = Message::from_bytes(&bytes);
                self.handle_message(message, src);
            } else {
                let mut map = HashMap::new();
                map.insert(packet.n, packet.clone());
                self.pending.insert(packet.id, map);
            }
        }
    }

    fn assemble_packets(&self, packet: Packet, map: HashMap<usize, Packet>) -> Message {
        let mut bytes = vec![];
        (1..=packet.total_n)
            .into_iter()
            .for_each(|n| {
                let converted = hex::decode(&map[&n].bytes.clone()).unwrap();
                bytes.extend(converted)
            });
        Message::from_bytes(&bytes)
    } 

    fn handle_message(&self, message: Message, src: SocketAddr) {
        match message.head {
            Header::Request | Header::Response => {
                let msg = KadMessage::from_bytes(&message.msg);
                if let Err(_) = self.kad_tx.send((src, msg)) {
                    println!("Error sending to kad");
                }
            }
            Header::Ack => {
                let ack = AckMessage::from_bytes(&message.msg);
                if let Err(_) = self.ia_tx.send(ack) {
                    println!("Error sending ack message")
                }
            }
            Header::Gossip => {
                if let Err(_) = self.gossip_tx.send((src, message)) {
                    println!("Error sending to gossip");
                }
            }
        }
    }
}