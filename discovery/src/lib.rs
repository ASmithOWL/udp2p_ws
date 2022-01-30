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
const REQUEST: &str = "Request";
const RESPONSE: &str = "Response";

#[cfg(test)]
mod tests {

    use crate::kad::Kademlia;
    use crate::routing::RoutingTable;
    use node::peer_id::PeerId;
    use node::peer_key::Key;
    use node::peer_info::PeerInfo;
    use rand::Rng;
    use std::net::SocketAddr;
    use utils::utils::Distance;
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

    // #[test]
    // fn kad_new_works() {
    //     let (kad, _, _) = setup(1);
    //     assert!(kad.routing_table.size() >= 1);
    // }

    // #[test]
    // fn kad_kbuckets_returns_vector_of_kbuckets() {
    //     let (kad, local, peer) = setup(1);
    //     let kbuckets = kad.kbuckets();
    //     assert!(kbuckets.len() >= 1);
    // }

    // #[test]
    // fn kad_kbucket_returns_single_kbucket() {
    //     let (kad, local, peers) = setup(1);
    //     let peer = peers[0].clone();
    //     let local_kbucket = kad.kbucket(local.clone());
    //     let peer_kbucket = kad.kbucket(peer.clone());
    //     assert!(local_kbucket.contains(local));
    //     assert!(peer_kbucket.contains(peer));
    // }

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

    // #[test]
    // fn kad_remove_peer_works() {
    //     let (mut kad, local, peers) = setup(5);
    //     let peer = peers[0].clone();
    //     let test_kad = kad.routing_table.tree.iter().any(|(k, v)| {
    //         v.contains(peer.clone())
    //     });
    //     assert_eq!(test_kad, true);
    //     assert!(kad.routing_table.size() <= 2);

    //     kad.remove_peer(peer.clone());
    //     let remove_test_kad = kad.routing_table.tree.iter().any(|(k, v)| {
    //         v.contains(peer.clone())
    //     });
    //     assert_eq!(remove_test_kad, false);

    // }
    
    // #[test]
    // fn kad_remove_peer_address_works() {
    //     let key = Key::rand();
    //     let id = PeerId::from_key(&key);
    //     let socket_address: SocketAddr = "127.0.0.1:9292".parse().expect("Unable to parse socket address");
    //     let addresses = Some(vec![socket_address.clone()]);
    //     let peer_info = PeerInfo::new(id, key, addresses);
    
    //     let new_key = Key::rand();
    //     let new_id = PeerId::from_key(&key);
    //     let new_socket_address: SocketAddr = "127.0.0.1:19292".parse().expect("Unable to parse socket address");
    //     let new_addresses = Some(vec![new_socket_address.clone()]);
    
    //     let peer_to_add = PeerInfo::new(new_id, new_key, new_addresses);
    //     let mut kad = Kademlia::new(peer_info);
    //     kad.add_address(peer_to_add.clone());
        
    //     kad.remove_address(&peer_to_add.clone(), &new_socket_address);
    //     assert_eq!(kad.routing_table.tree.iter().any(|(k, v)| v.contains(peer_to_add.clone())), true);
    //     let mut cloned_tree = kad.routing_table.tree.clone();
    //     cloned_tree.retain(|k, v| v.contains(peer_to_add.clone()));
    //     let blank_address = cloned_tree.iter().any(|(k, bucket)| bucket.get_nodes().iter().any(|v| v.addresses == Some(vec![])));
    //     assert!(blank_address);
    // }
}
