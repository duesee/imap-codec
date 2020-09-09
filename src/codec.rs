pub trait Encoder {
    fn encode(&self) -> Vec<u8>;
}
