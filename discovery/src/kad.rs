use crate::protocol::{Fwd, Req, Resp, RPC};
use crate::routing::RoutingTable;
use crate::{DEFAULT_N_PEERS, REQUEST, RESPONSE};
use node::peer_info::PeerInfo;
use node::peer_key::Key;
use protocol::protocol::{Header, KadMessage, Message, MessageKey};
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::mpsc::{channel, Receiver, Sender};
use utils::utils::timestamp_now;
use utils::utils::ByteRep;
use utils::utils::Distance;
use udp2p::routable::Routable;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Kademlia {
    pub routing_table: RoutingTable,
    pub to_transport: Sender<(SocketAddr, Message)>,
    pub from_transport: Receiver<(SocketAddr, KadMessage)>,
    pub pending: HashSet<MessageKey>,
    interval: Duration,
    ping_pong: Instant,
}

impl Kademlia {
    pub fn new(
        routing_table: RoutingTable,
        to_transport: Sender<(SocketAddr, Message)>,
        from_transport: Receiver<(SocketAddr, KadMessage)>,
        pending: HashSet<MessageKey>,
        interval: Duration,
        ping_pong: Instant,
    ) -> Kademlia {
        Kademlia {
            routing_table,
            to_transport,
            from_transport,
            pending,
            interval,
            ping_pong,
        }
    }

    pub fn recv(&mut self) {
        let res = self.from_transport.try_recv();
        match res {
            Ok((_src, msg)) => {
                self.handle_message(&msg);
            }
            Err(_) => {}
        }

        // TODO: check if its time to send pings out
        let now = Instant::now();
        if now.duration_since(self.ping_pong) > self.interval {
            // Send ping messages to peers.
        }
    }

    pub fn add_peer(&mut self, peer: Vec<u8>) {
        let peer = PeerInfo::from_bytes(&peer);
        self.routing_table.update_peer(&peer, 0);
    }

    pub fn bootstrap(&mut self, bootstrap: &SocketAddr) {
        // Structure Message
        let local_info = self.routing_table.local_info.clone();
        let (id, message) = self.prepare_find_node_message(local_info, None);
        if let Err(e) = self.to_transport.send((bootstrap.clone(), message)) {
            println!("Error sending to transport");
        }
        self.add_peer(self.routing_table.local_info.clone().as_bytes());
    }

    pub fn prepare_nodes_response_message(
        &self,
        req: Req,
        node: PeerInfo,
        nodes: Vec<PeerInfo>,
    ) -> Message {
        let nodes_vec: Vec<Vec<u8>> = nodes.iter().map(|peer| peer.as_bytes()).collect();
        let local_info = self.routing_table.local_info.clone();
        let rpc: RPC = RPC::Nodes(nodes_vec);
        let resp = Resp {
            request: req.as_bytes(),
            receiver: local_info.as_bytes(),
            payload: rpc.as_bytes(),
        };

        let msg = Message {
            head: Header::Response,
            msg: KadMessage::Response(resp.as_bytes()).as_bytes(),
        };

        msg
    }

    pub fn prepare_find_node_message(
        &self,
        peer: PeerInfo,
        req: Option<Req>,
    ) -> (MessageKey, Message) {
        if let Some(opt_req) = req {
            let message = Message {
                head: Header::Request,
                msg: KadMessage::Request(opt_req.as_bytes()).as_bytes(),
            };

            return (MessageKey::from_inner(opt_req.id), message);
        }

        let local_info = self.routing_table.local_info.clone();
        let rpc: RPC = RPC::FindNode(peer.as_bytes());
        let req: Req = Req {
            id: MessageKey::rand().inner(),
            sender: local_info.as_bytes(),
            payload: rpc.as_bytes(),
        };

        let msg = Message {
            head: Header::Request,
            msg: KadMessage::Request(req.as_bytes()).as_bytes(),
        };
        (MessageKey::from_inner(req.id), msg)
    }

    pub fn prepare_new_peer_message(&self, peer: PeerInfo) -> (MessageKey, Message) {
        let local_info = self.routing_table.local_info.clone();
        let rpc: RPC = RPC::NewPeer(peer.as_bytes());
        let req: Req = Req {
            id: MessageKey::rand().inner(),
            sender: local_info.as_bytes(),
            payload: rpc.as_bytes(),
        };

        let msg = Message {
            head: Header::Request,
            msg: KadMessage::Request(req.as_bytes()).as_bytes(),
        };
        (MessageKey::from_inner(req.id), msg)
    }

    /// Returns a tuple of a MessageKey and Message
    pub fn prepare_ping_message(&self, peer: &PeerInfo) -> (MessageKey, Message) {
        let local_info = self.routing_table.local_info.clone();
        let rpc: RPC = RPC::Ping;
        let req: Req = Req {
            id: MessageKey::rand().inner(),
            sender: local_info.as_bytes(),
            payload: rpc.as_bytes(),
        };

        let msg = Message {
            head: Header::Request,
            msg: KadMessage::Request(req.as_bytes()).as_bytes(),
        };
        (MessageKey::from_inner(req.id), msg)
    }
    
