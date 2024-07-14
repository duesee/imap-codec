//! Mailbox-related types.

use std::{borrow::Cow, str::from_utf8};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    core::{impl_try_from, AString, IString},
    error::{ValidationError, ValidationErrorKind},
    mailbox::error::MailboxOtherError,
    utils::indicators::is_list_char,
};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct ListCharString<'a>(pub(crate) Cow<'a, str>);

impl<'a> ListCharString<'a> {
    pub fn validate(value: impl AsRef<[u8]>) -> Result<(), ValidationError> {
        let value = value.as_ref();

        if value.is_empty() {
            return Err(ValidationError::new(ValidationErrorKind::Empty));
        }

        if let Some(at) = value.iter().position(|b| !is_list_char(*b)) {
            return Err(ValidationError::new(ValidationErrorKind::InvalidByteAt {
                byte: value[at],
                at,
            }));
        };

        Ok(())
    }

    /// Constructs a list char string without validation.
    ///
    /// # Warning: IMAP conformance
    ///
    /// The caller must ensure that `inner` is valid according to [`Self::validate`]. Failing to do
    /// so may create invalid/unparsable IMAP messages, or even produce unintended protocol flows.
    /// Do not call this constructor with untrusted data.
    ///
    /// Note: This method will `panic!` on wrong input in debug builds.
    pub fn unvalidated<C>(inner: C) -> Self
    where
        C: Into<Cow<'a, str>>,
    {
        let inner = inner.into();

        #[cfg(debug_assertions)]
        Self::validate(inner.as_bytes()).unwrap();

        Self(inner)
    }
}

impl<'a> TryFrom<&'a str> for ListCharString<'a> {
    type Error = ValidationError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::validate(value)?;

        Ok(Self(Cow::Borrowed(value)))
    }
}

impl<'a> TryFrom<String> for ListCharString<'a> {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::validate(&value)?;

        Ok(Self(Cow::Owned(value)))
    }
}

impl<'a> AsRef<[u8]> for ListCharString<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum ListMailbox<'a> {
    Token(ListCharString<'a>),
    String(IString<'a>),
}

impl<'a> TryFrom<&'a str> for ListMailbox<'a> {
    type Error = ValidationError;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        if s.is_empty() {
            // Safety: We know that an empty string can always be converted into a quoted string.
            return Ok(ListMailbox::String(IString::Quoted(s.try_into().unwrap())));
        }

        if let Ok(lcs) = ListCharString::try_from(s) {
            return Ok(ListMailbox::Token(lcs));
        }

        Ok(ListMailbox::String(s.try_into()?))
    }
}

impl<'a> TryFrom<String> for ListMailbox<'a> {
    type Error = ValidationError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.is_empty() {
            // Safety: We know that an empty string can always be converted into a quoted string.
            return Ok(ListMailbox::String(IString::Quoted(s.try_into().unwrap())));
        }

        // TODO(efficiency)
        if let Ok(lcs) = ListCharString::try_from(s.clone()) {
            return Ok(ListMailbox::Token(lcs));
        }

