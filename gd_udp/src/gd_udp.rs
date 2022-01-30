use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::{Duration, Instant};
use protocol::protocol::Packet;
use utils::utils::ByteRep;


#[derive(Debug, Clone)]
pub struct GDUdp {
    pub addr: SocketAddr,
    pub message_cache: HashSet<[u8; 32]>,
    pub outbox: HashMap<[u8; 32], HashMap<usize, (HashSet<SocketAddr>, HashSet<SocketAddr>, Packet, usize)>>,
    pub timer: Instant,
    pub log: String,
}

impl GDUdp {
    pub const MAINTENANCE: Duration = Duration::from_millis(300);
    pub const RETURN_RECEIPT: u8 = 1u8;
    pub const NO_RETURN_RECEIPT: u8 = 0u8;

    pub fn new(addr: SocketAddr) -> GDUdp {
        GDUdp {
            addr,
            message_cache: HashSet::new(),
            outbox: HashMap::new(),
            timer: Instant::now(),
            log: "log.log".to_string(),
        }
    }

    pub fn maintain(&mut self, sock: &UdpSocket) {
        self.outbox.retain(|_, map| {
            map.retain(|_, (sent_set, ack_set, _, attempts)| sent_set != ack_set && *attempts < 5);
            !map.is_empty()
        });
        
        self.outbox.clone().iter().for_each(|(_, map)| {
            map.iter().for_each(|(_, (sent_set, ack_set, packet, attempts))| {
                if *attempts < 5 {
                    let resend: HashSet<_> = sent_set.difference(ack_set).collect();
                    resend.iter().for_each(|peer| {
                        self.send_reliable(peer, &packet, sock);
                    });
                }
            });
        });
    }

    pub fn recv_from(
        &mut self,
        sock: Arc<UdpSocket>,
        buf: &mut [u8],
    ) -> Result<(usize, SocketAddr), std::io::Error> {
        match sock.recv_from(buf) {
            Err(e) => return Err(e),
            Ok((amt, src)) => return Ok((amt, src)),
        }
    }

    pub fn check_time_elapsed(&mut self, sock: &UdpSocket) {
        let now = Instant::now();
        let time_elapsed = now.duration_since(self.timer);
        let cloned_sock = sock.try_clone().expect("Unable to clone socket");

        if time_elapsed >= GDUdp::MAINTENANCE {
            self.maintain(&cloned_sock);
            self.timer = Instant::now()
        }
    }

    pub fn process_ack(&mut self, id: [u8; 32], packet_number: usize, src: Vec<u8>) {
        let src = String::from_utf8_lossy(&src);
        let src = src.parse().expect("Unable to parse socket address");
        if let Some(map) = self.outbox.get_mut(&id) {
            if let Some((_, ack_set, _, _)) = map.get_mut(&packet_number) {
                ack_set.insert(src);
            }
        }
    }

    pub fn send_reliable(
        &mut self,
        peer: &SocketAddr,
        packet: &Packet,
        sock: &UdpSocket,
    ) {
        if let Some(map) = self.outbox.get_mut(&packet.id) {
            if let Some((sent_set, _, _, attempts)) = map.get_mut(&packet.n) {
                sent_set.insert(peer.clone());
                *attempts += 1;
            } else {
                let mut sent_set = HashSet::new();
                let ack_set: HashSet<SocketAddr> = HashSet::new();
                sent_set.insert(peer.clone());
                let attempts = 1;
                map.insert(packet.n, (sent_set, ack_set, packet.clone(), attempts));
            }
        } else {
            let mut map = HashMap::new();
            let mut sent_set = HashSet::new();
            let ack_set: HashSet<SocketAddr> = HashSet::new();
            sent_set.insert(peer.clone());
            let attempts = 1;
            map.insert(packet.n, (sent_set, ack_set, packet.clone(), attempts));
            self.outbox.insert(packet.id, map);
        }
        sock.send_to(&packet.as_bytes(), peer)
            .expect("Error sending packet to peer");
    }

    pub fn ack(&mut self, sock: &UdpSocket, peer: &SocketAddr, packets: Vec<Packet>) {
        packets.iter().for_each(|packet| {
            sock.send_to(&packet.as_bytes(), peer)
                .expect("Unable to send message to peer");
        })
    }

}
