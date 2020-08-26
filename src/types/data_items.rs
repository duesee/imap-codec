use crate::types::core::AString;

/// There are three macros which specify commonly-used sets of data
/// items, and can be used instead of data items.
#[derive(Debug, Clone, PartialEq)]
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
    pub fn expand(&self) -> Vec<DataItem> {
        use DataItem::*;

        match self {
            Self::All => vec![Flags, InternalDate, Rfc822Size, Envelope],
            Self::Fast => vec![Flags, InternalDate, Rfc822Size],
            Self::Full => vec![Flags, InternalDate, Rfc822Size, Envelope, Body],
        }
    }
}

/// A macro must be used by itself, and not in conjunction with other macros or data items.
#[derive(Debug, Clone, PartialEq)]
pub enum MacroOrDataItems {
    Macro(Macro),
    DataItems(Vec<DataItem>),
}

/// The currently defined data items that can be fetched are:
#[derive(Debug, Clone, PartialEq)]
pub enum DataItem {
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
        section: Option<Section>,
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
        ///
        partial: Option<(u32, u32)>,
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
#[derive(Debug, Clone, PartialEq)]
pub enum PartSpecifier {
    PartNumber(u32),
    Header,
    HeaderFields(Vec<AString>),
    HeaderFieldsNot(Vec<AString>),
    Mime,
    Text,
}

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
#[derive(Debug, Clone, PartialEq)]
pub enum Section {
    Part(Part),

    Header(Option<Part>),

    /// The subset returned by HEADER.FIELDS contains only those header fields with a field-name that
    /// matches one of the names in the list.
    HeaderFields(Option<Part>, Vec<AString>),

    /// Similarly, the subset returned by HEADER.FIELDS.NOT contains only the header fields
    /// with a non-matching field-name.
    HeaderFieldsNot(Option<Part>, Vec<AString>),

    /// The TEXT part specifier refers to the text body of the message, omitting the [RFC-2822] header.
    Text(Option<Part>),

    /// The MIME part specifier MUST be prefixed by one or more numeric part specifiers
    /// and refers to the [MIME-IMB] header for this part.
    Mime(Part),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Part(pub Vec<u32>);
