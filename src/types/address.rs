use crate::{codec::Codec, types::core::NString};

/// An address structure is a parenthesized list that describes an
/// electronic mail address.  The fields of an address structure
/// are in the following order:
#[derive(Debug, Clone, PartialEq)]
pub struct Address {
    /// personal name,
    name: NString,
    /// [SMTP] at-domain-list (source route),
    adl: NString,
    /// mailbox name,
    mailbox: NString,
    /// and host name.
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

// impl std::fmt::Display for Address {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
//         write!(
//             f,
//             "({} {} {} {})",
//             self.name, self.adl, self.mailbox, self.host
//         )
//     }
// }
