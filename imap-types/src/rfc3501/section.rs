use std::num::NonZeroU32;

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "serdex")]
use serde::{Deserialize, Serialize};

use crate::core::{AString, NonEmptyVec};

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
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Section<'a> {
    Part(Part),

    Header(Option<Part>),

    /// The subset returned by HEADER.FIELDS contains only those header fields with a field-name that
    /// matches one of the names in the list.
    HeaderFields(Option<Part>, NonEmptyVec<AString<'a>>), // TODO: what if none matches?

    /// Similarly, the subset returned by HEADER.FIELDS.NOT contains only the header fields
    /// with a non-matching field-name.
    HeaderFieldsNot(Option<Part>, NonEmptyVec<AString<'a>>), // TODO: what if none matches?

    /// The TEXT part specifier refers to the text body of the message, omitting the [RFC-2822] header.
    Text(Option<Part>),

    /// The MIME part specifier MUST be prefixed by one or more numeric part specifiers
    /// and refers to the [MIME-IMB] header for this part.
    Mime(Part),
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Part(pub NonEmptyVec<NonZeroU32>);
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
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PartSpecifier<'a> {
    PartNumber(u32),
    Header,
    HeaderFields(NonEmptyVec<AString<'a>>),
    HeaderFieldsNot(NonEmptyVec<AString<'a>>),
    Mime,
    Text,
}
