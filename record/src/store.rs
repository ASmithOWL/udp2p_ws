#![allow(unused_imports)]
use std::collections::{BTreeMap, HashMap, HashSet};
use std::mem;
use udp2p_utils::utils::timestamp_now;
use udp2p_node::peer_id::PeerId;
use udp2p_node::peer_info::PeerInfo;
use std::error::Error;

/// A trait applied to any kind of key value storing
/// struct that is used to maintain records and providers
/// of those records for a peer to peer network.
pub trait Store {
    type Record;
    type Provision;
    type Key;
    type RecordIter: Iterator;
    type ProvisionIter: Iterator;

    fn get(&self, key: &Self::Key) -> Option<Self::Record>;
    fn put(&mut self, record: Self::Record) -> Result<(), Box<dyn Error>>;
    fn remove(&mut self, key: &Self::Key);
    fn records(&self) -> Self::RecordIter;
    fn add_provider(&mut self, provider: Self::Provision) -> Result<(), Box<dyn Error>>;
    fn providers(&self, key: &Self::Key) -> Vec<Self::Provision>;
    fn provided(&self) -> Self::ProvisionIter;
    fn remove_provider(&mut self, key: &Self::Key, peer: &PeerId);
}