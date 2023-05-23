use std::num::NonZeroU32;

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    core::NString,
    message::{DateTime, FlagFetch, Section},
    response::data::{BodyStructure, Envelope},
};

/// There are three macros which specify commonly-used sets of data
/// items, and can be used instead of data items.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Macro {
    /// `ALL` Macro equivalent to:
    ///   `(FLAGS INTERNALDATE RFC822.SIZE ENVELOPE)`
    All,
    /// `FAST` Macro equivalent to:
    ///   `(FLAGS INTERNALDATE RFC822.SIZE)`
    Fast,
    /// `FULL` Macro equivalent to:
    ///   `(FLAGS INTERNALDATE RFC822.SIZE ENVELOPE BODY)`
    Full,
}

impl Macro {
    pub fn expand(&self) -> Vec<FetchAttribute> {
        use FetchAttribute::*;

        match self {
            Self::All => vec![Flags, InternalDate, Rfc822Size, Envelope],
            Self::Fast => vec![Flags, InternalDate, Rfc822Size],
            Self::Full => vec![Flags, InternalDate, Rfc822Size, Envelope, Body],
        }
    }
}

/// A macro must be used by itself, and not in conjunction with other macros or data items.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MacroOrFetchAttributes<'a> {
    Macro(Macro),
    FetchAttributes(Vec<FetchAttribute<'a>>),
}

impl<'a> From<Macro> for MacroOrFetchAttributes<'a> {
    fn from(m: Macro) -> Self {
        MacroOrFetchAttributes::Macro(m)
    }
}

impl<'a> From<Vec<FetchAttribute<'a>>> for MacroOrFetchAttributes<'a> {
    fn from(attributes: Vec<FetchAttribute<'a>>) -> Self {
        MacroOrFetchAttributes::FetchAttributes(attributes)
    }
}

/// The currently defined data items that can be fetched are:
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FetchAttribute<'a> {
    /// `BODY`
    ///
    /// Non-extensible form of `BODYSTRUCTURE`.
    Body,

    /// `BODY[<section>]<<partial>>`
    BodyExt {
        /// The text of a particular body section.  The section
        /// specification is a set of zero or more part specifiers
        /// delimited by periods.
        ///
        /// An empty section specification refers to the entire message, including the header.
        ///
        /// See [Section](Section) and [PartSpecifier](PartSpecifier).
        ///
        /// Every message has at least one part number.  Non-[MIME-IMB]
        /// messages, and non-multipart [MIME-IMB] messages with no
        /// encapsulated message, only have a part 1.
        ///
        /// Multipart messages are assigned consecutive part numbers, as
        /// they occur in the message.  If a particular part is of type
        /// message or multipart, its parts MUST be indicated by a period
        /// followed by the part number within that nested multipart part.
        ///
        /// A part of type MESSAGE/RFC822 also has nested part numbers,
        /// referring to parts of the MESSAGE part's body.
        section: Option<Section<'a>>,
        /// It is possible to fetch a substring of the designated text.
        /// This is done by appending an open angle bracket ("<"), the
        /// octet position of the first desired octet, a period, the
        /// maximum number of octets desired, and a close angle bracket
        /// (">") to the part specifier.  If the starting octet is beyond
        /// the end of the text, an empty string is returned.
        ///
        /// Any partial fetch that attempts to read beyond the end of the
        /// text is truncated as appropriate.  A partial fetch that starts
        /// at octet 0 is returned as a partial fetch, even if this
        /// truncation happened.
        ///
        ///    Note: This means that BODY[]<0.2048> of a 1500-octet message
        ///    will return BODY[]<0> with a literal of size 1500, not
        ///    BODY[].
        ///
        ///    Note: A substring fetch of a HEADER.FIELDS or
        ///    HEADER.FIELDS.NOT part specifier is calculated after
        ///    subsetting the header.
        partial: Option<(u32, NonZeroU32)>,
        /// Defines, wheather BODY or BODY.PEEK should be used.
        ///
        /// `BODY[...]` implicitly sets the `\Seen` flag where `BODY.PEEK[...]` does not.
        peek: bool,
    },

    /// `BODYSTRUCTURE`
    ///
    /// The [MIME-IMB] body structure of the message.  This is computed
    /// by the server by parsing the [MIME-IMB] header fields in the
    /// [RFC-2822] header and [MIME-IMB] headers.
    BodyStructure,

    /// `ENVELOPE`
    ///
    /// The envelope structure of the message.  This is computed by the
    /// server by parsing the [RFC-2822] header into the component
    /// parts, defaulting various fields as necessary.
    Envelope,

    /// `FLAGS`
    ///
    /// The flags that are set for this message.
    Flags,

    /// `INTERNALDATE`
    ///
    /// The internal date of the message.
    InternalDate,

    /// `RFC822`
    ///
    /// Functionally equivalent to `BODY[]`, differing in the syntax of
    /// the resulting untagged FETCH data (`RFC822` is returned).
    Rfc822,

    /// `RFC822.HEADER`
    ///
    /// Functionally equivalent to `BODY.PEEK[HEADER]`, differing in the
    /// syntax of the resulting untagged FETCH data (`RFC822.HEADER` is returned).
    Rfc822Header,

    /// `RFC822.SIZE`
    ///
    /// The [RFC-2822] size of the message.
    Rfc822Size,

    /// `RFC822.TEXT`
    ///
    /// Functionally equivalent to `BODY[TEXT]`, differing in the syntax
    /// of the resulting untagged FETCH data (`RFC822.TEXT` is returned).
    Rfc822Text,

    /// `UID`
    ///
    /// The unique identifier for the message.
    Uid,
}

