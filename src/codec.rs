pub trait Codec {
    fn serialize(&self) -> Vec<u8>;
}
