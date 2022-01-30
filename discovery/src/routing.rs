use crate::{MAX_BUCKETS, MAX_BUCKET_LEN, REFRESH_INTEVAL};
use node::peer_info::PeerInfo;
use node::peer_key::Key;
use utils::utils::Distance;
use std::hash::Hash;
use std::{cmp, mem};
use utils::utils::{timestamp_now};
use std::collections::{BTreeMap, HashMap};
use ritelinked::LinkedHashMap;
use node::peer_id::PeerId;
use std::net::SocketAddr;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct KBucket {
    nodes: LinkedHashMap<PeerId, PeerInfo>,
    last_updated: u128,
}

impl KBucket {
    pub fn new() -> Self {
        KBucket {
            nodes: LinkedHashMap::with_capacity(MAX_BUCKET_LEN),
            last_updated: timestamp_now(),
        }
    }

    pub fn upsert(&mut self, peer: &PeerInfo) {
        self.last_updated = timestamp_now();
        if !self.nodes.contains_key(&peer.id) {
            self.nodes.insert(peer.id.clone(), peer.clone());    
        }
    }

    pub fn contains(&self, node: &PeerInfo) -> bool {
        self.nodes.contains_key(&node.id)
    }

    pub fn split(&mut self, index: usize, prefix: String) -> KBucket {
        let mut new_bucket = KBucket::new();        
        return new_bucket
    }

    pub fn get_nodes(&self) -> Vec<PeerInfo> {
        self.nodes.clone().iter().map(|(k, v)| v.clone()).collect()
    }

    pub fn remove_lru(&mut self) -> Option<PeerInfo> {
        if self.size() == 0 {
            None
        } else {
            Some(self.nodes.pop_front().unwrap().1)
        }
    }

    pub fn remove_peer(&mut self, peer: &PeerInfo) -> Option<PeerInfo> {
        self.nodes.remove(&peer.id)
    }

    pub fn is_full(&self) -> bool {
        self.nodes.len() >= MAX_BUCKET_LEN
    }

    pub fn is_stale(&self) -> bool {
        let diff = timestamp_now() - self.last_updated;
        diff > REFRESH_INTEVAL
    }

    pub fn size(&self) -> usize {
        self.nodes.len()
    }
}

#[derive(Debug, Clone)]
pub struct RoutingTable {
    pub tree: HashMap<String, KBucket>,
    pub local_info: PeerInfo,
}

impl RoutingTable {

    pub fn new(local_info: PeerInfo) -> Self {
        let mut tree = HashMap::new();
        let mut kbucket = KBucket::new();
        kbucket.upsert(&local_info);
        tree.insert(local_info.get_key().xor(local_info.get_key()).get_prefix(0), kbucket);
        RoutingTable { tree, local_info }
    }

    pub fn update_peer(&mut self, peer_info: &PeerInfo, traverse: usize) -> bool {
        // Get the distance prefix of the peer
        let distance = self.local_info.get_key().xor(peer_info.get_key());
        let prefix = distance.get_prefix(traverse);
        if let Some(bucket) = self.tree.get_mut(&prefix) {
            if !bucket.is_full() {
                bucket.upsert(peer_info);
                return true
            } else {
                self.update_peer(peer_info, traverse + 1)
            }
        } else {
            let mut new_bucket = KBucket::new();
            new_bucket.upsert(peer_info);
            self.tree.insert(prefix, new_bucket);
            return true
        }
    }

    pub fn get_closest_peers(&self, peer_info: PeerInfo, count: usize) -> Vec<PeerInfo> {
        if self.total_peers() < count {
            return self.get_all_peers()
        }
        
        let distance = self.local_info.get_key().xor(peer_info.get_key());
        let mut cloned_tree = self.tree.clone();
        cloned_tree.retain(|k, bucket| {
            let prefix = distance.get_prefix(k.len()-1);
            prefix == *k 
        });

        let total = cloned_tree.iter().fold(0, |acc, (k, v)| acc + v.size());

        if cloned_tree.is_empty() || total < count {
            let mut closest = self.get_all_peers();
            closest.sort_by_key(|peer| peer.get_key().xor(peer_info.get_key()));
            closest.truncate(count);
            return closest
        }

        let mut closest: Vec<_> = cloned_tree.iter().map(|(k, v)| {
            (k, v.get_nodes())
        }).collect();

        closest.sort_unstable_by_key(|(k, v)| k.len());
        closest.reverse();
        let mut iter = closest.iter();
        let mut ret = {
            if let Some((k, nodes)) = iter.next() {
                nodes.clone()
            } else {
                vec![]
            }
        };

        while ret.len() < count {
            if let Some((k, nodes)) = iter.next() {
                ret.extend(nodes.clone())
            } else {
                break
            }
        }
        
        ret.sort_by_key(|peer| peer.get_key().xor(peer_info.get_key()));
        ret.truncate(count);
        ret
    }

    pub fn remove_lru(&mut self, peer_key: &[u8; 32]) -> Option<PeerInfo> {
        let index = cmp::min(
            self.local_info.get_key().xor(Key::new(*peer_key)).leading_zeros(),
            self.tree.len() - 1,
        );

        let prefix = self.local_info.get_key().xor(Key::new(*peer_key)).get_prefix(index);

        if let Some(bucket) = self.tree.get_mut(&prefix) {
            return bucket.remove_lru();
        }

        None
    }
    pub fn remove_peer(&mut self, peer_info: &PeerInfo) {
        let index = cmp::min(
            self.local_info.get_key().xor(peer_info.get_key()).leading_zeros(),
            self.tree.len() - 1,
        );

        let prefix = self.local_info.get_key().xor(peer_info.get_key()).get_prefix(index);
        
        if let Some(bucket) = self.tree.get_mut(&prefix) {
            println!("{}", bucket.size());
            bucket.remove_peer(peer_info);
            println!("{}", bucket.size());
        }
    }

    pub fn get_stale_indices(&self) -> Vec<usize> {
        let mut ret = Vec::new();
        self.tree.iter().enumerate().for_each(|(idx, (_, bucket))| {
            if bucket.is_stale() {
                ret.push(idx)
            }
        });

        ret
    }

    pub fn is_new(&self, peer: &PeerInfo) -> bool {
        !self.tree.iter().any(|(_, bucket)| bucket.contains(peer))
    }

    pub fn size(&self) -> usize {
        self.tree.len()
    }

    pub fn total_peers(&self) -> usize {
        let mut sum = 0;
        self.tree.iter().for_each(|(k, v)| {
            sum += v.size();
        });

        sum
    }

    pub fn get_all_peers(&self) -> Vec<PeerInfo> {
        self.tree.iter().map(|(k, v)| {
            v.get_nodes()
        }).collect::<Vec<_>>().into_iter().flatten().collect()
    }
}