/// The current data items are:
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FetchAttributeValue<'a> {
    /// A form of BODYSTRUCTURE without extension data.
    ///
    /// `BODY`
    Body(BodyStructure<'a>),

    /// A string expressing the body contents of the specified section.
    /// The string SHOULD be interpreted by the client according to the
    /// content transfer encoding, body type, and subtype.
    ///
    /// If the origin octet is specified, this string is a substring of
    /// the entire body contents, starting at that origin octet.  This
    /// means that BODY[]<0> MAY be truncated, but BODY[] is NEVER
    /// truncated.
    ///
    ///    Note: The origin octet facility MUST NOT be used by a server
    ///    in a FETCH response unless the client specifically requested
    ///    it by means of a FETCH of a BODY[<section>]<<partial>> data
    ///    item.
    ///
    /// 8-bit textual data is permitted if a [CHARSET] identifier is
    /// part of the body parameter parenthesized list for this section.
    /// Note that headers (part specifiers HEADER or MIME, or the
    /// header portion of a MESSAGE/RFC822 part), MUST be 7-bit; 8-bit
    /// characters are not permitted in headers.  Note also that the
    /// [RFC-2822] delimiting blank line between the header and the
    /// body is not affected by header line subsetting; the blank line
    /// is always included as part of header data, except in the case
    /// of a message which has no body and no blank line.
    ///
    /// Non-textual data such as binary data MUST be transfer encoded
    /// into a textual form, such as BASE64, prior to being sent to the
    /// client.  To derive the original binary data, the client MUST
    /// decode the transfer encoded string.
    ///
    /// `BODY[<section>]<<origin octet>>`
    BodyExt {
        section: Option<Section<'a>>,
        origin: Option<u32>,
        data: NString<'a>,
    },

    /// A parenthesized list that describes the [MIME-IMB] body
    /// structure of a message.  This is computed by the server by
    /// parsing the [MIME-IMB] header fields, defaulting various fields
    /// as necessary.
    ///
    /// `BODYSTRUCTURE`
    BodyStructure(BodyStructure<'a>),

    /// A parenthesized list that describes the envelope structure of a
    /// message.  This is computed by the server by parsing the
    /// [RFC-2822] header into the component parts, defaulting various
    /// fields as necessary.
    ///
    /// `ENVELOPE`
    Envelope(Envelope<'a>),

    /// A parenthesized list of flags that are set for this message.
    ///
    /// `FLAGS`
    Flags(Vec<FlagFetch<'a>>),

    /// A string representing the internal date of the message.
    ///
    /// `INTERNALDATE`
    InternalDate(DateTime),

    /// Equivalent to BODY[].
    ///
    /// `RFC822`
    Rfc822(NString<'a>),

    /// Equivalent to BODY[HEADER].  Note that this did not result in
    /// \Seen being set, because RFC822.HEADER response data occurs as
    /// a result of a FETCH of RFC822.HEADER.  BODY[HEADER] response
    /// data occurs as a result of a FETCH of BODY[HEADER] (which sets
    /// \Seen) or BODY.PEEK[HEADER] (which does not set \Seen).
    ///
    /// `RFC822.HEADER`
    Rfc822Header(NString<'a>),

    /// A number expressing the [RFC-2822] size of the message.
    ///
    /// `RFC822.SIZE`
    Rfc822Size(u32),

    /// Equivalent to BODY[TEXT].
    ///
    /// `RFC822.TEXT`
    Rfc822Text(NString<'a>),

    /// A number expressing the unique identifier of the message.
    ///
    /// `UID`
    Uid(NonZeroU32),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::IString,
        imap4rev1::body::{BasicFields, SpecificFields},
        response::data::Body,
        testing::known_answer_test_encode,
    };

    #[test]
    fn test_encode_fetch_attribute() {
        let tests = [
            (FetchAttribute::Body, b"BODY".as_ref()),
            (
                FetchAttribute::BodyExt {
                    section: None,
                    partial: None,
                    peek: false,
                },
                b"BODY[]",
            ),
            (FetchAttribute::BodyStructure, b"BODYSTRUCTURE"),
            (FetchAttribute::Envelope, b"ENVELOPE"),
            (FetchAttribute::Flags, b"FLAGS"),
            (FetchAttribute::InternalDate, b"INTERNALDATE"),
            (FetchAttribute::Rfc822, b"RFC822"),
            (FetchAttribute::Rfc822Header, b"RFC822.HEADER"),
            (FetchAttribute::Rfc822Size, b"RFC822.SIZE"),
            (FetchAttribute::Rfc822Text, b"RFC822.TEXT"),
            (FetchAttribute::Uid, b"UID"),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }

    #[test]
    fn test_encode_fetch_attribute_value() {
        let tests = [
            (
                FetchAttributeValue::Body(BodyStructure::Single {
                    body: Body {
                        basic: BasicFields {
                            parameter_list: vec![],
                            id: NString(None),
                            description: NString(None),
                            content_transfer_encoding: IString::try_from("base64").unwrap(),
                            size: 42,
                        },
                        specific: SpecificFields::Text {
                            subtype: IString::try_from("foo").unwrap(),
                            number_of_lines: 1337,
                        },
                    },
                    extension_data: None,
                }),
                b"BODY (\"TEXT\" \"foo\" NIL NIL NIL \"base64\" 42 1337)".as_ref(),
            ),
            (
                FetchAttributeValue::BodyExt {
                    section: None,
                    origin: None,
                    data: NString(None),
                },
                b"BODY[] NIL",
            ),
            (
                FetchAttributeValue::BodyExt {
                    section: None,
                    origin: Some(123),
                    data: NString(None),
                },
                b"BODY[]<123> NIL",
            ),
            (
                FetchAttributeValue::BodyStructure(BodyStructure::Single {
                    body: Body {
                        basic: BasicFields {
                            parameter_list: vec![],
                            id: NString(None),
                            description: NString(None),
                            content_transfer_encoding: IString::try_from("base64").unwrap(),
                            size: 213,
                        },
                        specific: SpecificFields::Text {
                            subtype: IString::try_from("").unwrap(),
                            number_of_lines: 224,
                        },
                    },
                    extension_data: None,
                }),
                b"BODYSTRUCTURE (\"TEXT\" \"\" NIL NIL NIL \"base64\" 213 224)",
            ),
            (
                FetchAttributeValue::Envelope(Envelope {
                    date: NString(None),
                    subject: NString(None),
                    from: vec![],
                    sender: vec![],
                    reply_to: vec![],
                    to: vec![],
                    cc: vec![],
                    bcc: vec![],
                    in_reply_to: NString(None),
                    message_id: NString(None),
                }),
                b"ENVELOPE (NIL NIL NIL NIL NIL NIL NIL NIL NIL NIL)",
            ),
            (FetchAttributeValue::Flags(vec![]), b"FLAGS ()"),
            (
                FetchAttributeValue::InternalDate(
                    DateTime::try_from(
                        chrono::DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0200")
                            .unwrap(),
                    )
                    .unwrap(),
                ),
                b"INTERNALDATE \"01-Jul-2003 10:52:37 +0200\"",
            ),
            (FetchAttributeValue::Rfc822(NString(None)), b"RFC822 NIL"),
            (
                FetchAttributeValue::Rfc822Header(NString(None)),
                b"RFC822.HEADER NIL",
            ),
            (FetchAttributeValue::Rfc822Size(3456), b"RFC822.SIZE 3456"),
            (
                FetchAttributeValue::Rfc822Text(NString(None)),
                b"RFC822.TEXT NIL",
            ),
            (
                FetchAttributeValue::Uid(NonZeroU32::try_from(u32::MAX).unwrap()),
                b"UID 4294967295",
            ),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }
}
