//! Flag-related types.

use std::fmt::{Display, Formatter};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{core::Atom, error::ValidationError};

/// There are two types of flags in IMAP4rev1: System and keyword flags.
///
/// A system flag is a flag name that is pre-defined in RFC3501.
/// All system flags begin with "\\" and certain system flags (`\Deleted` and `\Seen`) have special semantics.
/// Flags that begin with "\\" but are not pre-defined system flags, are extension flags.
/// Clients MUST accept them and servers MUST NOT send them except when defined by future standard or standards-track revisions.
///
/// A keyword is defined by the server implementation.
/// Keywords do not begin with "\\" and servers may permit the client to define new ones
/// in the mailbox by sending the `\*` flag ([`FlagPerm::Asterisk`]) in the PERMANENTFLAGS response..
///
/// Note that a flag of either type can be permanent or session-only.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum Flag<'a> {
    /// Message has been answered (`\Answered`).
    Answered,
    /// Message is "deleted" for removal by later EXPUNGE (`\Deleted`).
    Deleted,
    /// Message has not completed composition (marked as a draft) (`\Draft`).
    Draft,
    /// Message is "flagged" for urgent/special attention (`\Flagged`).
    Flagged,
    /// Message has been read (`\Seen`).
    Seen,
    /// A future expansion of a system flag.
    Extension(FlagExtension<'a>),
    /// A keyword.
    Keyword(Atom<'a>),
}

/// An (extension) flag.
///
/// It's guaranteed that this type can't represent any flag from [`Flag`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct FlagExtension<'a>(Atom<'a>);

impl<'a> Flag<'a> {
    pub fn system(atom: Atom<'a>) -> Self {
        match atom.as_ref().to_ascii_lowercase().as_ref() {
            "answered" => Self::Answered,
            "deleted" => Self::Deleted,
            "draft" => Self::Draft,
            "flagged" => Self::Flagged,
            "seen" => Self::Seen,
            _ => Self::Extension(FlagExtension(atom)),
        }
    }

    pub fn keyword(atom: Atom<'a>) -> Self {
        Self::Keyword(atom)
    }
}

impl<'a> TryFrom<&'a str> for Flag<'a> {
    type Error = ValidationError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(if let Some(value) = value.strip_prefix('\\') {
            Self::system(Atom::try_from(value)?)
        } else {
            Self::keyword(Atom::try_from(value)?)
        })
    }
}

impl<'a> Display for Flag<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Flag::Answered => f.write_str("\\Answered"),
            Flag::Deleted => f.write_str("\\Deleted"),
            Flag::Draft => f.write_str("\\Draft"),
            Flag::Flagged => f.write_str("\\Flagged"),
            Flag::Seen => f.write_str("\\Seen"),
            Flag::Extension(other) => write!(f, "\\{}", other.0),
            Flag::Keyword(atom) => write!(f, "{}", atom),
        }
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum FlagFetch<'a> {
    Flag(Flag<'a>),

    /// Message is "recently" arrived in this mailbox. (`\Recent`)
    ///
    /// This session is the first session to have been notified about this message; if the session
    /// is read-write, subsequent sessions will not see \Recent set for this message.
    ///
    /// Note: This flag can not be altered by the client.
    Recent,
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum FlagPerm<'a> {
    Flag(Flag<'a>),

    /// Indicates that it is possible to create new keywords by
    /// attempting to store those flags in the mailbox (`\*`).
    Asterisk,
}

/// Four name attributes are defined.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum FlagNameAttribute<'a> {
    /// It is not possible for any child levels of hierarchy to exist
    /// under this name; no child levels exist now and none can be
    /// created in the future. (`\Noinferiors`)
    Noinferiors,

    /// It is not possible to use this name as a selectable mailbox. (`\Noselect`)
    Noselect,

    /// The mailbox has been marked "interesting" by the server; the
    /// mailbox probably contains messages that have been added since
    /// the last time the mailbox was selected. (`\Marked`)
    Marked,

    /// The mailbox does not contain any additional messages since the
    /// last time the mailbox was selected. (`\Unmarked`)
    Unmarked,

    /// An extension flags.
    Extension(FlagNameAttributeExtension<'a>),
}

/// An extension flag.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct FlagNameAttributeExtension<'a>(Atom<'a>);

impl<'a> FlagNameAttribute<'a> {
    pub fn is_selectability(&self) -> bool {
        matches!(
            self,
            FlagNameAttribute::Noselect | FlagNameAttribute::Marked | FlagNameAttribute::Unmarked
        )
    }
}

impl<'a> From<Atom<'a>> for FlagNameAttribute<'a> {
    fn from(atom: Atom<'a>) -> Self {
        match atom.as_ref().to_ascii_lowercase().as_ref() {
            "noinferiors" => Self::Noinferiors,
            "noselect" => Self::Noselect,
            "marked" => Self::Marked,
            "unmarked" => Self::Unmarked,
            _ => Self::Extension(FlagNameAttributeExtension(atom)),
        }
    }
}

impl<'a> Display for FlagNameAttribute<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::Noinferiors => f.write_str("\\Noinferiors"),
            Self::Noselect => f.write_str("\\Noselect"),
            Self::Marked => f.write_str("\\Marked"),
            Self::Unmarked => f.write_str("\\Unmarked"),
            Self::Extension(extension) => write!(f, "\\{}", extension.0),
        }
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToStatic)]
pub enum StoreType {
    Replace,
    Add,
    Remove,
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToStatic)]
pub enum StoreResponse {
    Answer,
    Silent,
}
