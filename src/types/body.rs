use crate::{
    codec::Codec,
    types::{
        core::{IString, NString, Number},
        envelope::Envelope,
    },
    List1AttributeValueOrNil, List1OrNil,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    /// Basic fields
    pub basic: BasicFields,
    /// Type-specific fields
    pub specific: SpecificFields,
}

impl Codec for Body {
    fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();

        match self.specific {
            SpecificFields::Basic {
                ref type_,
                ref subtype,
            } => {
                out.extend(&type_.serialize());
                out.push(b' ');
                out.extend(&subtype.serialize());
                out.push(b' ');
                out.extend(&self.basic.serialize());
            }
            SpecificFields::Message {
                ref envelope,
                ref body_structure,
                number_of_lines,
            } => {
                out.extend_from_slice(b"\"TEXT\" \"RFC822\" ");
                out.extend(&self.basic.serialize());
                out.push(b' ');
                out.extend(&envelope.serialize());
                out.push(b' ');
                out.extend(&body_structure.serialize());
                out.push(b' ');
                out.extend_from_slice(format!("{}", number_of_lines).as_bytes());
            }
            SpecificFields::Text {
                ref subtype,
                number_of_lines,
            } => {
                out.extend_from_slice(b"\"TEXT\" ");
                out.extend(&subtype.serialize());
                out.push(b' ');
                out.extend(&self.basic.serialize());
                out.push(b' ');
                out.extend_from_slice(format!("{}", number_of_lines).as_bytes());
            }
        }

        out
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

// impl std::fmt::Display for Body {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
//         let param_list = if self.parameter_list.is_empty() {
//             String::from("nil")
//         } else {
//             String::from("(")
//                 + &self
//                     .parameter_list
//                     .iter()
//                     .map(|(key, value)| format!("{} {}", key, value))
//                     .collect::<Vec<String>>()
//                     .join(" ")
//                 + ")"
//         };
//
//         match &self.specific {
//             SpecificFields::Basic { type_, subtype } => write!(
//                 f,
//                 "({} {} {} {} {} {} {})",
//                 type_,
//                 subtype,
//                 param_list,
//                 self.id,
//                 self.description,
//                 self.content_transfer_encoding,
//                 self.size
//             ),
//             SpecificFields::MessageRfc822 {
//                 envelope,
//                 body_structure,
//                 number_of_lines,
//             } => write!(
//                 f,
//                 r#"("message" "rfc822" {} {} {} {} {} {} {} {})"#,
//                 param_list,
//                 self.id,
//                 self.description,
//                 self.content_transfer_encoding,
//                 self.size,
//                 envelope,
//                 String::from_utf8(body_structure.serialize()).unwrap(),
//                 number_of_lines
//             ),
//             SpecificFields::Text {
//                 subtype,
//                 number_of_lines,
//             } => write!(
//                 f,
//                 r#"("text" {} {} {} {} {} {} {})"#,
//                 subtype,
//                 param_list,
//                 self.id,
//                 self.description,
//                 self.content_transfer_encoding,
//                 self.size,
//                 number_of_lines
//             ),
//         }
//     }
// }

/// The basic fields of a non-multipart body part.
#[derive(Debug, Clone, PartialEq)]
pub struct BasicFields {
    /// List of attribute/value pairs ([MIME-IMB].)
    pub parameter_list: Vec<(IString, IString)>,

    /// Content id ([MIME-IMB].)
    pub id: NString,

    /// Content description ([MIME-IMB].)
    pub description: NString,

    /// Content transfer encoding ([MIME-IMB].)
    pub content_transfer_encoding: IString,

