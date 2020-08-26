use crate::{
    codec::Codec,
    types::core::AString,
    utils::{join_bytes, join_serializable},
};

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

impl Codec for Macro {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Macro::All => b"ALL".to_vec(),
            Macro::Fast => b"FAST".to_vec(),
            Macro::Full => b"FULL".to_vec(),
        }
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

/// A macro must be used by itself, and not in conjunction with other macros or data items.
#[derive(Debug, Clone, PartialEq)]
pub enum MacroOrDataItems {
    Macro(Macro),
    DataItems(Vec<DataItem>),
}

impl Codec for MacroOrDataItems {
    fn serialize(&self) -> Vec<u8> {
        match self {
            MacroOrDataItems::Macro(m) => m.serialize(),
            MacroOrDataItems::DataItems(items) => {
                if items.len() == 1 {
                    items[0].serialize()
                } else {
                    let mut out = b"(".to_vec();
                    out.extend(join_serializable(items.as_slice(), b" "));
                    out.push(b')');
                    out
                }
            }
        }
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
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

impl Codec for DataItem {
    fn serialize(&self) -> Vec<u8> {
        match self {
            DataItem::Body => b"BODY".to_vec(),
            DataItem::BodyExt {
                section,
                partial,
                peek,
            } => {
                let mut out = if *peek {
                    b"BODY.PEEK[".to_vec()
                } else {
                    b"BODY[".to_vec()
                };
                if let Some(section) = section {
                    out.extend(section.serialize());
                }
                out.push(b']');
                if let Some((a, b)) = partial {
                    out.extend(format!("<{}.{}>", a, b).into_bytes());
                }
                out
            }
            DataItem::BodyStructure => b"BODYSTRUCTURE".to_vec(),
            DataItem::Envelope => b"ENVELOPE".to_vec(),
            DataItem::Flags => b"FLAGS".to_vec(),
            DataItem::InternalDate => b"INTERNALDATE".to_vec(),
            DataItem::Rfc822 => b"RFC822".to_vec(),
            DataItem::Rfc822Header => b"RFC822.HEADER".to_vec(),
            DataItem::Rfc822Size => b"RFC822.SIZE".to_vec(),
            DataItem::Rfc822Text => b"RFC822.TEXT".to_vec(),
            DataItem::Uid => b"UID".to_vec(),
        }
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
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

impl Codec for Section {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Section::Part(part) => part.serialize(),
            Section::Header(maybe_part) => match maybe_part {
                Some(part) => {
                    let mut out = part.serialize();
                    out.extend_from_slice(b".HEADER");
                    out
                }
                None => b"HEADER".to_vec(),
            },
            Section::HeaderFields(maybe_part, header_list) => {
                let mut out = match maybe_part {
                    Some(part) => {
                        let mut out = part.serialize();
                        out.extend_from_slice(b".HEADER.FIELDS (");
                        out
                    }
                    None => b"HEADER.FIElDS (".to_vec(),
                };
                out.extend(join_serializable(header_list, b" "));
                out.push(b')');
                out
            }
            Section::HeaderFieldsNot(maybe_part, header_list) => {
                let mut out = match maybe_part {
                    Some(part) => {
                        let mut out = part.serialize();
                        out.extend_from_slice(b".HEADER.FIELDS.NOT (");
                        out
                    }
                    None => b"HEADER.FIElDS.NOT (".to_vec(),
                };
                out.extend(join_serializable(header_list, b" "));
                out.push(b')');
                out
            }
            Section::Text(maybe_part) => match maybe_part {
                Some(part) => {
                    let mut out = part.serialize();
                    out.extend_from_slice(b".TEXT");
                    out
                }
                None => b"TEXT".to_vec(),
            },
            Section::Mime(part) => {
                let mut out = part.serialize();
                out.extend_from_slice(b".TEXT");
                out
            }
        }
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Part(pub Vec<u32>);

impl Codec for Part {
    fn serialize(&self) -> Vec<u8> {
        join_bytes(
            self.0
                .iter()
                .map(|num| format!("{}", num).into_bytes())
                .collect::<Vec<Vec<u8>>>(),
            b".",
        )
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
