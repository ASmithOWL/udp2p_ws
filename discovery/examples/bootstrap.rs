use udp2p_discovery::kad::Kademlia;
use udp2p_protocol::protocol::{Message};
use udp2p_transport::transport::Transport;
use udp2p_transport::handler::MessageHandler;
use udp2p_discovery::routing::RoutingTable;
use udp2p_node::peer_id::PeerId;
use udp2p_node::peer_info::PeerInfo;
use udp2p_node::peer_key::Key;
use udp2p_protocol::protocol::AckMessage;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};
use std::env::args;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{channel, Receiver, Sender};
use udp2p_utils::utils::ByteRep;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    // Bind a UDP Socket to a Socket Address with a random port between
    // 9292 and 19292 on the localhost address.
    let port: usize = thread_rng().gen_range(9292..19292);
    let addr: SocketAddr = format!("127.0.0.1:{}", port)
        .parse()
        .expect("Unable to parse address");
    let sock: UdpSocket = UdpSocket::bind(addr).expect("Unable to bind to address");

    // Initiate channels for communication between different threads
    let (to_transport_tx, to_transport_rx): (
        Sender<(SocketAddr, Message)>,
        Receiver<(SocketAddr, Message)>,
    ) = channel();
    let (to_gossip_tx, _to_gossip_rx) = channel();
    let (to_kad_tx, to_kad_rx) = channel();
    let (incoming_ack_tx, incoming_ack_rx): (Sender<AckMessage>, Receiver<AckMessage>) = channel();

    // Initialize local peer information
    let key: Key = Key::rand();
    let id: PeerId = PeerId::from_key(&key);
    let info: PeerInfo = PeerInfo::new(id, key, addr.clone());

    // initialize a kademlia, transport and message handler instance
    let routing_table = RoutingTable::new(info.clone());
    let interval = Duration::from_secs(20);
    let ping_pong = Instant::now();
    let mut kad = Kademlia::new(routing_table, to_transport_tx.clone(), to_kad_rx, HashSet::new(), interval, ping_pong);
    let mut transport = Transport::new(addr.clone(), incoming_ack_rx, to_transport_rx);
    let mut message_handler = MessageHandler::new(
        to_transport_tx.clone(),
        incoming_ack_tx.clone(),
        HashMap::new(),
        to_kad_tx.clone(),
        to_gossip_tx.clone(),
    );

    // Inform the local node of their address (since the port is randomized)
    println!("{:?}", addr);
    // Clone the socket for the transport and message handling thread(s)
    let thread_sock = sock.try_clone().expect("Unable to clone socket");
    thread::spawn(move || {
        let inner_sock = thread_sock.try_clone().expect("Unable to clone socket");
        thread::spawn(move || loop {
            transport.incoming_ack();
            transport.outgoing_msg(&inner_sock);
            transport.check_time_elapsed(&inner_sock);
        });

        loop {
            let local = addr.clone();
            let mut buf = [0u8; 65536];
            message_handler.recv_msg(&thread_sock, &mut buf, local);
        }
    });

    if let Some(to_dial) = args().nth(1) {
        let bootstrap: SocketAddr = to_dial.parse().expect("Unable to parse address");
        kad.bootstrap(&bootstrap);
    } else {
        kad.add_peer(info.as_bytes().unwrap())
    }

    loop {
        kad.recv()
    }
}
