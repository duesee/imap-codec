// ### 2.3.2. Flags Message Attribute

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{core::Atom, imap4rev1::core::AtomError};

/// There are two types of flags in IMAP4rev1: System and keyword flags.
///
/// A system flag is a flag name that is pre-defined in RFC3501.
/// All system flags begin with "\" and certain system flags (`\Deleted` and `\Seen`) have special semantics.
/// Flags that begin with "\" but are not pre-defined system flags, are extension flags.
/// Clients MUST accept them and servers MUST NOT send them except when defined by future standard or standards-track revisions.
///
/// A keyword is defined by the server implementation.
/// Keywords do not begin with "\" and servers may permit the client to define new ones
/// in the mailbox by sending the "\*" flag ([`FlagPerm::AllowNewKeyword`]) in the PERMANENTFLAGS response..
///
/// Note that a flag of either type can be permanent or session-only.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Flag<'a> {
    /// Message has been read (`\Seen`).
    Seen,
    /// Message has been answered (`\Answered`).
    Answered,
    /// Message is "flagged" for urgent/special attention (`\Flagged`).
    Flagged,
    /// Message is "deleted" for removal by later EXPUNGE (`\Deleted`).
    Deleted,
    /// Message has not completed composition (marked as a draft) (`\Draft`).
    Draft,
    /// A future expansion of a system flag.
    Extension(FlagExtension<'a>),
    /// A keyword.
    Keyword(Atom<'a>),
}

impl<'a> Flag<'a> {
    pub fn system<A>(value: A) -> Result<Self, FlagError<'a, A::Error>>
    where
        A: TryInto<Atom<'a>>,
    {
        let atom = value.try_into()?;

        match atom.as_ref().to_ascii_lowercase().as_ref() {
            "answered" => Ok(Self::Answered),
            "flagged" => Ok(Self::Flagged),
            "deleted" => Ok(Self::Deleted),
            "seen" => Ok(Self::Seen),
            "draft" => Ok(Self::Draft),
            _ => Err(FlagError::IsAnExtensionFlag { candidate: atom }),
        }
    }

    pub fn extension<A>(value: A) -> Result<Self, FlagError<'a, A::Error>>
    where
        A: TryInto<Atom<'a>>,
    {
        let atom = value.try_into()?;

        match atom.as_ref().to_ascii_lowercase().as_ref() {
            "answered" | "flagged" | "deleted" | "seen" | "draft" => {
                Err(FlagError::IsASystemFlag { candidate: atom })
            }
            _ => Ok(Self::Extension(FlagExtension(atom))),
        }
    }

    pub fn system_or_extension(atom: Atom<'a>) -> Self {
        match Self::system(atom) {
            Ok(system) => system,
            Err(FlagError::Atom(_)) => unreachable!(),
            Err(FlagError::IsASystemFlag { .. }) => unreachable!(),
            Err(FlagError::IsAnExtensionFlag { candidate }) => {
                Self::Extension(FlagExtension(candidate))
            }
        }
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FlagPerm<'a> {
    Flag(Flag<'a>),

    /// Indicates that it is possible to create new keywords by
    /// attempting to store those flags in the mailbox (`\*`).
    AllowNewKeywords,
}

/// Client implementations MUST accept flag-extension flags.
/// Server implementations MUST NOT generate flag-extension flags
/// except as defined by future standard or standards-track revisions of this specification.
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlagExtension<'a>(pub(crate) Atom<'a>);

impl<'a> TryFrom<Atom<'a>> for FlagExtension<'a> {
    type Error = FlagError<'a, AtomError>;

    fn try_from(value: Atom<'a>) -> Result<Self, Self::Error> {
        match value.as_ref().to_ascii_lowercase().as_ref() {
            "answered" | "flagged" | "deleted" | "seen" | "draft" => {
                Err(FlagError::IsASystemFlag { candidate: value })
            }
            _ => Ok(Self(value)),
        }
    }
}

impl<'a> AsRef<str> for FlagExtension<'a> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum FlagError<'a, A> {
    #[error(transparent)]
    Atom(#[from] A),
    #[error("Is a system flag.")]
    IsASystemFlag { candidate: Atom<'a> },
    #[error("Is an extension flag.")]
    IsAnExtensionFlag { candidate: Atom<'a> },
}

/// Four name attributes are defined.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    /// Note: extension flags must also be accepted here...
    Extension(Atom<'a>),
}

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
            _ => Self::Extension(atom),
        }
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StoreType {
    Replace,
    Add,
    Remove,
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StoreResponse {
    Answer,
    Silent,
}
