use std::{io::Write, num::NonZeroU32};

use base64::encode as b64encode;
use chrono::{DateTime, FixedOffset, NaiveDate};

use crate::{
    codec::utils::{join_serializable, List1AttributeValueOrNil, List1OrNil},
    types::{
        address::Address,
        body::{
            BasicFields, Body, BodyStructure, MultiPartExtensionData, SinglePartExtensionData,
            SpecificFields,
        },
        command::{Command, CommandBody, SearchKey, StatusAttribute},
        core::{AString, Atom, Charset, IString, NString, Tag, Text},
        datetime::{MyDateTime, MyNaiveDate},
        envelope::Envelope,
        fetch_attributes::{FetchAttribute, Macro, MacroOrFetchAttributes, Part, Section},
        flag::{Flag, FlagNameAttribute, StoreResponse, StoreType},
        mailbox::{ListMailbox, Mailbox, MailboxOther},
        response::{
            Capability, Code, Continuation, Data, MessageAttribute, Response, Status,
            StatusAttributeValue,
        },
        sequence::{SeqNo, Sequence, SequenceSet},
        AuthMechanism, AuthMechanismOther, CompressionAlgorithm,
    },
    utils::escape_quoted,
};

pub trait Encode {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()>;
}

// ----- Command -----

impl Encode for Command {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.tag.encode(writer)?;
        writer.write_all(b" ")?;
        self.body.encode(writer)?;
        writer.write_all(b"\r\n")
    }
}

impl Encode for Tag {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(self.0.as_bytes())
    }
}

impl Encode for CommandBody {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            CommandBody::Capability => writer.write_all(b"CAPABILITY"),
            CommandBody::Noop => writer.write_all(b"NOOP"),
            CommandBody::Logout => writer.write_all(b"LOGOUT"),
            CommandBody::StartTLS => writer.write_all(b"STARTTLS"),
            CommandBody::Authenticate {
                mechanism,
                initial_response,
            } => {
                writer.write_all(b"AUTHENTICATE")?;
                writer.write_all(b" ")?;
                mechanism.encode(writer)?;

                if let Some(ir) = initial_response {
                    writer.write_all(b" ")?;

                    // RFC 4959 (https://datatracker.ietf.org/doc/html/rfc4959#section-3)
                    // "To send a zero-length initial response, the client MUST send a single pad character ("=").
                    // This indicates that the response is present, but is a zero-length string."
                    if ir.is_empty() {
                        writer.write_all(b"=")?;
                    } else {
                        writer.write_all(b64encode(ir).as_bytes())?;
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
                writer.write_all(format!("{{{}}}\r\n", message.len()).as_bytes())?;
                writer.write_all(message)
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
                    writer.write_all(b" ")?;
                    write!(writer, "CHARSET {}", charset)?;
                }
                writer.write_all(b" ")?;
                if let SearchKey::And(search_keys) = criteria {
                    join_serializable(search_keys, b" ", writer) // TODO: use List1?
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
            CommandBody::Idle => writer.write_all(b"IDLE"),
            CommandBody::Enable { capabilities } => {
                writer.write_all(b"ENABLE ")?;
                join_serializable(capabilities, b" ", writer)
            }
            CommandBody::Compress { algorithm } => {
                writer.write_all(b"COMPRESS ")?;
                algorithm.encode(writer)
            }
        }
    }
}

impl Encode for AuthMechanism {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match &self {
            AuthMechanism::Plain => writer.write_all(b"PLAIN"),
            AuthMechanism::Login => writer.write_all(b"LOGIN"),
            AuthMechanism::Other(other) => other.encode(writer),
        }
    }
}

impl Encode for AuthMechanismOther {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.0.encode(writer)
    }
}

impl Encode for AString {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            AString::Atom(atom) => writer.write_all(atom.0.as_bytes()), // FIXME: use encode
            AString::String(imap_str) => imap_str.encode(writer),
        }
    }
}

impl Encode for Atom {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(self.0.as_bytes())
    }
}

impl Encode for IString {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Self::Literal(val) => {
                write!(writer, "{{{}}}\r\n", val.len())?;
                writer.write_all(val)
            }
            Self::Quoted(val) => write!(writer, "\"{}\"", escape_quoted(&val.0)), // FIXME: use encode
        }
    }
}

impl Encode for Mailbox {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Mailbox::Inbox => writer.write_all(b"INBOX"),
            Mailbox::Other(other) => other.encode(writer),
        }
    }
}

impl Encode for MailboxOther {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.0.encode(writer)
    }
}

