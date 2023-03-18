//! # Serialization of messages
//!
//! Every type in imap-codec can be serialized into bytes (`&[u8]`) by using the [Encode](crate::codec::Encode) trait.
//!
//! ## Example
//!
//! ```rust
//! use imap_types::{
//!     codec::Encode,
//!     response::{Greeting, Response},
//! };
//!
//! let rsp = Greeting::ok(None, "Hello, World!").unwrap();
//!
//! let bytes = rsp.encode_detached().unwrap();
//!
//! println!("{}", String::from_utf8(bytes).unwrap());
//! ```

use std::{io::Write, num::NonZeroU32};

use base64::{engine::general_purpose::STANDARD as base64, Engine};
use chrono::{DateTime, FixedOffset};

#[cfg(feature = "ext_compress")]
use crate::extensions::rfc4987::CompressionAlgorithm;
use crate::{
    codec::utils::{join_serializable, List1AttributeValueOrNil, List1OrNil},
    command::{
        fetch::{FetchAttribute, Macro, MacroOrFetchAttributes},
        search::SearchKey,
        status::StatusAttribute,
        store::{StoreResponse, StoreType},
        AuthenticateData, Command, CommandBody, ListCharString, ListMailbox, SeqNo, Sequence,
        SequenceSet,
    },
    core::{AString, Atom, AtomExt, IString, Literal, NString, Quoted},
    message::{
        AuthMechanism, AuthMechanismOther, Charset, Flag, FlagNameAttribute, Mailbox, MailboxOther,
        MyDateTime, MyNaiveDate, Part, Section, Tag,
    },
    response::{
        data::{
            Address, BasicFields, Body, BodyStructure, Capability, Envelope, FetchAttributeValue,
            MultiPartExtensionData, QuotedChar, SinglePartExtensionData, SpecificFields,
            StatusAttributeValue,
        },
        Code, CodeOther, Continue, Data, Greeting, GreetingKind, Response, Status, Text,
    },
    utils::escape_quoted,
};

pub trait Encode {
    #[must_use]
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()>;

    fn encode_detached(&self) -> std::io::Result<Vec<u8>> {
        let mut serialized = Vec::new();
        self.encode(&mut serialized)?;
        Ok(serialized)
    }
}

// ----- Primitive -----

impl Encode for u32 {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(self.to_string().as_bytes())
    }
}

impl Encode for u64 {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(self.to_string().as_bytes())
    }
}

// ----- Command -----

impl<'a> Encode for Command<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.tag.encode(writer)?;
        writer.write_all(b" ")?;
        self.body.encode(writer)?;
        writer.write_all(b"\r\n")
    }
}

impl<'a> Encode for Tag<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(self.inner().as_bytes())
    }
}

