use crate::peer_key::Key;
use serde::{Serialize, Deserialize};
use utils::utils::ByteRep;
use utils::impl_ByteRep;
use sha256::digest_bytes;

impl_ByteRep!(for PeerId);

#[derive(Clone, Debug, Hash, Serialize, Deserialize, Eq)]
pub struct PeerId(String);

impl PeerId {
    pub fn get_id(&self) -> String {
        self.0.clone()
    }

    pub fn rand() -> Self {
        let key = Key::rand();
        PeerId::from_key(&key)
    }

    pub fn rand_in_range(idx: usize) -> Self {
        let key = Key::rand_in_range(idx);
        PeerId::from_key(&key)
    }

    pub fn from_key(key: &Key) -> Self {
        let key_hash = digest_bytes(&key.get_key());
        PeerId(key_hash)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        serde_json::to_string(&self.clone()).unwrap().as_bytes().to_vec()
    }

    pub fn from_str(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}

impl PartialEq for PeerId {
    fn eq(&self, other: &PeerId) -> bool {
        self.get_id().eq(&other.get_id())
    }
}