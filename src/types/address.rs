#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "serdex")]
use serde::{Deserialize, Serialize};

use crate::types::core::NString;

/// An address structure describes an electronic mail address.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address {
    /// Personal name
    pub(crate) name: NString,
    /// At-domain-list (source route)
    pub(crate) adl: NString,
    /// Mailbox name
    pub(crate) mailbox: NString,
    /// Host name
    pub(crate) host: NString,
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
