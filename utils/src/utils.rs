use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

pub type Timestamp = u128;
/// A trait for measuring XOR distance
/// TODO: add a binary function and a prefix function
pub trait Distance {
    type Output;

    fn xor(&self, other: Self::Output) -> Self::Output;
    fn leading_zeros(&self) -> usize;
    fn leading_ones(&self) -> usize {0}
}

/// A trait for converting a type that implements Serialize + Deserialize
/// to a vector of bytes and from an array of bytes back into the type.
pub trait ByteRep<'a>: Serialize + Deserialize<'a> {
    fn as_bytes(&self) -> Vec<u8>;
    fn from_bytes(v: &[u8]) -> Self;
}

/// Get the current unix timestamp in nanoseconds
pub fn timestamp_now() -> Timestamp {
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