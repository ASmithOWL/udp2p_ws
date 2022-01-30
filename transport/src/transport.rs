use gd_udp::gd_udp::GDUdp;
use protocol::protocol::{packetize, AckMessage, Header, Message, MessageKey};
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::Receiver;
use utils::utils::ByteRep;

#[derive(Debug)]
pub struct Transport {
    gd_udp: GDUdp,
    ia_rx: Receiver<AckMessage>,
    om_rx: Receiver<(SocketAddr, Message)>,
}

impl Transport {
    pub fn new(
        addr: SocketAddr,
        ia_rx: Receiver<AckMessage>,
        om_rx: Receiver<(SocketAddr, Message)>,
    ) -> Transport {
        Transport {
            gd_udp: GDUdp::new(addr),
            ia_rx,
            om_rx,
        }
    }

    pub fn incoming_ack(&mut self) {
        let res = self.ia_rx.try_recv();
        match res {
            Ok(ack) => {
                let exists = self.gd_udp.outbox.contains_key(&ack.packet_id);
                if exists {
                    self.gd_udp
                        .process_ack(ack.packet_id, ack.packet_number, ack.src);
                };
            }
            Err(_) => {}
        }
    }

    pub fn outgoing_msg(&mut self, sock: &UdpSocket) {
        let res = self.om_rx.try_recv();
        sock.set_nonblocking(true).unwrap();
        match res {
            Ok((src, msg)) => match msg.head {
                Header::Ack => {
                    let packets_id = MessageKey::rand().inner();
                    let packets = packetize(msg.as_bytes().clone(), packets_id, 0u8);
                    packets.iter().for_each(|packet| {
                        if let Err(_) = sock.send_to(&packet.as_bytes(), src) {}
                    });
                }
                _ => {
                    let packets_id = MessageKey::rand().inner();
                    let packets = packetize(msg.as_bytes().clone(), packets_id, 1u8);
                    packets.iter().for_each(|packet| {
                        self.gd_udp.send_reliable(&src, packet, &sock);
                    });
                }
            },
            Err(_) => {}
        }
    }

    pub fn check_time_elapsed(&mut self, sock: &UdpSocket) {
        self.gd_udp.check_time_elapsed(sock)
    }
}