    /// Size of the body in octets.
    ///
    /// Note that this size is the size in its transfer encoding
    /// and not the resulting size after any decoding.
    pub size: Number,
}

impl Codec for BasicFields {
    fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend(List1AttributeValueOrNil(&self.parameter_list).serialize());
        out.push(b' ');
        out.extend(&self.id.serialize());
        out.push(b' ');
        out.extend(&self.description.serialize());
        out.push(b' ');
        out.extend(&self.content_transfer_encoding.serialize());
        out.push(b' ');
        out.extend(format!("{}", self.size).as_bytes());
        out
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpecificFields {
    /// # Example (not in RFC)
    ///
    /// Single application/{voodoo, unknown, whatever, meh} is represented as "basic"
    ///
    /// ```text
    /// (
    ///     "application" "voodoo" NIL NIL NIL "7bit" 20
    ///                            ^^^ ^^^ ^^^ ^^^^^^ ^^
    ///                            |   |   |   |      | size
    ///                            |   |   |   | content transfer encoding
    ///                            |   |   | description
    ///                            |   | id
    ///                            | parameter list
    ///
    ///     NIL NIL NIL NIL
    ///     ^^^ ^^^ ^^^ ^^^
    ///     |   |   |   | location
    ///     |   |   | language
    ///     |   | disposition
    ///     | md5
    /// )
    /// ```
    Basic {
        /// A string giving the content media type name as defined in [MIME-IMB].
        type_: IString,

        /// A string giving the content subtype name as defined in [MIME-IMB].
        subtype: IString,
    },

    /// # Example (not in RFC)
    ///
    /// Single message/rfc822 is represented as "message"
    ///
    /// ```text
    /// (
    ///     "message" "rfc822" NIL NIL NIL "7bit" 123
    ///                        ^^^ ^^^ ^^^ ^^^^^^ ^^^
    ///                        |   |   |   |      | size
    ///                        |   |   |   | content transfer encoding
    ///                        |   |   | description
    ///                        |   | id
    ///                        | parameter list
    ///
    ///     # envelope
    ///     (
    ///         NIL "message.inner.subject.ljcwooqy" ((NIL NIL "extern" "company.com")) ((NIL NIL "extern" "company.com")) ((NIL NIL "extern" "company.com")) ((NIL NIL "admin" "seurity.com")) NIL NIL NIL NIL
    ///     )
    ///
    ///     # body structure
    ///     (
    ///         "text" "plain" ("charset" "us-ascii") NIL NIL "7bit" 31
    ///         2
    ///         NIL NIL NIL NIL
    ///     )
    ///
    ///     6
    ///     ^
    ///     | number of lines
    ///
    ///     NIL NIL NIL NIL
    ///     ^^^ ^^^ ^^^ ^^^
    ///     |   |   |   | location
    ///     |   |   | language
    ///     |   | disposition
    ///     | md5
    /// )
    /// ```
    ///
    /// A body type of type MESSAGE and subtype RFC822 contains, immediately after the basic fields,
    Message {
        /// the envelope structure,
        envelope: Envelope,
        /// body structure,
        body_structure: Box<BodyStructure>,
        /// and size in text lines of the encapsulated message.
        number_of_lines: Number,
    },

    /// # Example (not in RFC)
    ///
    /// Single text/plain is represented as "text"
    ///
    /// ```text
    /// (
    ///     "text" "plain" ("charset" "us-ascii") NIL NIL "7bit" 25
    ///                    ^^^^^^^^^^^^^^^^^^^^^^ ^^^ ^^^ ^^^^^^ ^^
    ///                    |                      |   |   |      | size
    ///                    |                      |   |   | content transfer encoding
    ///                    |                      |   | description
    ///                    |                      | id
    ///                    | parameter list
    ///
    ///     1
    ///     ^
    ///     | number of lines
    ///
    ///     NIL NIL NIL NIL
    ///     ^^^ ^^^ ^^^ ^^^
    ///     |   |   |   | location
    ///     |   |   | language
    ///     |   | disposition
    ///     | md5
    /// )
    /// ```
    ///
    /// A body type of type TEXT contains, immediately after the basic fields,
    Text {
        subtype: IString,
        /// the size of the body in text lines.
        number_of_lines: Number,
    },
}

/// The extension data of a non-multipart body part are in the following order:
#[derive(Debug, Clone, PartialEq)]
pub struct SinglePartExtensionData {
    /// A string giving the body MD5 value as defined in [MD5].
    pub md5: NString,

