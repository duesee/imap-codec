//! Fetch-related types.

use std::{
    fmt::{Display, Formatter},
    num::NonZeroU32,
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    body::BodyStructure,
    core::{AString, NString, NString8, Vec1},
    datetime::DateTime,
    envelope::Envelope,
    flag::FlagFetch,
};

/// Shorthands for commonly-used message data items.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
#[non_exhaustive]
pub enum Macro {
    /// Shorthand for `(FLAGS INTERNALDATE RFC822.SIZE)`.
    Fast,
    /// Shorthand for `(FLAGS INTERNALDATE RFC822.SIZE ENVELOPE)`.
    All,
    /// Shorthand for `(FLAGS INTERNALDATE RFC822.SIZE ENVELOPE BODY)`.
    Full,
}

impl Macro {
    pub fn expand(&self) -> Vec<MessageDataItemName> {
        use MessageDataItemName::*;

        match self {
            Self::All => vec![Flags, InternalDate, Rfc822Size, Envelope],
            Self::Fast => vec![Flags, InternalDate, Rfc822Size],
            Self::Full => vec![Flags, InternalDate, Rfc822Size, Envelope, Body],
        }
    }
}

impl Display for Macro {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Macro::All => "ALL",
            Macro::Fast => "FAST",
            Macro::Full => "FULL",
        })
    }
}

/// Either a macro or a list of message data items.
///
/// A macro must be used by itself, and not in conjunction with other macros or data items.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum MacroOrMessageDataItemNames<'a> {
    Macro(Macro),
    MessageDataItemNames(Vec<MessageDataItemName<'a>>),
}

impl<'a> From<Macro> for MacroOrMessageDataItemNames<'a> {
    fn from(m: Macro) -> Self {
        MacroOrMessageDataItemNames::Macro(m)
    }
}

impl<'a> From<Vec<MessageDataItemName<'a>>> for MacroOrMessageDataItemNames<'a> {
    fn from(item_names: Vec<MessageDataItemName<'a>>) -> Self {
        MacroOrMessageDataItemNames::MessageDataItemNames(item_names)
    }
}

/// Message data item name used to request a message data item.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
#[doc(alias = "FetchAttribute")]
pub enum MessageDataItemName<'a> {
    /// Non-extensible form of `BODYSTRUCTURE`.
    ///
    /// ```imap
    /// BODY
    /// ```
    Body,

    /// The text of a particular body section.
    ///
    /// ```imap
    /// BODY[<section>]<<partial>>
    /// ```
    BodyExt {
        /// The section specification is a set of zero or more part specifiers delimited by periods.
        ///
        /// An empty section specification refers to the entire message, including the header.
        ///
        /// See [`crate::fetch::Section`] and [`crate::fetch::PartSpecifier`].
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

    /// The [MIME-IMB] body structure of a message.
    ///
    /// This is computed by the server by parsing the [MIME-IMB] header fields in the [RFC-2822]
    /// header and [MIME-IMB] headers.
    ///
    /// ```imap
    /// BODYSTRUCTURE
    /// ```
    BodyStructure,

    /// The envelope structure of a message.
    ///
    /// This is computed by the server by parsing the [RFC-2822] header into the component parts,
    /// defaulting various fields as necessary.
    ///
    /// ```imap
    /// ENVELOPE
    /// ```
    Envelope,

    /// The flags that are set for a message.
    ///
    /// ```imap
    /// FLAGS
    /// ```
    Flags,

    /// The internal date of a message.
    ///
    /// ```imap
    /// INTERNALDATE
    /// ```
    InternalDate,

    /// Functionally equivalent to `BODY[]`.
    ///
    /// Differs in the syntax of the resulting untagged FETCH data (`RFC822` is returned).
    ///
    /// ```imap
    /// RFC822
    /// ```
    ///
    /// Note: `BODY[]` is constructed as ...
    ///
    /// ```rust
    /// # use imap_types::fetch::MessageDataItemName;
    /// MessageDataItemName::BodyExt {
    ///     section: None,
    ///     partial: None,
    ///     peek: false,
    /// };
    /// ```
    Rfc822,

    /// Functionally equivalent to `BODY.PEEK[HEADER]`.
    ///
    /// Differs in the syntax of the resulting untagged FETCH data (`RFC822.HEADER` is returned).
    ///
    /// ```imap
    /// RFC822.HEADER
    /// ```
    Rfc822Header,

    /// The [RFC-2822] size of a message.
    ///
    /// ```imap
    /// RFC822.SIZE
    /// ```
    Rfc822Size,

    /// Functionally equivalent to `BODY[TEXT]`.
    ///
    /// Differs in the syntax of the resulting untagged FETCH data (`RFC822.TEXT` is returned).
    /// ```imap
    /// RFC822.TEXT
    /// ```
    Rfc822Text,

    /// The unique identifier for a message.
    ///
    /// ```imap
    /// UID
    /// ```
    Uid,

    Binary {
        section: Vec<NonZeroU32>,
        partial: Option<(u32, NonZeroU32)>,
        peek: bool,
    },

    BinarySize {
        section: Vec<NonZeroU32>,
    },
}