impl Encode for ListMailbox {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            ListMailbox::Token(str) => writer.write_all(str.as_bytes()), // TODO: use encode()
            ListMailbox::String(imap_str) => imap_str.encode(writer),
        }
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
        }
    }
}

impl Encode for Flag {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "{}", self)
    }
}

impl Encode for MyDateTime {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.0.encode(writer)
    }
}

impl Encode for Charset {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        // FIXME(perf): conversion calls should not
        //              be requires for serialization.
        writer.write_all(self.to_string().as_bytes())
    }
}

impl Encode for SearchKey {
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
            SearchKey::Larger(number) => write!(writer, "LARGER {}", number),
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
            SearchKey::Smaller(number) => write!(writer, "SMALLER {}", number),
            SearchKey::Uid(sequence_set) => {
                writer.write_all(b"UID ")?;
                sequence_set.encode(writer)
            }
            SearchKey::Undraft => writer.write_all(b"UNDRAFT"),
            SearchKey::SequenceSet(sequence_set) => sequence_set.encode(writer),
            SearchKey::And(search_keys) => {
                writer.write_all(b"(")?;
                join_serializable(search_keys, b" ", writer)?;
                writer.write_all(b")")
            }
        }
    }
}

impl Encode for SequenceSet {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        join_serializable(&self.0, b",", writer)
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
            SeqNo::Value(number) => write!(writer, "{}", number),
            SeqNo::Largest => writer.write_all(b"*"),
        }
    }
}

impl Encode for MyNaiveDate {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.0.encode(writer)
    }
}

impl Encode for NaiveDate {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "\"{}\"", self.format("%d-%b-%Y"))
    }
}

impl Encode for MacroOrFetchAttributes {
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

impl Encode for FetchAttribute {
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
                    write!(writer, "<{}.{}>", a, b)?;
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

impl Encode for Section {
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
                join_serializable(header_list, b" ", writer)?;
                writer.write_all(b")")
            }
            Section::HeaderFieldsNot(maybe_part, header_list) => {
                match maybe_part {
                    Some(part) => {
                        part.encode(writer)?;
                        writer.write_all(b".HEADER.FIELDS.NOT (")?;
                    }
                    None => writer.write_all(b"HEADER.FIElDS.NOT (")?,
                };
                join_serializable(header_list, b" ", writer)?;
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
        join_serializable(&self.0, b".", writer)
    }
}

impl Encode for NonZeroU32 {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "{}", self)
    }
}

impl Encode for Capability {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "{}", self)
    }
}

impl Encode for CompressionAlgorithm {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            CompressionAlgorithm::Deflate => writer.write_all(b"DEFLATE"),
        }
    }
}

// ----- Responses -----

impl Encode for Response {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Response::Status(status) => status.encode(writer),
            Response::Data(data) => data.encode(writer),
            Response::Continuation(continuation) => continuation.encode(writer),
        }
    }
}

impl Encode for Status {
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
                write!(writer, "[{}] ", code)?;
            }
            comment.encode(writer)?;
            writer.write_all(b"\r\n")
        }

        match self {
            Status::Ok { tag, code, text } => format_status(tag, "OK", code, text, writer),
            Status::No { tag, code, text } => format_status(tag, "NO", code, text, writer),
            Status::Bad { tag, code, text } => format_status(tag, "BAD", code, text, writer),
            Status::PreAuth { code, text } => format_status(&None, "PREAUTH", code, text, writer),
            Status::Bye { code, text } => format_status(&None, "BYE", code, text, writer),
        }
    }
}

impl Encode for Code {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "{}", self)
    }
}

impl Encode for Text {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(self.0.as_bytes())
    }
}

impl Encode for Data {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Data::Capability(caps) => {
                writer.write_all(b"* CAPABILITY ")?;
                join_serializable(caps, b" ", writer)?;
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
                    // TODO: newtype Delimiter?
                    write!(writer, "\"{}\"", escape_quoted(&delimiter.to_string()))?;
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
                    // TODO: newtype Delimiter?
                    write!(writer, "\"{}\"", escape_quoted(&delimiter.to_string()))?;
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
            Data::Exists(count) => write!(writer, "* {} EXISTS", count)?,
            Data::Recent(count) => write!(writer, "* {} RECENT", count)?,
            Data::Expunge(msg) => write!(writer, "* {} EXPUNGE", msg)?,
            Data::Fetch {
                seq_or_uid,
                attributes,
            } => {
                write!(writer, "* {} FETCH (", seq_or_uid)?;
                join_serializable(attributes, b" ", writer)?;
                writer.write_all(b")")?;
            }
            Data::Enabled { capabilities } => {
                write!(writer, "* ENABLED ")?;
                join_serializable(capabilities, b" ", writer)?;
            }
        }

