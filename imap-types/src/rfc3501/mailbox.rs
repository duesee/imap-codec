use std::{
    borrow::Cow,
    convert::{TryFrom, TryInto},
    str::from_utf8,
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    core::{AString, IString},
    rfc3501::core::{impl_try_from, impl_try_from_try_from},
    utils::indicators::is_list_char,
};

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ListCharString<'a>(pub(crate) Cow<'a, str>);

impl<'a> ListCharString<'a> {
    pub fn verify(value: &str) -> bool {
        !value.is_empty() && value.bytes().all(is_list_char)
    }

    #[cfg(feature = "unchecked")]
    pub fn new_unchecked(inner: Cow<'a, str>) -> Self {
        #[cfg(debug_assertions)]
        assert!(Self::verify(&inner));

        Self(inner)
    }
}

impl<'a> TryFrom<&'a str> for ListCharString<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if Self::verify(value) {
            Ok(Self(Cow::Borrowed(value)))
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<String> for ListCharString<'a> {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if Self::verify(&value) {
            Ok(Self(Cow::Owned(value)))
        } else {
            Err(())
        }
    }
}

impl<'a> AsRef<[u8]> for ListCharString<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ListMailbox<'a> {
    Token(ListCharString<'a>),
    String(IString<'a>),
}

impl<'a> TryFrom<&'a str> for ListMailbox<'a> {
    type Error = ();

    fn try_from(s: &'a str) -> Result<Self, ()> {
        Ok(if s.is_empty() {
            ListMailbox::String(IString::Quoted(s.try_into()?))
        } else if let Ok(lcs) = ListCharString::try_from(s) {
            ListMailbox::Token(lcs)
        } else {
            ListMailbox::String(s.try_into()?)
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
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Mailbox<'a> {
    Inbox,
    Other(MailboxOther<'a>),
}

impl_try_from!(AString, 'a, &'a [u8], Mailbox<'a>);
impl_try_from!(AString, 'a, Vec<u8>, Mailbox<'a>);
impl_try_from!(AString, 'a, &'a str, Mailbox<'a>);
impl_try_from!(AString, 'a, String, Mailbox<'a>);

impl<'a> From<AString<'a>> for Mailbox<'a> {
    fn from(value: AString<'a>) -> Self {
        match from_utf8(value.as_ref()) {
            Ok(value) if value.to_ascii_lowercase() == "inbox" => Self::Inbox,
            _ => Self::Other(MailboxOther::try_from(value).unwrap()),
        }
    }
}

// We do not implement `AsRef<...>` for `Mailbox` because we want to enforce that a consumer
// `match`es on `Mailbox::Inbox`/`Mailbox::Other`.

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MailboxOther<'a>(pub(crate) AString<'a>);

impl_try_from_try_from!(AString, 'a, &'a [u8], MailboxOther<'a>);
impl_try_from_try_from!(AString, 'a, Vec<u8>, MailboxOther<'a>);
impl_try_from_try_from!(AString, 'a, &'a str, MailboxOther<'a>);
impl_try_from_try_from!(AString, 'a, String, MailboxOther<'a>);

impl<'a> TryFrom<AString<'a>> for MailboxOther<'a> {
    type Error = ();

    fn try_from(mailbox: AString<'a>) -> Result<Self, Self::Error> {
        if let Ok(str) = from_utf8(mailbox.as_ref()) {
            if str.to_ascii_lowercase() == "inbox" {
                return Err(());
            }
        }

        Ok(MailboxOther(mailbox))
    }
}

impl<'a> AsRef<[u8]> for MailboxOther<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, convert::TryFrom};

    use crate::{
        core::{AString, IString, Literal},
        message::{Mailbox, MailboxOther},
    };

    #[test]
    fn mailbox_try_from_str_string_positive() {
        let tests = [
            ("inbox", Mailbox::Inbox),
            ("inboX", Mailbox::Inbox),
            ("Inbox", Mailbox::Inbox),
            ("InboX", Mailbox::Inbox),
            ("INBOX", Mailbox::Inbox),
            (
                "INBO²",
                Mailbox::Other(MailboxOther(AString::String(IString::Literal(Literal(
                    Cow::Borrowed("INBO²".as_bytes()),
                ))))),
            ),
        ];

        for (test, expected) in tests {
            let got = Mailbox::try_from(test).unwrap();
            assert_eq!(expected, got);

            let got = Mailbox::try_from(String::from(test)).unwrap();
            assert_eq!(expected, got);
        }
    }

    #[test]
    fn mailbox_try_from_str_string_negative() {
        let tests = ["\x00", "A\x00", "\x00A"];

        for test in tests {
            assert!(Mailbox::try_from(test).is_err());
            assert!(Mailbox::try_from(String::from(test)).is_err());
        }
    }
}