impl<'a> Encode for CommandBody<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            CommandBody::Capability => writer.write_all(b"CAPABILITY"),
            CommandBody::Noop => writer.write_all(b"NOOP"),
            CommandBody::Logout => writer.write_all(b"LOGOUT"),
            #[cfg(feature = "starttls")]
            CommandBody::StartTLS => writer.write_all(b"STARTTLS"),
            CommandBody::Authenticate {
                mechanism,
                #[cfg(feature = "ext_sasl_ir")]
                initial_response,
            } => {
                writer.write_all(b"AUTHENTICATE")?;
                writer.write_all(b" ")?;
                mechanism.encode(writer)?;

                #[cfg(feature = "ext_sasl_ir")]
                if let Some(ir) = initial_response {
                    writer.write_all(b" ")?;

                    // RFC 4959 (https://datatracker.ietf.org/doc/html/rfc4959#section-3)
                    // "To send a zero-length initial response, the client MUST send a single pad character ("=").
                    // This indicates that the response is present, but is a zero-length string."
                    if ir.is_empty() {
                        writer.write_all(b"=")?;
                    } else {
                        writer.write_all(base64.encode(ir).as_bytes())?;
                    };
                };

                Ok(())
            }
            CommandBody::Login { username, password } => {
                writer.write_all(b"LOGIN")?;
                writer.write_all(b" ")?;
                username.encode(writer)?;
                writer.write_all(b" ")?;
                password.encode(writer)
            }
            CommandBody::Select { mailbox } => {
                writer.write_all(b"SELECT")?;
                writer.write_all(b" ")?;
                mailbox.encode(writer)
            }
            CommandBody::Examine { mailbox } => {
                writer.write_all(b"EXAMINE")?;
                writer.write_all(b" ")?;
                mailbox.encode(writer)
            }
            CommandBody::Create { mailbox } => {
                writer.write_all(b"CREATE")?;
                writer.write_all(b" ")?;
                mailbox.encode(writer)
            }
            CommandBody::Delete { mailbox } => {
                writer.write_all(b"DELETE")?;
                writer.write_all(b" ")?;
                mailbox.encode(writer)
            }
            CommandBody::Rename {
                mailbox,
                new_mailbox,
            } => {
                writer.write_all(b"RENAME")?;
                writer.write_all(b" ")?;
                mailbox.encode(writer)?;
                writer.write_all(b" ")?;
                new_mailbox.encode(writer)
            }
            CommandBody::Subscribe { mailbox } => {
                writer.write_all(b"SUBSCRIBE")?;
                writer.write_all(b" ")?;
                mailbox.encode(writer)
            }
            CommandBody::Unsubscribe { mailbox } => {
                writer.write_all(b"UNSUBSCRIBE")?;
                writer.write_all(b" ")?;
                mailbox.encode(writer)
            }
            CommandBody::List {
                reference,
                mailbox_wildcard,
            } => {
                writer.write_all(b"LIST")?;
                writer.write_all(b" ")?;
                reference.encode(writer)?;
                writer.write_all(b" ")?;
                mailbox_wildcard.encode(writer)
            }
            CommandBody::Lsub {
                reference,
                mailbox_wildcard,
            } => {
                writer.write_all(b"LSUB")?;
                writer.write_all(b" ")?;
                reference.encode(writer)?;
                writer.write_all(b" ")?;
                mailbox_wildcard.encode(writer)
            }
            CommandBody::Status {
                mailbox,
                attributes,
            } => {
                writer.write_all(b"STATUS")?;
                writer.write_all(b" ")?;
                mailbox.encode(writer)?;
                writer.write_all(b" ")?;
                writer.write_all(b"(")?;
                join_serializable(attributes, b" ", writer)?;
                writer.write_all(b")")
            }
            CommandBody::Append {
                mailbox,
                flags,
                date,
                message,
            } => {
                writer.write_all(b"APPEND")?;
                writer.write_all(b" ")?;
                mailbox.encode(writer)?;

                if !flags.is_empty() {
                    writer.write_all(b" ")?;
                    writer.write_all(b"(")?;
                    join_serializable(flags, b" ", writer)?;
                    writer.write_all(b")")?;
                }

                if let Some(date) = date {
                    writer.write_all(b" ")?;
                    date.encode(writer)?;
                }

                writer.write_all(b" ")?;
                writer.write_all(format!("{{{}}}\r\n", message.as_ref().len()).as_bytes())?;
                writer.write_all(message.as_ref())
            }
            CommandBody::Check => writer.write_all(b"CHECK"),
            CommandBody::Close => writer.write_all(b"CLOSE"),
            CommandBody::Expunge => writer.write_all(b"EXPUNGE"),
            CommandBody::Search {
                charset,
                criteria,
                uid,
            } => {
                if *uid {
                    writer.write_all(b"UID SEARCH")?;
                } else {
                    writer.write_all(b"SEARCH")?;
                }
                if let Some(charset) = charset {
                    writer.write_all(b" CHARSET ")?;
                    charset.encode(writer)?;
                }
                writer.write_all(b" ")?;
                if let SearchKey::And(search_keys) = criteria {
                    join_serializable(search_keys.as_ref(), b" ", writer) // TODO: use List1?
                } else {
                    criteria.encode(writer)
                }
            }
            CommandBody::Fetch {
                sequence_set,
                attributes,
                uid,
            } => {
                if *uid {
                    writer.write_all(b"UID FETCH ")?;
                } else {
                    writer.write_all(b"FETCH ")?;
                }

                sequence_set.encode(writer)?;
                writer.write_all(b" ")?;
                attributes.encode(writer)
            }
            CommandBody::Store {
                sequence_set,
                kind,
                response,
                flags,
                uid,
            } => {
                if *uid {
                    writer.write_all(b"UID STORE ")?;
                } else {
                    writer.write_all(b"STORE ")?;
                }

                sequence_set.encode(writer)?;
                writer.write_all(b" ")?;

                match kind {
                    StoreType::Add => writer.write_all(b"+")?,
                    StoreType::Remove => writer.write_all(b"-")?,
                    StoreType::Replace => {}
                }

                writer.write_all(b"FLAGS")?;

                match response {
                    StoreResponse::Answer => {}
                    StoreResponse::Silent => writer.write_all(b".SILENT")?,
                }

                writer.write_all(b" (")?;
                join_serializable(flags, b" ", writer)?;
                writer.write_all(b")")
            }
            CommandBody::Copy {
                sequence_set,
                mailbox,
                uid,
            } => {
                if *uid {
                    writer.write_all(b"UID COPY ")?;
                } else {
                    writer.write_all(b"COPY ")?;
                }
                sequence_set.encode(writer)?;
                writer.write_all(b" ")?;
                mailbox.encode(writer)
            }
            #[cfg(feature = "ext_idle")]
            CommandBody::Idle => writer.write_all(b"IDLE"),
            #[cfg(feature = "ext_enable")]
            CommandBody::Enable { capabilities } => {
                writer.write_all(b"ENABLE ")?;
                join_serializable(capabilities.as_ref(), b" ", writer)
            }
            #[cfg(feature = "ext_compress")]
            CommandBody::Compress { algorithm } => {
                writer.write_all(b"COMPRESS ")?;
                algorithm.encode(writer)
            }
            #[cfg(feature = "ext_quota")]
            CommandBody::GetQuota { root } => {
                writer.write_all(b"GETQUOTA ")?;
                root.encode(writer)
            }
            #[cfg(feature = "ext_quota")]
            CommandBody::GetQuotaRoot { mailbox } => {
                writer.write_all(b"GETQUOTAROOT ")?;
                mailbox.encode(writer)
            }
            #[cfg(feature = "ext_quota")]
            CommandBody::SetQuota { root, quotas } => {
                writer.write_all(b"SETQUOTA ")?;
                root.encode(writer)?;
                writer.write_all(b" (")?;
                join_serializable(quotas.as_ref(), b" ", writer)?;
                writer.write_all(b")")
            }
        }
    }
}

impl<'a> Encode for AuthMechanism<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match &self {
            AuthMechanism::Plain => writer.write_all(b"PLAIN"),
            AuthMechanism::Login => writer.write_all(b"LOGIN"),
            AuthMechanism::Other(other) => other.encode(writer),
        }
    }
}

impl<'a> Encode for AuthMechanismOther<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.inner().encode(writer)
    }
}

