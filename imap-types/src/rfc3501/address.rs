#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "serdex")]
use serde::{Deserialize, Serialize};

use crate::core::NString;

/// An address structure describes an electronic mail address.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address<'a> {
    /// Personal name
    pub(crate) name: NString<'a>,
    /// At-domain-list (source route)
    pub(crate) adl: NString<'a>,
    /// Mailbox name
    pub(crate) mailbox: NString<'a>,
    /// Host name
    pub(crate) host: NString<'a>,
}

impl<'a> Address<'a> {
    pub fn new(
        name: NString<'a>,
        adl: NString<'a>,
        mailbox: NString<'a>,
        host: NString<'a>,
    ) -> Address<'a> {
        Address {
            name,
            adl,
            mailbox,
            host,
        }
    }
}
