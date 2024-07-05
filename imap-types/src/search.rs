//! Search-related types.

use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    core::{AString, Atom, Vec1},
    datetime::NaiveDate,
    sequence::SequenceSet,
};

/// The defined search keys.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum SearchKey<'a> {
    // <Not in RFC.>
    //
    // IMAP doesn't have a dedicated AND operator in its search syntax.
    // ANDing multiple search keys works by concatenating them with an ascii space.
    // Introducing this variant makes sense, because
    //   * it may help in understanding the RFC
    //   * and it can be used to distinguish between a single search key
    //     and multiple search keys.
    //
    // See also the corresponding `search` parser.
    And(Vec1<SearchKey<'a>>),

    /// Messages with message sequence numbers corresponding to the
    /// specified message sequence number set.
    SequenceSet(SequenceSet),

    /// All messages in the mailbox; the default initial key for ANDing.
    All,

    /// Messages with the \Answered flag set.
    Answered,

    /// Messages that contain the specified string in the envelope
    /// structure's BCC field.
    Bcc(AString<'a>),

    /// Messages whose internal date (disregarding time and timezone)
    /// is earlier than the specified date.
    Before(NaiveDate),

    /// Messages that contain the specified string in the body of the
    /// message.
    Body(AString<'a>),

    /// Messages that contain the specified string in the envelope
    /// structure's CC field.
    Cc(AString<'a>),

    /// Messages with the \Deleted flag set.
    Deleted,

    /// Messages with the \Draft flag set.
    Draft,

    /// Messages with the \Flagged flag set.
    Flagged,

    /// Messages that contain the specified string in the envelope
    /// structure's FROM field.
    From(AString<'a>),

    /// Messages that have a header with the specified field-name (as
    /// defined in [RFC-2822]) and that contains the specified string
    /// in the text of the header (what comes after the colon).  If the
    /// string to search is zero-length, this matches all messages that
    /// have a header line with the specified field-name regardless of
    /// the contents.
    Header(AString<'a>, AString<'a>),

    /// Messages with the specified keyword flag set.
    Keyword(Atom<'a>),

    /// Messages with an [RFC-2822] size larger than the specified
    /// number of octets.
    Larger(u32),

    /// Messages that have the \Recent flag set but not the \Seen flag.
    /// This is functionally equivalent to "(RECENT UNSEEN)".
    New,

    /// Messages that do not match the specified search key.
    Not(Box<SearchKey<'a>>),

    /// Messages that do not have the \Recent flag set.  This is
    /// functionally equivalent to "NOT RECENT" (as opposed to "NOT
    /// NEW").
    Old,

    /// Messages whose internal date (disregarding time and timezone)
    /// is within the specified date.
    On(NaiveDate),

    /// Messages that match either search key.
    Or(Box<SearchKey<'a>>, Box<SearchKey<'a>>),

    /// Messages that have the \Recent flag set.
    Recent,

    /// Messages that have the \Seen flag set.
    Seen,

    /// Messages whose [RFC-2822] Date: header (disregarding time and
    /// timezone) is earlier than the specified date.
    SentBefore(NaiveDate),

    /// Messages whose [RFC-2822] Date: header (disregarding time and
    /// timezone) is within the specified date.
    SentOn(NaiveDate),

    /// Messages whose [RFC-2822] Date: header (disregarding time and
    /// timezone) is within or later than the specified date.
    SentSince(NaiveDate),

    /// Messages whose internal date (disregarding time and timezone)
    /// is within or later than the specified date.
    Since(NaiveDate),

    /// Messages with an [RFC-2822] size smaller than the specified
    /// number of octets.
    Smaller(u32),

    /// Messages that contain the specified string in the envelope
    /// structure's SUBJECT field.
    Subject(AString<'a>),

    /// Messages that contain the specified string in the header or
    /// body of the message.
    Text(AString<'a>),

    /// Messages that contain the specified string in the envelope
    /// structure's TO field.
    To(AString<'a>),

    /// Messages with unique identifiers corresponding to the specified
    /// unique identifier set.  Sequence set ranges are permitted.
    Uid(SequenceSet),

    /// Messages that do not have the \Answered flag set.
    Unanswered,

    /// Messages that do not have the \Deleted flag set.
    Undeleted,

    /// Messages that do not have the \Draft flag set.
    Undraft,

    /// Messages that do not have the \Flagged flag set.
    Unflagged,

    /// Messages that do not have the specified keyword flag set.
    Unkeyword(Atom<'a>),

    /// Messages that do not have the \Seen flag set.
    Unseen,
}

impl<'a> SearchKey<'a> {
    pub fn uid<S>(sequence_set: S) -> Self
    where
        S: Into<SequenceSet>,
    {
        Self::Uid(sequence_set.into())
    }
}
