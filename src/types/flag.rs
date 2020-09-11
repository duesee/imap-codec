// ### 2.3.2. Flags Message Attribute

use crate::{codec::Serialize, types::core::Atom};
use serde::Deserialize;
use std::io::Write;

/// A list of zero or more named tokens associated with the message.  A
/// flag is set by its addition to this list, and is cleared by its
/// removal.  There are two types of flags in IMAP4rev1. A flag of either
/// type can be permanent or session-only.
/// FIXME: this struct is not very usable currently...
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Flag {
    // ----- System -----
    //
    // A system flag is a flag name that is pre-defined in this
    // specification.  All system flags begin with "\".  Certain system
    // flags (\Deleted and \Seen) have special semantics described elsewhere.
    /// Message has been read (`\Seen`)
    Seen,
    /// Message has been answered (`\Answered`)
    Answered,
    /// Message is "flagged" for urgent/special attention (`\Flagged`)
    Flagged,
    /// Message is "deleted" for removal by later EXPUNGE (`\Deleted`)
    Deleted,
    /// Message has not completed composition (marked as a draft). (`\Draft`)
    Draft,

    // ----- Fetch -----
    /// Message is "recently" arrived in this mailbox. (`\Recent`)
    ///
    /// This session is the first session to have been notified about this
    /// message; if the session is read-write, subsequent sessions
    /// will not see \Recent set for this message.  This flag can not
    /// be altered by the client.
    Recent,

    // ----- Selectability -----
    NameAttribute(FlagNameAttribute),

    // ----- Keyword -----
    /// Indicates that it is possible to create new keywords by
    /// attempting to store those flags in the mailbox. (`\*`)
    Permanent,
    /// A keyword is defined by the server implementation.  Keywords do not
    /// begin with "\".  Servers MAY permit the client to define new keywords
    /// in the mailbox (see the description of the PERMANENTFLAGS response
    /// code for more information).
    Keyword(Atom),

    // ----- Others -----
    Extension(Atom),
}

impl std::fmt::Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            // ----- System -----
            Flag::Seen => write!(f, "\\Seen"),
            Flag::Answered => write!(f, "\\Answered"),
            Flag::Flagged => write!(f, "\\Flagged"),
            Flag::Deleted => write!(f, "\\Deleted"),
            Flag::Draft => write!(f, "\\Draft"),

            // ----- Fetch -----
            Flag::Recent => write!(f, "\\Recent"),

            // ----- Selectability -----
            Flag::NameAttribute(flag) => write!(f, "{}", flag),

            // ----- Keyword -----
            Flag::Permanent => write!(f, "\\*"),
            Flag::Keyword(atom) => write!(f, "{}", atom),

            // ----- Others -----
            Flag::Extension(atom) => write!(f, "\\{}", atom),
        }
    }
}

impl Serialize for Flag {
    fn serialize(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "{}", self)
    }
}

/// Four name attributes are defined.
#[derive(Eq, Debug, Clone, PartialEq, Deserialize)]
pub enum FlagNameAttribute {
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
    Extension(Atom),
}

impl FlagNameAttribute {
    pub fn is_selectability(&self) -> bool {
        matches!(
            self,
            FlagNameAttribute::Noselect | FlagNameAttribute::Marked | FlagNameAttribute::Unmarked
        )
    }
}

impl std::fmt::Display for FlagNameAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Noinferiors => write!(f, "\\Noinferiors"),
            Self::Noselect => write!(f, "\\Noselect"),
            Self::Marked => write!(f, "\\Marked"),
            Self::Unmarked => write!(f, "\\Unmarked"),
            Self::Extension(atom) => write!(f, "\\{}", atom),
        }
    }
}

impl Serialize for FlagNameAttribute {
    fn serialize(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "{}", self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StoreType {
    Replace,
    Add,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StoreResponse {
    Answer,
    Silent,
}
