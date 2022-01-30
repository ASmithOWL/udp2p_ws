pub trait Record {
    type Key;
    fn get_key(&self) -> Self::Key;
}