impl Encode for AuthenticateData {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        let encoded = base64.encode(&self.data);
        writer.write_all(encoded.as_bytes())
    }
}

impl<'a> Encode for AString<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            AString::Atom(atom) => atom.encode(writer),
            AString::String(imap_str) => imap_str.encode(writer),
        }
    }
}

impl<'a> Encode for Atom<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(self.inner().as_bytes())
    }
}

impl<'a> Encode for AtomExt<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(self.inner().as_bytes())
    }
}

impl<'a> Encode for IString<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Self::Literal(val) => val.encode(writer),
            Self::Quoted(val) => val.encode(writer),
        }
    }
}

impl<'a> Encode for Literal<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "{{{}}}\r\n", self.as_ref().len())?;
        writer.write_all(self.as_ref())
    }
}

impl<'a> Encode for Quoted<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "\"{}\"", escape_quoted(self.inner()))
    }
}

impl<'a> Encode for Mailbox<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Mailbox::Inbox => writer.write_all(b"INBOX"),
            Mailbox::Other(other) => other.encode(writer),
        }
    }
}

impl<'a> Encode for MailboxOther<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.inner.encode(writer)
    }
}

impl<'a> Encode for ListMailbox<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            ListMailbox::Token(lcs) => lcs.encode(writer),
            ListMailbox::String(istr) => istr.encode(writer),
        }
    }
}

impl<'a> Encode for ListCharString<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(self.as_ref())
    }
}

impl Encode for StatusAttribute {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            StatusAttribute::Messages => writer.write_all(b"MESSAGES"),
            StatusAttribute::Recent => writer.write_all(b"RECENT"),
            StatusAttribute::UidNext => writer.write_all(b"UIDNEXT"),
            StatusAttribute::UidValidity => writer.write_all(b"UIDVALIDITY"),
            StatusAttribute::Unseen => writer.write_all(b"UNSEEN"),
            #[cfg(feature = "ext_quota")]
            StatusAttribute::Deleted => writer.write_all(b"DELETED"),
            #[cfg(feature = "ext_quota")]
            StatusAttribute::DeletedStorage => writer.write_all(b"DELETED-STORAGE"),
        }
    }
}

impl<'a> Encode for Flag<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            // ----- System -----
            Flag::Seen => writer.write_all(b"\\Seen"),
            Flag::Answered => writer.write_all(b"\\Answered"),
            Flag::Flagged => writer.write_all(b"\\Flagged"),
            Flag::Deleted => writer.write_all(b"\\Deleted"),
            Flag::Draft => writer.write_all(b"\\Draft"),

            // ----- Fetch -----
            Flag::Recent => writer.write_all(b"\\Recent"),

            // ----- Selectability -----
            Flag::NameAttribute(flag) => flag.encode(writer),

            // ----- Keyword -----
            Flag::Permanent => writer.write_all(b"\\*"),
            Flag::Keyword(atom) => atom.encode(writer),

            // ----- Others -----
            Flag::Extension(atom) => {
                writer.write_all(b"\\")?;
                atom.encode(writer)
            }
        }
    }
}

impl Encode for MyDateTime {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.0.encode(writer)
    }
}

impl<'a> Encode for Charset<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Charset::Atom(atom) => atom.encode(writer),
            Charset::Quoted(quoted) => quoted.encode(writer),
        }
    }
}

impl<'a> Encode for SearchKey<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            SearchKey::All => writer.write_all(b"ALL"),
            SearchKey::Answered => writer.write_all(b"ANSWERED"),
            SearchKey::Bcc(astring) => {
                writer.write_all(b"BCC ")?;
                astring.encode(writer)
            }
            SearchKey::Before(date) => {
                writer.write_all(b"BEFORE ")?;
                date.encode(writer)
            }
            SearchKey::Body(astring) => {
                writer.write_all(b"BODY ")?;
                astring.encode(writer)
            }
            SearchKey::Cc(astring) => {
                writer.write_all(b"CC ")?;
                astring.encode(writer)
            }
            SearchKey::Deleted => writer.write_all(b"DELETED"),
            SearchKey::Flagged => writer.write_all(b"FLAGGED"),
            SearchKey::From(astring) => {
                writer.write_all(b"FROM ")?;
                astring.encode(writer)
            }
            SearchKey::Keyword(flag_keyword) => {
                writer.write_all(b"KEYWORD ")?;
                flag_keyword.encode(writer)
            }
            SearchKey::New => writer.write_all(b"NEW"),
            SearchKey::Old => writer.write_all(b"OLD"),
            SearchKey::On(date) => {
                writer.write_all(b"ON ")?;
                date.encode(writer)
            }
            SearchKey::Recent => writer.write_all(b"RECENT"),
            SearchKey::Seen => writer.write_all(b"SEEN"),
            SearchKey::Since(date) => {
                writer.write_all(b"SINCE ")?;
                date.encode(writer)
            }
            SearchKey::Subject(astring) => {
                writer.write_all(b"SUBJECT ")?;
                astring.encode(writer)
            }
            SearchKey::Text(astring) => {
                writer.write_all(b"TEXT ")?;
                astring.encode(writer)
            }
            SearchKey::To(astring) => {
                writer.write_all(b"TO ")?;
                astring.encode(writer)
            }
            SearchKey::Unanswered => writer.write_all(b"UNANSWERED"),
            SearchKey::Undeleted => writer.write_all(b"UNDELETED"),
            SearchKey::Unflagged => writer.write_all(b"UNFLAGGED"),
            SearchKey::Unkeyword(flag_keyword) => {
                writer.write_all(b"UNKEYWORD ")?;
                flag_keyword.encode(writer)
            }
            SearchKey::Unseen => writer.write_all(b"UNSEEN"),
            SearchKey::Draft => writer.write_all(b"DRAFT"),
            SearchKey::Header(header_fld_name, astring) => {
                writer.write_all(b"HEADER ")?;
                header_fld_name.encode(writer)?;
                writer.write_all(b" ")?;
                astring.encode(writer)
            }
            SearchKey::Larger(number) => write!(writer, "LARGER {number}"),
            SearchKey::Not(search_key) => {
                writer.write_all(b"NOT ")?;
                search_key.encode(writer)
            }
            SearchKey::Or(search_key_a, search_key_b) => {
                writer.write_all(b"OR ")?;
                search_key_a.encode(writer)?;
                writer.write_all(b" ")?;
                search_key_b.encode(writer)
            }
            SearchKey::SentBefore(date) => {
                writer.write_all(b"SENTBEFORE ")?;
                date.encode(writer)
            }
            SearchKey::SentOn(date) => {
                writer.write_all(b"SENTON ")?;
                date.encode(writer)
            }
            SearchKey::SentSince(date) => {
                writer.write_all(b"SENTSINCE ")?;
                date.encode(writer)
            }
            SearchKey::Smaller(number) => write!(writer, "SMALLER {number}"),
            SearchKey::Uid(sequence_set) => {
                writer.write_all(b"UID ")?;
                sequence_set.encode(writer)
            }
            SearchKey::Undraft => writer.write_all(b"UNDRAFT"),
            SearchKey::SequenceSet(sequence_set) => sequence_set.encode(writer),
            SearchKey::And(search_keys) => {
                writer.write_all(b"(")?;
                join_serializable(search_keys.as_ref(), b" ", writer)?;
                writer.write_all(b")")
            }
        }
    }
}

