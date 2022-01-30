use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

pub trait Distance {
    type Output;

    fn xor(&self, other: Self::Output) -> Self::Output;
    fn leading_zeros(&self) -> usize;
    fn leading_ones(&self) -> usize {0}
}

pub trait ByteRep<'a>: Serialize + Deserialize<'a> {
    fn as_bytes(&self) -> Vec<u8>;
    fn from_bytes(v: &[u8]) -> Self;
}

pub fn timestamp_now() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("Error getting_timestamp").as_nanos()
}

#[macro_export]
macro_rules! impl_ByteRep {
    (for $($t:ty), +) => {
        $(impl<'a> ByteRep<'a> for $t {
            fn as_bytes(&self) -> Vec<u8> {
                serde_json::to_string(&self).unwrap().as_bytes().to_vec()
            }
            fn from_bytes(v: &[u8]) -> Self {
                serde_json::from_slice(v).unwrap()
            }
        })*
    };
}