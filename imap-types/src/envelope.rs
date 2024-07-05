//! Envelope-related types.

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::core::NString;

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct Envelope<'a> {
    pub date: NString<'a>,
    pub subject: NString<'a>,
    pub from: Vec<Address<'a>>,
    pub sender: Vec<Address<'a>>,
    pub reply_to: Vec<Address<'a>>,
    pub to: Vec<Address<'a>>,
    pub cc: Vec<Address<'a>>,
    pub bcc: Vec<Address<'a>>,
    pub in_reply_to: NString<'a>,
    pub message_id: NString<'a>,
}

/// An address structure describes an electronic mail address.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
/// TODO(misuse):
///
///   Here are many invariants ...
///
///   mailbox:
///     NIL indicates end of [RFC-2822] group;
///     if non-NIL and host is NIL, holds [RFC-2822] group name.
///     Otherwise, holds [RFC-2822] local-part after removing [RFC-2822] quoting
///
///   host:
///     NIL indicates [RFC-2822] group syntax.
///     Otherwise, holds [RFC-2822] domain name
pub struct Address<'a> {
    /// Personal name
    pub name: NString<'a>,
    /// At-domain-list (source route)
    pub adl: NString<'a>,
    /// Mailbox name
    pub mailbox: NString<'a>,
    /// Host name
    pub host: NString<'a>,
}