impl Encode for SequenceSet {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        join_serializable(self.0.as_ref(), b",", writer)
    }
}

impl Encode for Sequence {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Sequence::Single(seq_no) => seq_no.encode(writer),
            Sequence::Range(from, to) => {
                from.encode(writer)?;
                writer.write_all(b":")?;
                to.encode(writer)
            }
        }
    }
}

impl Encode for SeqNo {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            SeqNo::Value(number) => write!(writer, "{number}"),
            SeqNo::Largest => writer.write_all(b"*"),
        }
    }
}

impl Encode for MyNaiveDate {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "\"{}\"", self.0.format("%d-%b-%Y"))
    }
}

impl<'a> Encode for MacroOrFetchAttributes<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            MacroOrFetchAttributes::Macro(m) => m.encode(writer),
            MacroOrFetchAttributes::FetchAttributes(attributes) => {
                if attributes.len() == 1 {
                    attributes[0].encode(writer)
                } else {
                    writer.write_all(b"(")?;
                    join_serializable(attributes.as_slice(), b" ", writer)?;
                    writer.write_all(b")")
                }
            }
        }
    }
}

impl Encode for Macro {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Macro::All => writer.write_all(b"ALL"),
            Macro::Fast => writer.write_all(b"FAST"),
            Macro::Full => writer.write_all(b"FULL"),
        }
    }
}

impl<'a> Encode for FetchAttribute<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            FetchAttribute::Body => writer.write_all(b"BODY"),
            FetchAttribute::BodyExt {
                section,
                partial,
                peek,
            } => {
                if *peek {
                    writer.write_all(b"BODY.PEEK[")?;
                } else {
                    writer.write_all(b"BODY[")?;
                }
                if let Some(section) = section {
                    section.encode(writer)?;
                }
                writer.write_all(b"]")?;
                if let Some((a, b)) = partial {
                    write!(writer, "<{a}.{b}>")?;
                }

                Ok(())
            }
            FetchAttribute::BodyStructure => writer.write_all(b"BODYSTRUCTURE"),
            FetchAttribute::Envelope => writer.write_all(b"ENVELOPE"),
            FetchAttribute::Flags => writer.write_all(b"FLAGS"),
            FetchAttribute::InternalDate => writer.write_all(b"INTERNALDATE"),
            FetchAttribute::Rfc822 => writer.write_all(b"RFC822"),
            FetchAttribute::Rfc822Header => writer.write_all(b"RFC822.HEADER"),
            FetchAttribute::Rfc822Size => writer.write_all(b"RFC822.SIZE"),
            FetchAttribute::Rfc822Text => writer.write_all(b"RFC822.TEXT"),
            FetchAttribute::Uid => writer.write_all(b"UID"),
        }
    }
}

impl<'a> Encode for Section<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Section::Part(part) => part.encode(writer),
            Section::Header(maybe_part) => match maybe_part {
                Some(part) => {
                    part.encode(writer)?;
                    writer.write_all(b".HEADER")
                }
                None => writer.write_all(b"HEADER"),
            },
            Section::HeaderFields(maybe_part, header_list) => {
                match maybe_part {
                    Some(part) => {
                        part.encode(writer)?;
                        writer.write_all(b".HEADER.FIELDS (")?;
                    }
                    None => writer.write_all(b"HEADER.FIELDS (")?,
                };
                join_serializable(header_list.as_ref(), b" ", writer)?;
                writer.write_all(b")")
            }
            Section::HeaderFieldsNot(maybe_part, header_list) => {
                match maybe_part {
                    Some(part) => {
                        part.encode(writer)?;
                        writer.write_all(b".HEADER.FIELDS.NOT (")?;
                    }
                    None => writer.write_all(b"HEADER.FIELDS.NOT (")?,
                };
                join_serializable(header_list.as_ref(), b" ", writer)?;
                writer.write_all(b")")
            }
            Section::Text(maybe_part) => match maybe_part {
                Some(part) => {
                    part.encode(writer)?;
                    writer.write_all(b".TEXT")
                }
                None => writer.write_all(b"TEXT"),
            },
            Section::Mime(part) => {
                part.encode(writer)?;
                writer.write_all(b".MIME")
            }
        }
    }
}

