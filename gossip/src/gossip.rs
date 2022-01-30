#![allow(dead_code)]
use crate::protocol::GossipMessage;
use discovery::kad::Kademlia;
use protocol::protocol::{Message, MessageKey};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use utils::utils::ByteRep;
use std::time::Instant;
use rand::Rng;
use udp2p::routable::Routable;

pub struct GossipConfig {
    // Protocol ID
    id: String,
    // Number of heartbeats to keep in cache
    history_len: usize,
    // Number of past heartbeats to gossip about
    history_gossip: usize,
    // Target number of peers
    target: usize,
    // Minimum number of peers
    low: usize,
    // Maximum number of peers
    high: usize,
    // Minimum number of peers to disseminate gossip to
    min_gossip: usize,
    // % of peers to send gossip to
    factor: f64,
    // Time between heartbeats
    interval: Duration,
    check: usize,
}

pub struct GossipService {
    address: SocketAddr,
    to_gossip_rx: Receiver<(SocketAddr, Message)>,
    to_transport_tx: Sender<(SocketAddr, Message)>,
    to_app_tx: Sender<GossipMessage>,
    pub kad: Kademlia,
    cache: HashMap<MessageKey, (Message, Instant)>,
    config: GossipConfig,
    heartbeat: Instant,
    ping_pong: Instant,
    // add pending pings
}

impl GossipConfig {
    pub fn new(
        id: String,
        history_len: usize,
        history_gossip: usize,
        target: usize,
        low: usize,
        high: usize,
        min_gossip: usize,
        factor: f64,
        interval: Duration,
        check: usize,
    ) -> GossipConfig {
        GossipConfig {
            id,
            history_len,
            history_gossip,
            target,
            low,
            high,
            min_gossip,
            factor,
            interval,
            check
        }
    }

    pub fn min(&self) -> usize {
        self.min_gossip
    }

    pub fn max(&self) -> usize {
        self.high
    }
}

impl GossipService {
    pub fn new(
        address: SocketAddr,
        to_gossip_rx: Receiver<(SocketAddr, Message)>,
        to_transport_tx: Sender<(SocketAddr, Message)>,
        to_app_tx: Sender<GossipMessage>,
        kad: Kademlia,
        config: GossipConfig,
        heartbeat: Instant,
        ping_pong: Instant,
    ) -> GossipService {
        GossipService {
            address,
            to_gossip_rx,
            to_transport_tx,
            to_app_tx,
            cache: HashMap::new(),
            kad,
            config,
            heartbeat,
            ping_pong,
        }
    }

    pub fn start(&mut self) {
        loop {
            self.kad.recv();
            self.recv();
            self.gossip();
        }
    }

    pub fn heartbeat(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.heartbeat) > self.config.interval {
            self.heartbeat = now;
            return true
        }
        false 
    }

    pub fn gossip(&mut self) {
        let now = Instant::now();
        let cache_clone = self.cache.clone();
        if self.heartbeat() {
            cache_clone.iter().for_each(|(key, (message, expires))| {
                if now.duration_since(*expires) < self.config.interval * self.config.history_gossip as u32 {
                    let gossip_message = GossipMessage::from_bytes(&message.msg);
                    let src = gossip_message.sender;
                    self.publish(&src, message.clone())     
                }
                if now.duration_since(*expires) > self.config.interval * self.config.history_len as u32 {
                    self.cache.remove(&key);
                }
            });

            // TODO: Add Ping Pong messages every x heartbeats.
            if now.duration_since(self.ping_pong) > self.config.interval * self.config.check as u32 {
                // Send Ping message to peers
            }
        }
    }

    pub fn publish(&mut self, src: &SocketAddr, message: Message) {
        let local = self.kad.routing_table.local_info.clone();
        let gossip_to = {
            let mut sample = HashSet::new();
            let peers = self.kad.routing_table.get_closest_peers(local, 30).clone();
            if peers.len() > 7 {

                let infection_factor = self.config.factor;
                let n_peers = peers.len() as f64 * infection_factor;
                for _ in 0..n_peers as usize {
                    let rn: usize = rand::thread_rng().gen_range(0..peers.len());
                    let address = peers[rn].get_address();
                    if &address != src && address != self.address {
                        sample.insert(address);
                    }
                }
            
                sample

            } else {
                peers.iter().for_each(|peer| {
                    let address = peer.get_address();
                    if address != self.address {
                        sample.insert(address);
                    }
                });

                sample
            }
        };

        gossip_to.iter().for_each(|peer| {
            if let Err(_) = self.to_transport_tx.send((peer.clone(), message.clone())) {
                println!("Error forwarding to transport")
            }
        });
    }

    fn handle_message(&mut self, src: &SocketAddr, msg: &Message) {
        let message = GossipMessage::from_bytes(&msg.msg);
        if !self.cache.contains_key(&MessageKey::from_inner(message.id)) {
            if *src != self.address {
                println!("Received message from {:?}", src);
                let string = String::from_utf8_lossy(&message.data);
                println!("{:?}", string);
            }
            let key = {
                let gossip_mesasge = GossipMessage::from_bytes(&msg.msg);
                MessageKey::from_inner(gossip_mesasge.id)
            };
            self.publish(src, msg.clone());
            self.cache.entry(key).or_insert((msg.clone(), Instant::now()));
        }
    }

    pub fn recv(&mut self) {
        let res = self.to_gossip_rx.try_recv();
        match res {
            Ok((src, msg)) => {
                self.handle_message(&src, &msg);
            }
            Err(_) => {}
        }
    }
}
