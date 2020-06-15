pub trait Codec {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized;
}
