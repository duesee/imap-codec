use crate::{
    codec::Encoder,
    parse::mailbox::is_list_char,
    types::core::{AString, IString},
};
use serde::Deserialize;
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq)]
pub enum ListMailbox {
    Token(String),
    String(IString),
}

impl Encoder for ListMailbox {
    fn encode(&self) -> Vec<u8> {
        match self {
            ListMailbox::Token(str) => str.clone().into_bytes(),
            ListMailbox::String(imap_str) => imap_str.encode(),
        }
    }
}

impl From<&str> for ListMailbox {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

impl From<String> for ListMailbox {
    fn from(s: String) -> Self {
        if s.is_empty() {
            ListMailbox::String(IString::Quoted(s))
        } else if s.chars().all(|c| c.is_ascii() && is_list_char(c as u8)) {
            ListMailbox::Token(s)
        } else {
            ListMailbox::String(s.into())
        }
    }
}

impl TryFrom<Mailbox> for String {
    type Error = std::string::FromUtf8Error;

    fn try_from(value: Mailbox) -> Result<Self, Self::Error> {
        match value {
            Mailbox::Inbox => Ok("INBOX".to_string()),
            Mailbox::Other(astring) => String::try_from(astring),
        }
    }
}

impl TryFrom<ListMailbox> for String {
    type Error = std::string::FromUtf8Error;

    fn try_from(value: ListMailbox) -> Result<Self, Self::Error> {
        match value {
            ListMailbox::Token(string) => Ok(string),
            ListMailbox::String(istring) => String::try_from(istring),
        }
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Mailbox {
    Inbox,
    // FIXME: prevent `Mailbox::Other("Inbox")`?
    Other(AString),
}

impl Encoder for Mailbox {
    fn encode(&self) -> Vec<u8> {
        match self {
            Mailbox::Inbox => b"INBOX".to_vec(),
            Mailbox::Other(a_str) => a_str.encode(),
        }
    }
}

impl From<&str> for Mailbox {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

impl From<String> for Mailbox {
    fn from(s: String) -> Self {
        if s.to_lowercase() == "inbox" {
            Mailbox::Inbox
        } else {
            Mailbox::Other(s.into())
        }
    }
}