    pub fn prepare_pong_response(&self, peer: &PeerInfo, req: Req) -> Message {
        let rpc = RPC::Pong(self.routing_table.local_info.as_bytes());
        let resp = Resp {
            request: req.as_bytes(),
            receiver: peer.as_bytes(),
            payload: rpc.as_bytes()
        };

        let msg = Message {
            head: Header::Response,
            msg: KadMessage::Response(resp.as_bytes()).as_bytes(),
        };

        msg
    }

    pub fn prepare_store_message(&self, peer: &PeerInfo, value: Vec<u8>) {}
    pub fn prepare_saved_response(&self, peer: &PeerInfo, req: Req) {}
    pub fn prepare_find_value_message(&self, value: Vec<u8>) {}
    pub fn prepare_value_response(&self, peer: &PeerInfo, value: Vec<u8>, req: Req) {}
    fn handle_request(&mut self, req: &Vec<u8>) {
        let req_msg = Req::from_bytes(&req);
        let (id, sender, rpc) = req_msg.to_components();
        self.add_peer(sender.as_bytes());
        match rpc {
            RPC::FindNode(node) => {
                let peer = PeerInfo::from_bytes(&node);
                self.lookup_node(peer, req_msg.clone());
            }
            RPC::NewPeer(peer) => {
                self.add_peer(peer);
            }
            RPC::FindValue(value) => {
                // Check if you are a provider
                // if so respond with the value
                // if not, check if you know a provider
                // if so, request the value from the provider
                // if not then find the closest peers to the key
                // and respond with a nodes message.
                // Change response handler to be able to handle the
                // case of a nodes response to a FindValue request.
            }
            RPC::Store(key, value) => {
                // Add to record store, respond with
                // Saved response with the key for this record
                // so that the receiving node can add you as a
                // provider of the value.
            }
            RPC::Ping => {
                // Send pong response
                let resp_msg = self.prepare_pong_response(&sender, req_msg);
                if let Err(e) = self.to_transport.send((sender.address.clone(), resp_msg.clone())) {
                    println!("Error sending to transport: {:?}", e);
                }

            }
            _ => {
                self.handle_response(req);
            }
        }
    }

    fn handle_response(&mut self, resp: &Vec<u8>) {
        let resp_msg = Resp::from_bytes(&resp);
        let (req, receiver, rpc) = resp_msg.to_components();
        let (id, sender, req_rpc) = req.to_components();
        let mut complete = false;
        match rpc {
            RPC::Nodes(nodes) => {
                // match req.
                // IF req is a FindNode
                // then proceed with the below functionality
                nodes.iter().for_each(|peer| {
                    let peer_info = PeerInfo::from_bytes(&peer);
                    let new = self.routing_table.is_new(&peer_info);
                    self.add_peer(peer.clone());
                    if new {
                        self.bootstrap(&peer_info.get_address());
                    }
                })
                // IF the req is a FindValue then
                // attempt to find the value from the new
                // nodes received. This means that the
                // peer you requested the value from
                // didn't have it and didn't know any
                // providers of it, so they sent you
                // the closest nodes they know of to
                // the value.
                // If
            }
            RPC::Value(value) => {}
            RPC::Saved(key) => {}
            RPC::Pong(peer) => {
                // Check pending requests to ensure
                // the Ping request isn't expired
                // If it hadn't expired then
                // The peer is still alive
                // keep them in the routing table
                self.add_peer(peer)
            }
            _ => {
                self.handle_request(resp);
            }
        }
    }

    pub fn handle_message(&mut self, message: &KadMessage) {
        match message {
            KadMessage::Request(req) => {
                self.handle_request(req);
            }
            KadMessage::Response(resp) => {
                self.handle_response(&resp);
            }
            KadMessage::Kill => {}
        }
    }

    pub fn ping_node(&mut self, node: PeerInfo) {}
    pub fn pong_response(&mut self, node: PeerInfo, req: Req) {}

    pub fn lookup_node(&mut self, node: PeerInfo, req: Req) {
        let mut closest_peers = self
            .routing_table
            .get_closest_peers(node.clone(), DEFAULT_N_PEERS);
        let (_, sender, rpc) = req.to_components();
        self.add_peer(node.as_bytes());
        let (id, msg) = self.prepare_new_peer_message(node.clone());
        let resp_msg =
            self.prepare_nodes_response_message(req.clone(), node.clone(), closest_peers.clone());
        if let Err(e) = self.to_transport.send((node.address.clone(), resp_msg.clone())) {
            println!("Error sending to transport: {:?}", e);
        }
        closest_peers.iter().for_each(|peer| {
           if let Err(e) = self.to_transport.send((peer.get_address(), msg.clone())) {
               println!("Error sending to transport: {:?}", e);
            }
        });
    }

    pub fn lookup_value(&mut self, value: Vec<u8>) {}
    pub fn store_value(&mut self, value: Vec<u8>) {}
}
