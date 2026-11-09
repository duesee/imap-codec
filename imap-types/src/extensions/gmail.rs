//! Gmail's IMAP extensions (`X-GM-EXT-1`).
//!
//! See <https://developers.google.com/workspace/gmail/imap/imap-extensions>.
//! Gmail identifies every message with a 64-bit `X-GM-MSGID` that is the same
//! in every folder (label) it appears in, groups messages into threads by
//! `X-GM-THRID`, and reports the message's labels as `X-GM-LABELS`.

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::core::{AString, Atom};

/// One entry of `X-GM-LABELS`.
///
/// Gmail writes system labels as a backslash and an atom (`\Inbox`,
/// `\Important`, `\Sent`, ...) and user labels as astrings in modified
/// UTF-7. Gmail also sends system labels quoted (`"\\Important"`); those
/// parse as [`GmailLabel::Name`] to keep the codec lossless, and
/// [`GmailLabel::system_name`] recognizes both spellings.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum GmailLabel<'a> {
    /// `\` followed by an atom, e.g. `\Inbox`.
    System(Atom<'a>),
    /// A label name, e.g. `Receipts` or `"Muy importante"`.
    Name(AString<'a>),
}

impl GmailLabel<'_> {
    /// The system label's name without its backslash (`Inbox`), whether it
    /// arrived bare or quoted. `None` for user labels.
    pub fn system_name(&self) -> Option<&str> {
        match self {
            Self::System(atom) => Some(atom.inner()),
            Self::Name(name) => core::str::from_utf8(name.as_ref())
                .ok()
                .and_then(|name| name.strip_prefix('\\'))
                .filter(|rest| Atom::validate(rest).is_ok()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_name() {
        let bare = GmailLabel::System(Atom::try_from("Inbox").unwrap());
        let quoted = GmailLabel::Name(AString::try_from("\\Important").unwrap());
        let user = GmailLabel::Name(AString::try_from("Receipts").unwrap());

        assert_eq!(bare.system_name(), Some("Inbox"));
        assert_eq!(quoted.system_name(), Some("Important"));
        assert_eq!(user.system_name(), None);
    }
}
