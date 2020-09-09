use crate::{codec::Encoder, types::core::NString};

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

impl Encoder for Address {
    fn encode(&self) -> Vec<u8> {
        let mut out = b"(".to_vec();
        out.extend(self.name.encode());
        out.push(b' ');
        out.extend(self.adl.encode());
        out.push(b' ');
        out.extend(self.mailbox.encode());
        out.push(b' ');
        out.extend(self.host.encode());
        out.push(b')');
        out
    }
}