impl Encode for Part {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        join_serializable(self.0.as_ref(), b".", writer)
    }
}

impl Encode for NonZeroU32 {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "{self}")
    }
}

impl<'a> Encode for Capability<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Self::Imap4Rev1 => writer.write_all(b"IMAP4REV1"),
            Self::Auth(mechanism) => match mechanism {
                AuthMechanism::Plain => writer.write_all(b"AUTH=PLAIN"),
                AuthMechanism::Login => writer.write_all(b"AUTH=LOGIN"),
                AuthMechanism::Other(other) => {
                    writer.write_all(b"AUTH=")?;
                    other.encode(writer)
                }
            },
            #[cfg(feature = "starttls")]
            Self::LoginDisabled => writer.write_all(b"LOGINDISABLED"),
            #[cfg(feature = "starttls")]
            Self::StartTls => writer.write_all(b"STARTTLS"),
            #[cfg(feature = "ext_mailbox_referrals")]
            Self::MailboxReferrals => writer.write_all(b"MAILBOX-REFERRALS"),
            #[cfg(feature = "ext_login_referrals")]
            Self::LoginReferrals => writer.write_all(b"LOGIN-REFERRALS"),
            #[cfg(feature = "ext_sasl_ir")]
            Self::SaslIr => writer.write_all(b"SASL-IR"),
            #[cfg(feature = "ext_idle")]
            Self::Idle => writer.write_all(b"IDLE"),
            #[cfg(feature = "ext_enable")]
            Self::Enable => writer.write_all(b"ENABLE"),
            #[cfg(feature = "ext_compress")]
            Self::Compress { algorithm } => match algorithm {
                CompressionAlgorithm::Deflate => writer.write_all(b"COMPRESS=DEFLATE"),
            },
            #[cfg(feature = "ext_quota")]
            Self::Quota => writer.write_all(b"QUOTA"),
            #[cfg(feature = "ext_quota")]
            Self::QuotaRes(resource) => {
                writer.write_all(b"QUOTA=RES-")?;
                resource.encode(writer)
            }
            #[cfg(feature = "ext_quota")]
            Self::QuotaSet => writer.write_all(b"QUOTASET"),
            Self::Other(other) => other.inner.encode(writer),
        }
    }
}

// ----- Responses -----

impl<'a> Encode for Response<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Response::Status(status) => status.encode(writer),
            Response::Data(data) => data.encode(writer),
            Response::Continue(continue_request) => continue_request.encode(writer),
        }
    }
}

impl<'a> Encode for Greeting<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(b"* ")?;
        self.kind.encode(writer)?;
        writer.write_all(b" ")?;

        if let Some(ref code) = self.code {
            writer.write_all(b"[")?;
            code.encode(writer)?;
            writer.write_all(b"] ")?;
        }

        self.text.encode(writer)?;
        writer.write_all(b"\r\n")
    }
}

impl Encode for GreetingKind {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            GreetingKind::Ok => writer.write_all(b"OK"),
            GreetingKind::PreAuth => writer.write_all(b"PREAUTH"),
            GreetingKind::Bye => writer.write_all(b"BYE"),
        }
    }
}

impl<'a> Encode for Status<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        fn format_status(
            tag: &Option<Tag>,
            status: &str,
            code: &Option<Code>,
            comment: &Text,
            writer: &mut impl Write,
        ) -> std::io::Result<()> {
            match tag {
                Some(tag) => tag.encode(writer)?,
                None => writer.write_all(b"*")?,
            }
            writer.write_all(b" ")?;
            writer.write_all(status.as_bytes())?;
            writer.write_all(b" ")?;
            if let Some(code) = code {
                writer.write_all(b"[")?;
                code.encode(writer)?;
                writer.write_all(b"] ")?;
            }
            comment.encode(writer)?;
            writer.write_all(b"\r\n")
        }

        match self {
            Status::Ok { tag, code, text } => format_status(tag, "OK", code, text, writer),
            Status::No { tag, code, text } => format_status(tag, "NO", code, text, writer),
            Status::Bad { tag, code, text } => format_status(tag, "BAD", code, text, writer),
            Status::Bye { code, text } => format_status(&None, "BYE", code, text, writer),
        }
    }
}