        Ok(ListMailbox::String(s.try_into()?))
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum Mailbox<'a> {
    Inbox,
    Other(MailboxOther<'a>),
}

impl_try_from!(AString<'a>, 'a, &'a [u8], Mailbox<'a>);
impl_try_from!(AString<'a>, 'a, Vec<u8>, Mailbox<'a>);
impl_try_from!(AString<'a>, 'a, &'a str, Mailbox<'a>);
impl_try_from!(AString<'a>, 'a, String, Mailbox<'a>);

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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "AString<'a>"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct MailboxOther<'a>(pub(crate) AString<'a>);

impl<'a> MailboxOther<'a> {
    pub fn validate(value: impl AsRef<[u8]>) -> Result<(), MailboxOtherError> {
        if value.as_ref().to_ascii_lowercase() == b"inbox" {
            return Err(MailboxOtherError::Reserved);
        }

        Ok(())
    }

    pub fn inner(&self) -> &AString {
        &self.0
    }

    /// Constructs a mailbox without validation.
    ///
    /// # Warning: IMAP conformance
    ///
    /// The caller must ensure that `value` is valid according to [`Self::validate`]. Failing to do
    /// so may create invalid/unparsable IMAP messages, or even produce unintended protocol flows.
    /// Do not call this constructor with untrusted data.
    ///
    /// Note: This method will `panic!` on wrong input in debug builds.
    pub fn unvalidated(value: AString<'a>) -> Self {
        #[cfg(debug_assertions)]
        Self::validate(&value).unwrap();

        Self(value)
    }
}

macro_rules! impl_try_from_mailbox_other {
    ($from:ty) => {
        impl<'a> TryFrom<$from> for MailboxOther<'a> {
            type Error = MailboxOtherError;

            fn try_from(value: $from) -> Result<Self, Self::Error> {
                let astring = AString::try_from(value)?;

                Self::validate(&astring)?;

                Ok(Self(astring))
            }
        }
    };
}

impl_try_from_mailbox_other!(&'a [u8]);
impl_try_from_mailbox_other!(Vec<u8>);
impl_try_from_mailbox_other!(&'a str);
impl_try_from_mailbox_other!(String);

impl<'a> TryFrom<AString<'a>> for MailboxOther<'a> {
    type Error = MailboxOtherError;

    fn try_from(value: AString<'a>) -> Result<Self, Self::Error> {
        Self::validate(&value)?;

        Ok(Self(value))
    }
}

impl<'a> AsRef<[u8]> for MailboxOther<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/// Error-related types.
pub mod error {
    use thiserror::Error;

    use crate::error::ValidationError;

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum MailboxOtherError {
        #[error(transparent)]
        Literal(#[from] ValidationError),
        #[error("Reserved: Please use one of the typed variants")]
        Reserved,
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;
    #[cfg(feature = "serde")]
    use crate::core::AtomExt;
    use crate::core::{AString, IString, Literal, LiteralMode};

    #[test]
    fn test_conversion_mailbox() {
        let tests = [
            ("inbox", Mailbox::Inbox),
            ("inboX", Mailbox::Inbox),
            ("Inbox", Mailbox::Inbox),
            ("InboX", Mailbox::Inbox),
            ("INBOX", Mailbox::Inbox),
            (
                "INBO²",
                Mailbox::Other(MailboxOther(AString::String(IString::Literal(Literal {
                    data: Cow::Borrowed("INBO²".as_bytes()),
                    mode: LiteralMode::Sync,
                })))),
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
    fn test_conversion_mailbox_failing() {
        let tests = ["\x00", "A\x00", "\x00A"];

        for test in tests {
            assert!(Mailbox::try_from(test).is_err());
            assert!(Mailbox::try_from(String::from(test)).is_err());
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialization_list_char_string() {
        let valid_input = r#""OneWord""#;
        let invalid_input = r#""Two Words""#;

        let list_char_string = serde_json::from_str::<ListCharString>(valid_input)
            .expect("valid input should deserialize successfully");
        assert_eq!(list_char_string, ListCharString(Cow::Borrowed("OneWord")));

        let err = serde_json::from_str::<ListCharString>(invalid_input)
            .expect_err("invalid input should not deserialize successfully");
        assert_eq!(
            err.to_string(),
            r"Validation failed: Invalid byte b'\x20' at index 3"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialization_mailbox_other() {
        let valid_input = r#"{ "Atom": "other" }"#;
        let invalid_input = r#"{ "Atom": "inbox" }"#;

        let mailbox_other = serde_json::from_str::<MailboxOther>(valid_input)
            .expect("valid input should deserialize successfully");
        assert_eq!(
            mailbox_other,
            MailboxOther(AString::Atom(AtomExt(Cow::Borrowed("other"))))
        );

        let err = serde_json::from_str::<MailboxOther>(invalid_input)
            .expect_err("invalid input should not deserialize successfully");
        assert_eq!(
            err.to_string(),
            r"Reserved: Please use one of the typed variants"
        );
    }
}
