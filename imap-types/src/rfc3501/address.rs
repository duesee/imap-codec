#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "serdex")]
use serde::{Deserialize, Serialize};

use crate::core::NString;

/// An address structure describes an electronic mail address.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