impl<'a> Encode for Code<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Code::Alert => writer.write_all(b"ALERT"),
            Code::BadCharset { allowed } => {
                if allowed.is_empty() {
                    writer.write_all(b"BADCHARSET")
                } else {
                    writer.write_all(b"BADCHARSET (")?;
                    join_serializable(allowed, b" ", writer)?;
                    writer.write_all(b")")
                }
            }
            Code::Capability(caps) => {
                writer.write_all(b"CAPABILITY ")?;
                join_serializable(caps.as_ref(), b" ", writer)
            }
            Code::Parse => writer.write_all(b"PARSE"),
            Code::PermanentFlags(flags) => {
                writer.write_all(b"PERMANENTFLAGS (")?;
                join_serializable(flags, b" ", writer)?;
                writer.write_all(b")")
            }
            Code::ReadOnly => writer.write_all(b"READ-ONLY"),
            Code::ReadWrite => writer.write_all(b"READ-WRITE"),
            Code::TryCreate => writer.write_all(b"TRYCREATE"),
            Code::UidNext(next) => {
                writer.write_all(b"UIDNEXT ")?;
                next.encode(writer)
            }
            Code::UidValidity(validity) => {
                writer.write_all(b"UIDVALIDITY ")?;
                validity.encode(writer)
            }
            Code::Unseen(seq) => {
                writer.write_all(b"UNSEEN ")?;
                seq.encode(writer)
            }
            // RFC 2221
            #[cfg(any(feature = "ext_login_referrals", feature = "ext_mailbox_referrals"))]
            Code::Referral(url) => {
                writer.write_all(b"REFERRAL ")?;
                writer.write_all(url.as_bytes())
            }
            #[cfg(feature = "ext_compress")]
            Code::CompressionActive => writer.write_all(b"COMPRESSIONACTIVE"),
            #[cfg(feature = "ext_quota")]
            Code::OverQuota => writer.write_all(b"OVERQUOTA"),
            Code::Other(other, text) => match text {
                Some(text) => {
                    other.encode(writer)?;
                    writer.write_all(b" ")?;
                    text.encode(writer)
                }
                None => other.encode(writer),
            },
        }
    }
}

impl<'a> Encode for CodeOther<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.inner().encode(writer)
    }
}

impl<'a> Encode for Text<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(self.inner().as_bytes())
    }
}

impl<'a> Encode for Data<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Data::Capability(caps) => {
                writer.write_all(b"* CAPABILITY ")?;
                join_serializable(caps.as_ref(), b" ", writer)?;
            }
            Data::List {
                items,
                delimiter,
                mailbox,
            } => {
                writer.write_all(b"* LIST (")?;
                join_serializable(items, b" ", writer)?;
                writer.write_all(b") ")?;

                if let Some(delimiter) = delimiter {
                    writer.write_all(b"\"")?;
                    delimiter.encode(writer)?;
                    writer.write_all(b"\"")?;
                } else {
                    writer.write_all(b"NIL")?;
                }
                writer.write_all(b" ")?;
                mailbox.encode(writer)?;
            }
            Data::Lsub {
                items,
                delimiter,
                mailbox,
            } => {
                writer.write_all(b"* LSUB (")?;
                join_serializable(items, b" ", writer)?;
                writer.write_all(b") ")?;

                if let Some(delimiter) = delimiter {
                    writer.write_all(b"\"")?;
                    delimiter.encode(writer)?;
                    writer.write_all(b"\"")?;
                } else {
                    writer.write_all(b"NIL")?;
                }
                writer.write_all(b" ")?;
                mailbox.encode(writer)?;
            }
            Data::Status {
                mailbox,
                attributes,
            } => {
                writer.write_all(b"* STATUS ")?;
                mailbox.encode(writer)?;
                writer.write_all(b" (")?;
                join_serializable(attributes, b" ", writer)?;
                writer.write_all(b")")?;
            }
            Data::Search(seqs) => {
                if seqs.is_empty() {
                    writer.write_all(b"* SEARCH")?;
                } else {
                    writer.write_all(b"* SEARCH ")?;
                    join_serializable(seqs, b" ", writer)?;
                }
            }
            Data::Flags(flags) => {
                writer.write_all(b"* FLAGS (")?;
                join_serializable(flags, b" ", writer)?;
                writer.write_all(b")")?;
            }
            Data::Exists(count) => write!(writer, "* {count} EXISTS")?,
            Data::Recent(count) => write!(writer, "* {count} RECENT")?,
            Data::Expunge(msg) => write!(writer, "* {msg} EXPUNGE")?,
            Data::Fetch {
                seq_or_uid,
                attributes,
            } => {
                write!(writer, "* {seq_or_uid} FETCH (")?;
                join_serializable(attributes.as_ref(), b" ", writer)?;
                writer.write_all(b")")?;
            }
            #[cfg(feature = "ext_enable")]
            Data::Enabled { capabilities } => {
                write!(writer, "* ENABLED")?;

                for cap in capabilities {
                    writer.write_all(b" ")?;
                    cap.encode(writer)?;
                }
            }
            #[cfg(feature = "ext_quota")]
            Data::Quota { root, quotas } => {
                writer.write_all(b"* QUOTA ")?;
                root.encode(writer)?;
                writer.write_all(b" (")?;
                join_serializable(quotas.as_ref(), b" ", writer)?;
                writer.write_all(b")")?;
            }
            #[cfg(feature = "ext_quota")]
            Data::QuotaRoot { mailbox, roots } => {
                writer.write_all(b"* QUOTAROOT ")?;
                mailbox.encode(writer)?;
                for root in roots {
                    writer.write_all(b" ")?;
                    root.encode(writer)?;
                }
            }
        }

        writer.write_all(b"\r\n")
    }
}

impl<'a> Encode for FlagNameAttribute<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Self::Noinferiors => writer.write_all(b"\\Noinferiors"),
            Self::Noselect => writer.write_all(b"\\Noselect"),
            Self::Marked => writer.write_all(b"\\Marked"),
            Self::Unmarked => writer.write_all(b"\\Unmarked"),
            Self::Extension(atom) => {
                writer.write_all(b"\\")?;
                atom.encode(writer)
            }
        }
    }
}

impl Encode for QuotedChar {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self.inner() {
            '\\' => writer.write_all(b"\\\\"),
            '"' => writer.write_all(b"\\\""),
            other => writer.write_all(&[*other as u8]),
        }
    }
}

