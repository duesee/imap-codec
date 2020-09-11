use std::io::Write;

pub trait Serialize {
    fn serialize(&self, writer: &mut impl Write) -> std::io::Result<()>;
}
