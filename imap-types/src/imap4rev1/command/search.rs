#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    command::SequenceSet,
    core::{AString, Atom, NonEmptyVec},
    message::NaiveDate,
};

/// The defined search keys.
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    And(NonEmptyVec<SearchKey<'a>>),

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
    Not(Box<SearchKey<'a>>), // TODO(misuse): is this a Vec or a single SearchKey?

    /// Messages that do not have the \Recent flag set.  This is
    /// functionally equivalent to "NOT RECENT" (as opposed to "NOT
    /// NEW").
    Old,

    /// Messages whose internal date (disregarding time and timezone)
    /// is within the specified date.
    On(NaiveDate),

    /// Messages that match either search key.
    Or(Box<SearchKey<'a>>, Box<SearchKey<'a>>), /* TODO(misuse): is this a Vec or a single SearchKey? */

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

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::*;
    use crate::{command::Sequence, testing::known_answer_test_encode};

    #[test]
    fn test_encode_search_key() {
        let tests = [
            (
                SearchKey::And(
                    NonEmptyVec::try_from(vec![SearchKey::Answered, SearchKey::Seen]).unwrap(),
                ),
                b"(ANSWERED SEEN)".as_ref(),
            ),
            (
                SearchKey::SequenceSet(SequenceSet::try_from(1).unwrap()),
                b"1",
            ),
            (SearchKey::All, b"ALL"),
            (SearchKey::Answered, b"ANSWERED"),
            (SearchKey::Bcc(AString::try_from("A").unwrap()), b"BCC A"),
            (
                SearchKey::Before(NaiveDate(
                    chrono::NaiveDate::from_ymd_opt(2023, 04, 12).unwrap(),
                )),
                b"BEFORE \"12-Apr-2023\"",
            ),
            (SearchKey::Body(AString::try_from("A").unwrap()), b"BODY A"),
            (SearchKey::Cc(AString::try_from("A").unwrap()), b"CC A"),
            (SearchKey::Deleted, b"DELETED"),
            (SearchKey::Draft, b"DRAFT"),
            (SearchKey::Flagged, b"FLAGGED"),
            (SearchKey::From(AString::try_from("A").unwrap()), b"FROM A"),
            (
                SearchKey::Header(
                    AString::try_from("A").unwrap(),
                    AString::try_from("B").unwrap(),
                ),
                b"HEADER A B",
            ),
            (
                SearchKey::Keyword(Atom::try_from("A").unwrap()),
                b"KEYWORD A",
            ),
            (SearchKey::Larger(42), b"LARGER 42"),
            (SearchKey::New, b"NEW"),
            (SearchKey::Not(Box::new(SearchKey::New)), b"NOT NEW"),
            (SearchKey::Old, b"OLD"),
            (
                SearchKey::On(NaiveDate(
                    chrono::NaiveDate::from_ymd_opt(2023, 04, 12).unwrap(),
                )),
                b"ON \"12-Apr-2023\"",
            ),
            (
                SearchKey::Or(Box::new(SearchKey::New), Box::new(SearchKey::Recent)),
                b"OR NEW RECENT",
            ),
            (SearchKey::Recent, b"RECENT"),
            (SearchKey::Seen, b"SEEN"),
            (
                SearchKey::SentBefore(NaiveDate(
                    chrono::NaiveDate::from_ymd_opt(2023, 04, 12).unwrap(),
                )),
                b"SENTBEFORE \"12-Apr-2023\"",
            ),
            (
                SearchKey::SentOn(NaiveDate(
                    chrono::NaiveDate::from_ymd_opt(2023, 04, 12).unwrap(),
                )),
                b"SENTON \"12-Apr-2023\"",
            ),
            (
                SearchKey::SentSince(NaiveDate(
                    chrono::NaiveDate::from_ymd_opt(2023, 04, 12).unwrap(),
                )),
                b"SENTSINCE \"12-Apr-2023\"",
            ),
            (
                SearchKey::Since(NaiveDate(
                    chrono::NaiveDate::from_ymd_opt(2023, 04, 12).unwrap(),
                )),
                b"SINCE \"12-Apr-2023\"",
            ),
            (SearchKey::Smaller(1337), b"SMALLER 1337"),
            (
                SearchKey::Subject(AString::try_from("A").unwrap()),
                b"SUBJECT A",
            ),
            (SearchKey::Text(AString::try_from("A").unwrap()), b"TEXT A"),
            (SearchKey::To(AString::try_from("A").unwrap()), b"TO A"),
            (
                SearchKey::Uid(SequenceSet::try_from(Sequence::try_from(1..).unwrap()).unwrap()),
                b"UID 1:*",
            ),
            (SearchKey::Unanswered, b"UNANSWERED"),
            (SearchKey::Undeleted, b"UNDELETED"),
            (SearchKey::Undraft, b"UNDRAFT"),
            (SearchKey::Unflagged, b"UNFLAGGED"),
            (
                SearchKey::Unkeyword(Atom::try_from("A").unwrap()),
                b"UNKEYWORD A",
            ),
            (SearchKey::Unseen, b"UNSEEN"),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }
}