impl Encode for StatusAttributeValue {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Self::Messages(count) => {
                writer.write_all(b"MESSAGES ")?;
                count.encode(writer)
            }
            Self::Recent(count) => {
                writer.write_all(b"RECENT ")?;
                count.encode(writer)
            }
            Self::UidNext(next) => {
                writer.write_all(b"UIDNEXT ")?;
                next.encode(writer)
            }
            Self::UidValidity(identifier) => {
                writer.write_all(b"UIDVALIDITY ")?;
                identifier.encode(writer)
            }
            Self::Unseen(count) => {
                writer.write_all(b"UNSEEN ")?;
                count.encode(writer)
            }
            #[cfg(feature = "ext_quota")]
            Self::Deleted(count) => {
                writer.write_all(b"DELETED ")?;
                count.encode(writer)
            }
            #[cfg(feature = "ext_quota")]
            Self::DeletedStorage(count) => {
                writer.write_all(b"DELETED-STORAGE ")?;
                count.encode(writer)
            }
        }
    }
}

impl<'a> Encode for FetchAttributeValue<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Self::BodyExt {
                section,
                origin,
                data,
            } => {
                writer.write_all(b"BODY[")?;
                if let Some(section) = section {
                    section.encode(writer)?;
                }
                writer.write_all(b"]")?;
                if let Some(origin) = origin {
                    write!(writer, "<{origin}>")?;
                }
                writer.write_all(b" ")?;
                data.encode(writer)
            }
            // FIXME: do not return body-ext-1part and body-ext-mpart here
            Self::Body(body) => {
                writer.write_all(b"BODY ")?;
                body.encode(writer)
            }
            Self::BodyStructure(body) => {
                writer.write_all(b"BODYSTRUCTURE ")?;
                body.encode(writer)
            }
            Self::Envelope(envelope) => {
                writer.write_all(b"ENVELOPE ")?;
                envelope.encode(writer)
            }
            Self::Flags(flags) => {
                writer.write_all(b"FLAGS (")?;
                join_serializable(flags, b" ", writer)?;
                writer.write_all(b")")
            }
            Self::InternalDate(datetime) => {
                writer.write_all(b"INTERNALDATE ")?;
                datetime.encode(writer)
            }
            Self::Rfc822(nstring) => {
                writer.write_all(b"RFC822 ")?;
                nstring.encode(writer)
            }
            Self::Rfc822Header(nstring) => {
                writer.write_all(b"RFC822.HEADER ")?;
                nstring.encode(writer)
            }
            Self::Rfc822Size(size) => write!(writer, "RFC822.SIZE {size}"),
            Self::Rfc822Text(nstring) => {
                writer.write_all(b"RFC822.TEXT ")?;
                nstring.encode(writer)
            }
            Self::Uid(uid) => write!(writer, "UID {uid}"),
        }
    }
}

impl<'a> Encode for NString<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match &self.inner {
            Some(imap_str) => imap_str.encode(writer),
            None => writer.write_all(b"NIL"),
        }
    }
}

impl<'a> Encode for BodyStructure<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(b"(")?;
        match self {
            BodyStructure::Single { body, extension } => {
                body.encode(writer)?;
                if let Some(extension) = extension {
                    writer.write_all(b" ")?;
                    extension.encode(writer)?;
                }
            }
            BodyStructure::Multi {
                bodies,
                subtype,
                extension_data,
            } => {
                for body in &bodies.inner {
                    body.encode(writer)?;
                }
                writer.write_all(b" ")?;
                subtype.encode(writer)?;

                if let Some(extension) = extension_data {
                    writer.write_all(b" ")?;
                    extension.encode(writer)?;
                }
            }
        }
        writer.write_all(b")")
    }
}

impl<'a> Encode for Body<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self.specific {
            SpecificFields::Basic {
                ref type_,
                ref subtype,
            } => {
                type_.encode(writer)?;
                writer.write_all(b" ")?;
                subtype.encode(writer)?;
                writer.write_all(b" ")?;
                self.basic.encode(writer)
            }
            SpecificFields::Message {
                ref envelope,
                ref body_structure,
                number_of_lines,
            } => {
                writer.write_all(b"\"MESSAGE\" \"RFC822\" ")?;
                self.basic.encode(writer)?;
                writer.write_all(b" ")?;
                envelope.encode(writer)?;
                writer.write_all(b" ")?;
                body_structure.encode(writer)?;
                writer.write_all(b" ")?;
                write!(writer, "{number_of_lines}")
            }
            SpecificFields::Text {
                ref subtype,
                number_of_lines,
            } => {
                writer.write_all(b"\"TEXT\" ")?;
                subtype.encode(writer)?;
                writer.write_all(b" ")?;
                self.basic.encode(writer)?;
                writer.write_all(b" ")?;
                write!(writer, "{number_of_lines}")
            }
        }
    }
}

impl<'a> Encode for BasicFields<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        List1AttributeValueOrNil(&self.parameter_list).encode(writer)?;
        writer.write_all(b" ")?;
        self.id.encode(writer)?;
        writer.write_all(b" ")?;
        self.description.encode(writer)?;
        writer.write_all(b" ")?;
        self.content_transfer_encoding.encode(writer)?;
        writer.write_all(b" ")?;
        write!(writer, "{}", self.size)
    }
}