    /// A parenthesized list with the same content and function as
    /// the body disposition for a multipart body part.
    pub disposition: Option<Option<(IString, Vec<(IString, IString)>)>>,

    /// A string or parenthesized list giving the body language
    /// value as defined in [LANGUAGE-TAGS].
    pub language: Option<Vec<IString>>,

    /// A string list giving the body content URI as defined in [LOCATION].
    pub location: Option<NString>,

    pub extension: Vec<u8>,
}

impl Codec for SinglePartExtensionData {
    fn serialize(&self) -> Vec<u8> {
        let mut out = self.md5.serialize();
        if let Some(ref dsp) = self.disposition {
            out.push(b' ');
            match dsp {
                Some((s, param)) => {
                    out.extend(s.serialize());
                    out.push(b' ');
                    out.extend(&List1AttributeValueOrNil(&param).serialize());
                }
                None => out.extend_from_slice(b"NIL"),
            }

            if let Some(ref lang) = self.language {
                out.push(b' ');
                out.extend(&List1OrNil(lang, b" ").serialize());

                if let Some(ref loc) = self.location {
                    out.push(b' ');
                    out.extend(&loc.serialize());

                    if !self.extension.is_empty() {
                        out.push(b' ');
                        out.extend(&self.extension);
                    }
                }
            }
        }

        out
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

/// The extension data of a multipart body part are in the following order:
///
/// # Trace (not in RFC)
///
/// ```text
/// (
///   ("text" "html"  ("charset" "us-ascii") NIL NIL "7bit" 28 0 NIL NIL NIL NIL)
///   ("text" "plain" ("charset" "us-ascii") NIL NIL "7bit" 11 0 NIL NIL NIL NIL)
///   "mixed" ("boundary" "xxx") NIL NIL NIL
///           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///           |
///           | extension multipart data
/// )
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MultiPartExtensionData {
    /// `body parameter parenthesized list`
    ///
    /// A parenthesized list of attribute/value pairs [e.g., ("foo"
    /// "bar" "baz" "rag") where "bar" is the value of "foo", and
    /// "rag" is the value of "baz"] as defined in [MIME-IMB].
    pub parameter_list: Vec<(IString, IString)>,

    /// `body disposition`
    ///
    /// A parenthesized list, consisting of a disposition type
    /// string, followed by a parenthesized list of disposition
    /// attribute/value pairs as defined in [DISPOSITION].
    pub disposition: Option<Option<(IString, Vec<(IString, IString)>)>>,

    /// `body language`
    ///
    /// A string or parenthesized list giving the body language
    /// value as defined in [LANGUAGE-TAGS].
    pub language: Option<Vec<IString>>,

    /// `body location`
    ///
    /// A string list giving the body content URI as defined in
    /// [LOCATION].
    pub location: Option<NString>,

    pub extension: Vec<u8>,
}

impl Codec for MultiPartExtensionData {
    fn serialize(&self) -> Vec<u8> {
        let mut out = List1AttributeValueOrNil(&self.parameter_list).serialize();

        if let Some(ref dsp) = self.disposition {
            out.push(b' ');
            match dsp {
                Some((s, param)) => {
                    out.extend(s.serialize());
                    out.push(b' ');
                    out.extend(&List1AttributeValueOrNil(&param).serialize());
                }
                None => out.extend_from_slice(b"NIL"),
            }

            if let Some(ref lang) = self.language {
                out.push(b' ');
                out.extend(&List1OrNil(lang, b" ").serialize());

                if let Some(ref loc) = self.location {
                    out.push(b' ');
                    out.extend(&loc.serialize());

                    if !self.extension.is_empty() {
                        out.push(b' ');
                        out.extend(&self.extension);
                    }
                }
            }
        }

        out
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BodyStructure {
    /// For example, a simple text message of 48 lines and 2279 octets
    /// can have a body structure of:
    ///
    /// ```text
    /// ("TEXT" "PLAIN" ("CHARSET" "US-ASCII") NIL NIL "7BIT" 2279 48)
    /// ```
    Single {
        body: Body,
        /// Extension data
        ///
        /// Extension data is never returned with the BODY fetch,
        /// but can be returned with a BODYSTRUCTURE fetch.
        /// Extension data, if present, MUST be in the defined order.
        ///
        /// Any following extension data are not yet defined in this
        /// version of the protocol, and would be as described above under
        /// multipart extension data.
        extension: Option<SinglePartExtensionData>,
    },

    /// Multiple parts are indicated by parenthesis nesting.  Instead
    /// of a body type as the first element of the parenthesized list,
    /// there is a sequence of one or more nested body structures.  The
    /// second (last?!) element of the parenthesized list is the multipart
    /// subtype (mixed, digest, parallel, alternative, etc.).
    ///
    /// For example, a two part message consisting of a text and a
    /// BASE64-encoded text attachment can have a body structure of:
    ///
    /// ```text
    /// (
    ///     ("TEXT" "PLAIN" ("CHARSET" "US-ASCII") NIL NIL "7BIT" 1152 23)
    ///     ("TEXT" "PLAIN" ("CHARSET" "US-ASCII" "NAME" "cc.diff") "<960723163407.20117h@cac.washington.edu>" "Compiler diff" "BASE64" 4554 73)
    ///     "MIXED"
    /// )
    /// ```
    ///
    /// Extension data follows the multipart subtype.  Extension data
    /// is never returned with the BODY fetch, but can be returned with
    /// a BODYSTRUCTURE fetch.  Extension data, if present, MUST be in
    /// the defined order.
    ///
    /// See [ExtensionMultiPartData](struct.ExtensionMultiPartData.html).
    ///
    /// Any following extension data are not yet defined in this
    /// version of the protocol.  Such extension data can consist of
    /// zero or more NILs, strings, numbers, or potentially nested
    /// parenthesized lists of such data.  Client implementations that
    /// do a BODYSTRUCTURE fetch MUST be prepared to accept such
    /// extension data.  Server implementations MUST NOT send such
    /// extension data until it has been defined by a revision of this
    /// protocol.
    ///
    /// # Example (not in RFC)
    ///
    /// Multipart/mixed is represented as follows...
    ///
    /// ```text
    /// (
    ///     ("text" "html" ("charset" "us-ascii") NIL NIL "7bit" 28 0 NIL NIL NIL NIL)
    ///     ("text" "plain" ("charset" "us-ascii") NIL NIL "7bit" 11 0 NIL NIL NIL NIL)
    ///     "mixed" ("boundary" "xxx") NIL NIL NIL
    ///             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    ///             |
    ///             | extension data
    /// )
    /// ```
    Multi {
        bodies: Vec<BodyStructure>,
        subtype: IString,
        extension_data: Option<MultiPartExtensionData>,
    },
}

impl Codec for BodyStructure {
    fn serialize(&self) -> Vec<u8> {
        let mut out = b"(".to_vec();
        match self {
            BodyStructure::Single { body, extension } => {
                out.extend(&body.serialize());
                if let Some(extension) = extension {
                    out.push(b' ');
                    out.extend(&extension.serialize());
                }
            }
            BodyStructure::Multi {
                bodies,
                subtype,
                extension_data,
            } => {
                for body in bodies {
                    out.extend(&body.serialize());
                }
                out.push(b' ');
                out.extend(&subtype.serialize());

                if let Some(extension) = extension_data {
                    out.push(b' ');
                    out.extend(&extension.serialize());
                }
            }
        }
        out.push(b')');
        out
    }

    fn deserialize(_input: &[u8]) -> Result<(&[u8], Self), BodyStructure>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
