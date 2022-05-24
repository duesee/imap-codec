use std::{
    convert::{TryFrom, TryInto},
    ops::Deref,
    str::from_utf8,
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "serdex")]
use serde::{Deserialize, Serialize};

use crate::{
    core::{AString, IString},
    utils::indicators::is_list_char,
};

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ListCharString(String);

impl TryFrom<&str> for ListCharString {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.to_string().try_into()
    }
}

impl TryFrom<String> for ListCharString {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value.is_empty() && value.bytes().all(is_list_char) {
            Ok(ListCharString(value))
        } else {
            Err(())
        }
    }
}

impl Deref for ListCharString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ListMailbox<'a> {
    Token(ListCharString),
    String(IString<'a>),
}

impl<'a> TryFrom<&'a str> for ListMailbox<'a> {
    type Error = ();

    fn try_from(s: &'a str) -> Result<Self, ()> {
        Ok(if s.is_empty() {
            ListMailbox::String(IString::Quoted(s.try_into().map_err(|_| ())?))
        } else if let Ok(lcs) = ListCharString::try_from(s.clone()) {
            ListMailbox::Token(lcs)
        } else {
            ListMailbox::String(s.try_into().map_err(|_| ())?)
        })
    }
}

impl<'a> TryFrom<String> for ListMailbox<'a> {
    type Error = ();

    fn try_from(s: String) -> Result<Self, ()> {
        Ok(if s.is_empty() {
            ListMailbox::String(IString::Quoted(s.try_into().map_err(|_| ())?))
        } else if let Ok(lcs) = ListCharString::try_from(s.clone()) {
            ListMailbox::Token(lcs)
        } else {
            ListMailbox::String(s.try_into().map_err(|_| ())?)
        })
    }
}

/// 5.1. Mailbox Naming
///
/// Mailbox names are 7-bit.  Client implementations MUST NOT attempt to
/// create 8-bit mailbox names, and SHOULD interpret any 8-bit mailbox
/// names returned by LIST or LSUB as UTF-8.  Server implementations
/// SHOULD prohibit the creation of 8-bit mailbox names, and SHOULD NOT
/// return 8-bit mailbox names in LIST or LSUB.  See section 5.1.3 for
/// more information on how to represent non-ASCII mailbox names.
///
/// Note: 8-bit mailbox names were undefined in earlier
/// versions of this protocol.  Some sites used a local 8-bit
/// character set to represent non-ASCII mailbox names.  Such
/// usage is not interoperable, and is now formally deprecated.
///
/// The case-insensitive mailbox name INBOX is a special name reserved to
/// mean "the primary mailbox for this user on this server".  The
/// interpretation of all other names is implementation-dependent.
///
/// In particular, this specification takes no position on case
/// sensitivity in non-INBOX mailbox names.  Some server implementations
/// are fully case-sensitive; others preserve case of a newly-created
/// name but otherwise are case-insensitive; and yet others coerce names
/// to a particular case.  Client implementations MUST interact with any
/// of these.  If a server implementation interprets non-INBOX mailbox
/// names as case-insensitive, it MUST treat names using the
/// international naming convention specially as described in section 5.1.3.
///
/// There are certain client considerations when creating a new mailbox name:
///
/// 1) Any character which is one of the atom-specials (see the Formal Syntax) will require
///    that the mailbox name be represented as a quoted string or literal.
/// 2) CTL and other non-graphic characters are difficult to represent in a user interface
///    and are best avoided.
/// 3) Although the list-wildcard characters ("%" and "*") are valid in a mailbox name, it is
///    difficult to use such mailbox names with the LIST and LSUB commands due to the conflict
///    with wildcard interpretation.
/// 4) Usually, a character (determined by the server implementation) is reserved to delimit
///    levels of hierarchy.
/// 5) Two characters, "#" and "&", have meanings by convention, and should be avoided except
///    when used in that convention.
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Mailbox<'a> {
    Inbox,
    Other(MailboxOther<'a>),
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MailboxOther<'a> {
    pub(crate) inner: AString<'a>,
}

impl<'a> TryFrom<AString<'a>> for MailboxOther<'a> {
    type Error = ();

    fn try_from(mailbox: AString<'a>) -> Result<Self, Self::Error> {
        match mailbox {
            AString::Atom(ref str) => {
                if str.to_lowercase() == "inbox" {
                    Err(())
                } else {
                    Ok(MailboxOther { inner: mailbox })
                }
            }
            AString::String(ref imap_str) => match imap_str {
                IString::Quoted(ref str) => {
                    if str.to_lowercase() == "inbox" {
                        Err(())
                    } else {
                        Ok(MailboxOther { inner: mailbox })
                    }
                }
                IString::Literal(bytes) => {
                    // "INBOX" (in any case) is certainly valid ASCII/UTF-8...
                    if let Ok(str) = from_utf8(bytes) {
                        // After the conversion we ignore the case...
                        if str.to_lowercase() == "inbox" {
                            // ...and return the Inbox variant.
                            Err(())
                        } else {
                            Ok(MailboxOther { inner: mailbox })
                        }
                    } else {
                        // ... If not, it must be something else.
                        Ok(MailboxOther { inner: mailbox })
                    }
                }
            },
        }
    }
}

impl<'a> TryFrom<&'a str> for Mailbox<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, ()> {
        if value.to_lowercase() == "inbox" {
            Ok(Mailbox::Inbox)
        } else {
            let astr = AString::try_from(value)?;
            let other = MailboxOther::try_from(astr)?;

            Ok(Mailbox::Other(other))
        }
    }
}

impl<'a> TryFrom<String> for Mailbox<'a> {
    type Error = ();

    fn try_from(s: String) -> Result<Self, ()> {
        if s.to_lowercase() == "inbox" {
            Ok(Mailbox::Inbox)
        } else {
            let astr = AString::try_from(s)?;
            let other = MailboxOther::try_from(astr)?;

            Ok(Mailbox::Other(other))
        }
    }
}