impl<'a> Encode for Envelope<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(b"(")?;
        self.date.encode(writer)?;
        writer.write_all(b" ")?;
        self.subject.encode(writer)?;
        writer.write_all(b" ")?;
        List1OrNil(&self.from, b"").encode(writer)?;
        writer.write_all(b" ")?;
        List1OrNil(&self.sender, b"").encode(writer)?;
        writer.write_all(b" ")?;
        List1OrNil(&self.reply_to, b"").encode(writer)?;
        writer.write_all(b" ")?;
        List1OrNil(&self.to, b"").encode(writer)?;
        writer.write_all(b" ")?;
        List1OrNil(&self.cc, b"").encode(writer)?;
        writer.write_all(b" ")?;
        List1OrNil(&self.bcc, b"").encode(writer)?;
        writer.write_all(b" ")?;
        self.in_reply_to.encode(writer)?;
        writer.write_all(b" ")?;
        self.message_id.encode(writer)?;
        writer.write_all(b")")
    }
}

impl<'a> Encode for Address<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(b"(")?;
        self.name.encode(writer)?;
        writer.write_all(b" ")?;
        self.adl.encode(writer)?;
        writer.write_all(b" ")?;
        self.mailbox.encode(writer)?;
        writer.write_all(b" ")?;
        self.host.encode(writer)?;
        writer.write_all(b")")?;

        Ok(())
    }
}

impl<'a> Encode for SinglePartExtensionData<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.md5.encode(writer)?;
        if let Some(ref dsp) = self.disposition {
            writer.write_all(b" ")?;

            match dsp {
                Some((s, param)) => {
                    writer.write_all(b"(")?;
                    s.encode(writer)?;
                    writer.write_all(b" ")?;
                    List1AttributeValueOrNil(param).encode(writer)?;
                    writer.write_all(b")")?;
                }
                None => writer.write_all(b"NIL")?,
            }

            if let Some(ref lang) = self.language {
                writer.write_all(b" ")?;
                List1OrNil(lang, b" ").encode(writer)?;

                if let Some(ref loc) = self.location {
                    writer.write_all(b" ")?;
                    loc.encode(writer)?;

                    if !self.extension.is_empty() {
                        // FIXME: Extension includes the SP for now, as it is unparsed.
                        //writer.write_all(b" ")?;
                        writer.write_all(&self.extension)?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl<'a> Encode for MultiPartExtensionData<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        List1AttributeValueOrNil(&self.parameter_list).encode(writer)?;

        if let Some(ref dsp) = self.disposition {
            writer.write_all(b" ")?;

            match dsp {
                Some((s, param)) => {
                    writer.write_all(b"(")?;
                    s.encode(writer)?;
                    writer.write_all(b" ")?;
                    List1AttributeValueOrNil(param).encode(writer)?;
                    writer.write_all(b")")?;
                }
                None => writer.write_all(b"NIL")?,
            }

            if let Some(ref lang) = self.language {
                writer.write_all(b" ")?;
                List1OrNil(lang, b" ").encode(writer)?;

                if let Some(ref loc) = self.location {
                    writer.write_all(b" ")?;
                    loc.encode(writer)?;

                    if !self.extension.is_empty() {
                        // FIXME: Extension includes the SP for now, as it is unparsed.
                        //writer.write_all(b" ");
                        writer.write_all(&self.extension)?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Encode for DateTime<FixedOffset> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "\"{}\"", self.format("%d-%b-%Y %H:%M:%S %z"))
    }
}

impl<'a> Encode for Continue<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Continue::Basic { code, text } => match code {
                Some(ref code) => {
                    writer.write_all(b"+ [")?;
                    code.encode(writer)?;
                    writer.write_all(b"] ")?;
                    text.encode(writer)?;
                    writer.write_all(b"\r\n")
                }
                None => {
                    writer.write_all(b"+ ")?;
                    text.encode(writer)?;
                    writer.write_all(b"\r\n")
                }
            },
            // TODO: Is this correct when data is empty?
            Continue::Base64(data) => {
                writer.write_all(b"+ ")?;
                writer.write_all(base64.encode(data).as_bytes())?;
                writer.write_all(b"\r\n")
            }
        }
    }
}

pub(crate) mod utils {
    use std::io::Write;

    use super::Encode;

    pub(crate) struct List1OrNil<'a, T>(pub(crate) &'a Vec<T>, pub(crate) &'a [u8]);

    pub(crate) struct List1AttributeValueOrNil<'a, T>(pub(crate) &'a Vec<(T, T)>);

    pub(crate) fn join_serializable<I: Encode>(
        elements: &[I],
        sep: &[u8],
        writer: &mut impl Write,
    ) -> std::io::Result<()> {
        if let Some((last, head)) = elements.split_last() {
            for item in head {
                item.encode(writer)?;
                writer.write_all(sep)?;
            }

            last.encode(writer)
        } else {
            Ok(())
        }
    }

    impl<'a, T> Encode for List1OrNil<'a, T>
    where
        T: Encode,
    {
        fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
            if let Some((last, head)) = self.0.split_last() {
                writer.write_all(b"(")?;

                for item in head {
                    item.encode(writer)?;
                    writer.write_all(self.1)?;
                }

                last.encode(writer)?;

                writer.write_all(b")")
            } else {
                writer.write_all(b"NIL")
            }
        }
    }

    impl<'a, T> Encode for List1AttributeValueOrNil<'a, T>
    where
        T: Encode,
    {
        fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
            if let Some((last, head)) = self.0.split_last() {
                writer.write_all(b"(")?;

                for (attribute, value) in head {
                    attribute.encode(writer)?;
                    writer.write_all(b" ")?;
                    value.encode(writer)?;
                    writer.write_all(b" ")?;
                }

                let (attribute, value) = last;
                attribute.encode(writer)?;
                writer.write_all(b" ")?;
                value.encode(writer)?;

                writer.write_all(b")")
            } else {
                writer.write_all(b"NIL")
            }
        }
    }
}
