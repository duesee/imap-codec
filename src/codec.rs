pub trait Codec {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized;
}

pub(crate) fn escape(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| match byte {
            0x00..=0x08 => format!("\\x{:02x}", byte),
            0x09 => String::from("\\t"),
            0x0A => String::from("\\n"),
            0x0B => format!("\\x{:02x}", byte),
            0x0C => format!("\\x{:02x}", byte),
            0x0D => String::from("\\r"),
            0x0e..=0x1f => format!("\\x{:02x}", byte),
            0x20..=0x22 => format!("{}", *byte as char),
            0x23..=0x5B => format!("{}", *byte as char),
            0x5C => String::from("\\\\"),
            0x5D..=0x7E => format!("{}", *byte as char),
            0x7f => format!("\\x{:02x}", byte),
            0x80..=0xff => format!("\\x{:02x}", byte),
        })
        .collect::<Vec<String>>()
        .join("")
}
