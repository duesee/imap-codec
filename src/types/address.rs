use crate::{codec::Codec, types::core::NString};

/// An address structure describes an electronic mail address.
#[derive(Debug, Clone, PartialEq)]
pub struct Address {
    /// Personal name
    name: NString,
    /// At-domain-list (source route)
    adl: NString,
    /// Mailbox name
    mailbox: NString,
    /// Host name
    host: NString,
}

impl Address {
    pub fn new(name: NString, adl: NString, mailbox: NString, host: NString) -> Address {
        Address {
            name,
            adl,
            mailbox,
            host,
        }
    }
}

impl Codec for Address {
    fn serialize(&self) -> Vec<u8> {
        let mut out = b"(".to_vec();
        out.extend(self.name.serialize());
        out.push(b' ');
        out.extend(self.adl.serialize());
        out.push(b' ');
        out.extend(self.mailbox.serialize());
        out.push(b' ');
        out.extend(self.host.serialize());
        out.push(b')');
        out
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
