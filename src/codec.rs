use std::io::Write;

pub trait Encode {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()>;
}