        writer.write_all(b"\r\n")
    }
}

impl Encode for FlagNameAttribute {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "{}", self)
    }
}

impl Encode for StatusAttributeValue {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "{}", self)
    }
}

impl Encode for MessageAttribute {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        use MessageAttribute::*;

        match self {
            BodyExt {
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
                    write!(writer, "<{}>", origin)?;
                }
                writer.write_all(b" ")?;
                data.encode(writer)
            }
            // FIXME: do not return body-ext-1part and body-ext-mpart here
            Body(body) => {
                writer.write_all(b"BODY ")?;
                body.encode(writer)
            }
            BodyStructure(body) => {
                writer.write_all(b"BODYSTRUCTURE ")?;
                body.encode(writer)
            }
            Envelope(envelope) => {
                writer.write_all(b"ENVELOPE ")?;
                envelope.encode(writer)
            }
            Flags(flags) => {
                writer.write_all(b"FLAGS (")?;
                join_serializable(flags, b" ", writer)?;
                writer.write_all(b")")
            }
            InternalDate(datetime) => {
                writer.write_all(b"INTERNALDATE ")?;
                datetime.encode(writer)
            }
            Rfc822(nstring) => {
                writer.write_all(b"RFC822 ")?;
                nstring.encode(writer)
            }
            Rfc822Header(nstring) => {
                writer.write_all(b"RFC822.HEADER ")?;
                nstring.encode(writer)
            }
            Rfc822Size(size) => write!(writer, "RFC822.SIZE {}", size),
            Rfc822Text(nstring) => {
                writer.write_all(b"RFC822.TEXT ")?;
                nstring.encode(writer)
            }
            Uid(uid) => write!(writer, "UID {}", uid),
        }
    }
}

impl Encode for NString {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match &self.0 {
            Some(imap_str) => imap_str.encode(writer),
            None => writer.write_all(b"NIL"),
        }
    }
}

impl Encode for BodyStructure {
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
                for body in bodies {
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

impl Encode for Body {
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
                write!(writer, "{}", number_of_lines)
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
                write!(writer, "{}", number_of_lines)
            }
        }
    }
}

impl Encode for BasicFields {
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

impl Encode for Envelope {
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

impl Encode for Address {
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

impl Encode for SinglePartExtensionData {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.md5.encode(writer)?;
        if let Some(ref dsp) = self.disposition {
            writer.write_all(b" ")?;

            match dsp {
                Some((s, param)) => {
                    writer.write_all(b"(")?;
                    s.encode(writer)?;
                    writer.write_all(b" ")?;
                    List1AttributeValueOrNil(&param).encode(writer)?;
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
                        //writer.write_all(b" ")?; // TODO: Extension includes the SP for now, as it is unparsed.
                        writer.write_all(&self.extension)?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Encode for MultiPartExtensionData {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        List1AttributeValueOrNil(&self.parameter_list).encode(writer)?;

        if let Some(ref dsp) = self.disposition {
            writer.write_all(b" ")?;

            match dsp {
                Some((s, param)) => {
                    writer.write_all(b"(")?;
                    s.encode(writer)?;
                    writer.write_all(b" ")?;
                    List1AttributeValueOrNil(&param).encode(writer)?;
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
                        //writer.write_all(b" "); // TODO: Extension includes the SP for now, as it is unparsed.
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

impl Encode for Continuation {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Continuation::Basic { code, text } => match code {
                Some(ref code) => write!(writer, "+ [{}] {}\r\n", code, text),
                None => write!(writer, "+ {}\r\n", text),
            },
            // TODO: Is this correct when data is empty?
            Continuation::Base64(data) => write!(writer, "+ {}\r\n", base64::encode(data)),
        }
    }
}

// ----- Unused -----

impl Encode for u32 {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "{}", self)
    }
}

pub(crate) mod utils {
    use std::io::Write;

    use crate::Encode;

    pub(crate) struct List1OrNil<'a, T>(pub(crate) &'a Vec<T>, pub(crate) &'a [u8]);

    pub(crate) struct List1AttributeValueOrNil<'a, T>(pub(crate) &'a Vec<(T, T)>);

    pub(crate) fn join<T: std::fmt::Display>(elements: &[T], sep: &str) -> String {
        elements
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>()
            .join(sep)
    }

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