/// Message data item.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
#[doc(alias = "FetchAttributeValue")]
pub enum MessageDataItem<'a> {
    /// A form of `BODYSTRUCTURE` without extension data.
    ///
    /// ```imap
    /// BODY
    /// ```
    Body(BodyStructure<'a>),

    /// The body contents of the specified section.
    ///
    /// 8-bit textual data is permitted if a \[CHARSET\] identifier is
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
    /// ```imap
    /// BODY[<section>]<<origin octet>>
    /// ```
    BodyExt {
        /// The specified section.
        section: Option<Section<'a>>,
        /// If the origin octet is specified, this string is a substring of
        /// the entire body contents, starting at that origin octet.  This
        /// means that `BODY[]<0>` MAY be truncated, but `BODY[]` is NEVER
        /// truncated.
        ///
        ///    Note: The origin octet facility MUST NOT be used by a server
        ///    in a FETCH response unless the client specifically requested
        ///    it by means of a FETCH of a `BODY[<section>]<<partial>>` data
        ///    item.
        origin: Option<u32>,
        /// The string SHOULD be interpreted by the client according to the
        /// content transfer encoding, body type, and subtype.
        data: NString<'a>,
    },

    /// The [MIME-IMB] body structure of a message.
    ///
    /// This is computed by the server by parsing the [MIME-IMB] header fields, defaulting various
    /// fields as necessary.
    ///
    /// ```imap
    /// BODYSTRUCTURE
    /// ```
    BodyStructure(BodyStructure<'a>),

    /// The envelope structure of a message.
    ///
    /// This is computed by the server by parsing the [RFC-2822] header into the component parts,
    /// defaulting various fields as necessary.
    ///
    /// ```imap
    /// ENVELOPE
    /// ```
    Envelope(Envelope<'a>),

    /// A list of flags that are set for a message.
    ///
    /// ```imap
    /// FLAGS
    /// ```
    Flags(Vec<FlagFetch<'a>>),

    /// A string representing the internal date of a message.
    ///
    /// ```imap
    /// INTERNALDATE
    /// ```
    InternalDate(DateTime),

    /// Equivalent to `BODY[]`.
    ///
    /// ```imap
    /// RFC822
    /// ```
    Rfc822(NString<'a>),

    /// Equivalent to `BODY[HEADER]`.
    ///
    /// Note that this did not result in `\Seen` being set, because `RFC822.HEADER` response data
    /// occurs as a result of a `FETCH` of `RFC822.HEADER`. `BODY[HEADER]` response data occurs as a
    /// result of a `FETCH` of `BODY[HEADER]` (which sets `\Seen`) or `BODY.PEEK[HEADER]` (which
    /// does not set `\Seen`).
    ///
    /// ```imap
    /// RFC822.HEADER
    /// ```
    Rfc822Header(NString<'a>),

    /// A number expressing the [RFC-2822] size of a message.
    ///
    /// ```imap
    /// RFC822.SIZE
    /// ```
    Rfc822Size(u32),

    /// Equivalent to `BODY[TEXT]`.
    ///
    /// ```imap
    /// RFC822.TEXT
    /// ```
    Rfc822Text(NString<'a>),

    /// A number expressing the unique identifier of a message.
    ///
    /// ```imap
    /// UID
    /// ```
    Uid(NonZeroU32),

    Binary {
        section: Vec<NonZeroU32>,
        value: NString8<'a>,
    },

    BinarySize {
        section: Vec<NonZeroU32>,
        size: u32,
    },
}

/// A part specifier is either a part number or one of the following:
/// `HEADER`, `HEADER.FIELDS`, `HEADER.FIELDS.NOT`, `MIME`, and `TEXT`.
///
/// The HEADER, HEADER.FIELDS, and HEADER.FIELDS.NOT part
/// specifiers refer to the [RFC-2822] header of the message or of
/// an encapsulated [MIME-IMT] MESSAGE/RFC822 message.
/// HEADER.FIELDS and HEADER.FIELDS.NOT are followed by a list of
/// field-name (as defined in [RFC-2822]) names, and return a
/// subset of the header.
///
/// The field-matching is case-insensitive but otherwise exact.
/// Subsetting does not exclude the [RFC-2822] delimiting blank line between the header
/// and the body; the blank line is included in all header fetches,
/// except in the case of a message which has no body and no blank
/// line.
///
/// The HEADER, HEADER.FIELDS, HEADER.FIELDS.NOT, and TEXT part
/// specifiers can be the sole part specifier or can be prefixed by
/// one or more numeric part specifiers, provided that the numeric
/// part specifier refers to a part of type MESSAGE/RFC822.
///
/// Here is an example of a complex message with some of its part specifiers:
///
/// ```text
/// HEADER     ([RFC-2822] header of the message)
/// TEXT       ([RFC-2822] text body of the message) MULTIPART/MIXED
/// 1          TEXT/PLAIN
/// 2          APPLICATION/OCTET-STREAM
/// 3          MESSAGE/RFC822
/// 3.HEADER   ([RFC-2822] header of the message)
/// 3.TEXT     ([RFC-2822] text body of the message) MULTIPART/MIXED
/// 3.1        TEXT/PLAIN
/// 3.2        APPLICATION/OCTET-STREAM
/// 4          MULTIPART/MIXED
/// 4.1        IMAGE/GIF
/// 4.1.MIME   ([MIME-IMB] header for the IMAGE/GIF)
/// 4.2        MESSAGE/RFC822
/// 4.2.HEADER ([RFC-2822] header of the message)
/// 4.2.TEXT   ([RFC-2822] text body of the message) MULTIPART/MIXED
/// 4.2.1      TEXT/PLAIN
/// 4.2.2      MULTIPART/ALTERNATIVE
/// 4.2.2.1    TEXT/PLAIN
/// 4.2.2.2    TEXT/RICHTEXT
/// ```
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum Section<'a> {
    Part(Part),

    Header(Option<Part>),

    /// The subset returned by HEADER.FIELDS contains only those header fields with a field-name that
    /// matches one of the names in the list.
    HeaderFields(Option<Part>, Vec1<AString<'a>>), // TODO: what if none matches?

    /// Similarly, the subset returned by HEADER.FIELDS.NOT contains only the header fields
    /// with a non-matching field-name.
    HeaderFieldsNot(Option<Part>, Vec1<AString<'a>>), // TODO: what if none matches?

    /// The TEXT part specifier refers to the text body of the message, omitting the [RFC-2822] header.
    Text(Option<Part>),

    /// The MIME part specifier MUST be prefixed by one or more numeric part specifiers
    /// and refers to the [MIME-IMB] header for this part.
    Mime(Part),
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct Part(pub Vec1<NonZeroU32>);

/// A part specifier is either a part number or one of the following:
/// `HEADER`, `HEADER.FIELDS`, `HEADER.FIELDS.NOT`, `MIME`, and `TEXT`.
///
/// The HEADER, HEADER.FIELDS, and HEADER.FIELDS.NOT part
/// specifiers refer to the [RFC-2822] header of the message or of
/// an encapsulated [MIME-IMT] MESSAGE/RFC822 message.
/// HEADER.FIELDS and HEADER.FIELDS.NOT are followed by a list of
/// field-name (as defined in [RFC-2822]) names, and return a
/// subset of the header.
///
/// The field-matching is case-insensitive but otherwise exact.
/// Subsetting does not exclude the [RFC-2822] delimiting blank line between the header
/// and the body; the blank line is included in all header fetches,
/// except in the case of a message which has no body and no blank
/// line.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum PartSpecifier<'a> {
    PartNumber(u32),
    Header,
    HeaderFields(Vec1<AString<'a>>),
    HeaderFieldsNot(Vec1<AString<'a>>),
    Mime,
    Text,
}
