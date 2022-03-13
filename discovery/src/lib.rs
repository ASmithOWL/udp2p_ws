#![allow(unused)]
pub mod routing;
pub mod protocol;
pub mod kad;

const MAX_BUCKET_LEN: usize = 30;
const MAX_BUCKETS: usize = 10;
const REFRESH_INTEVAL: u128 = 900_000_000_000;
const KAD_MESSAGE_LEN: usize = 55000;
const REQ_TIMEOUT: usize = 60_000_000_000;
const MAX_ACTIVE_RPCS: usize = 3;
const DEFAULT_N_PEERS: usize = 8;

#[cfg(test)]
mod tests {

    use crate::kad::Kademlia;
    use crate::routing::RoutingTable;
    use udp2p_node::peer_id::PeerId;
    use udp2p_node::peer_key::Key;
    use udp2p_node::peer_info::PeerInfo;
    use rand::Rng;
    use std::net::SocketAddr;
    use udp2p_utils::utils::Distance;
    use std::cmp;

    fn setup(n_peers: usize) -> (RoutingTable, PeerInfo, Vec<PeerInfo>) {
        let local_key = Key::rand();
        let local_id = PeerId::from_key(&local_key);
        let rn: usize = rand::thread_rng().gen_range(9292..19292);
        let local_addr: SocketAddr = format!("127.0.0.1:{}", rn).parse().expect("Unable to parse address");
        let local_info = PeerInfo::new(local_id, local_key, local_addr);
        let routing_table = RoutingTable::new(local_info.clone());
        let mut peers = vec![];
        for _ in 0..n_peers {
            let peer_key = Key::rand();
            let peer_id = PeerId::from_key(&peer_key);
            let rn: usize = rand::thread_rng().gen_range(9292..19292);
            let peer_addr: SocketAddr = format!("127.0.0.1:{}", rn).parse().expect("Unable to parse address");
            let peer_info = PeerInfo::new(peer_id, peer_key, peer_addr);
            peers.push(peer_info);
        }

        (routing_table, local_info, peers)
        
    }

    #[test]
    fn kad_add_address_works() {
        let (mut rt, local, peers) = setup(5);
        let peer = peers[0].clone();
        let test_kad = rt.tree.iter().any(|(k, v)| {
            v.contains(&peer)
        });

        assert_eq!(test_kad, false);
        rt.update_peer(&peer, 0);
        let test_kad = rt.tree.iter().any(|(k, v)| {
            v.contains(&peer)
        });

        assert_eq!(test_kad, true);
        assert!(rt.size() <= 2);
    }

    #[test]
    fn kad_split_bucket_works() {
        let (mut rt, local, peers) = setup(90);
        let rn: usize = rand::thread_rng().gen_range(0..peers.len()-1);
        let random_peer = peers[rn].clone();
        let test_kad = rt.tree.iter().any(|(k, v)| {
            v.contains(&random_peer)
        });
        assert!(!test_kad);

        peers.iter().for_each(|peer| {
            rt.update_peer(peer, 0);
        });

        let test_kad = rt.tree.iter().any(|(k, v)| {
            v.contains(&random_peer)
        });

        let mut sum = 0;
        rt.tree.iter().for_each(|(k, v)| {
            sum += v.size();
        });

        assert!(sum == 91);
        assert!(rt.size() > 2);
        assert!(test_kad);

    }

    #[test]
    fn kad_get_closest_peers_works() {
        let (mut kad, local, peers) = setup(90);
        peers.iter().for_each(|peer| {
            kad.update_peer(peer, 0);
        });
        let peer = peers[5].clone();
        let four_closest_peers = kad.get_closest_peers(peer.clone(), 4);
        let eight_closest_peers = kad.get_closest_peers(peer.clone(), 8);
        let twelve_closest_peers = kad.get_closest_peers(peer.clone(), 12);
        let fifty_closest_peers = kad.get_closest_peers(peer.clone(), 50);
        assert_eq!(four_closest_peers.len(), 4);
        assert_eq!(eight_closest_peers.len(), 8);
        assert_eq!(twelve_closest_peers.len(), 12);
        assert_eq!(fifty_closest_peers.len(), 50);
        
    }
}
