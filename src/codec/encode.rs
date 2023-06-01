use std::{io::Write, num::NonZeroU32};

use base64::{engine::general_purpose::STANDARD as base64, Engine};
use chrono::{DateTime as ChronoDateTime, FixedOffset};
use utils::{join_serializable, List1AttributeValueOrNil, List1OrNil};

#[cfg(feature = "ext_compress")]
use crate::extensions::compress::CompressionAlgorithm;
use crate::{
    auth::{AuthMechanism, AuthMechanismOther, AuthenticateData},
    body::{
        BasicFields, Body, BodyExtension, BodyStructure, Disposition, Language, Location,
        MultiPartExtensionData, SinglePartExtensionData, SpecificFields,
    },
    command::{Command, CommandBody},
    core::{
        AString, Atom, AtomExt, Charset, IString, Literal, NString, Quoted, QuotedChar, Tag, Text,
    },
    datetime::{DateTime, NaiveDate},
    envelope::{Address, Envelope},
    fetch::{FetchAttribute, FetchAttributeValue, Macro, MacroOrFetchAttributes},
    flag::{Flag, FlagExtension, FlagFetch, FlagNameAttribute, FlagPerm, StoreResponse, StoreType},
    mailbox::{ListCharString, ListMailbox, Mailbox, MailboxOther},
    response::{
        Capability, Code, CodeOther, Continue, Data, Greeting, GreetingKind, Response, Status,
    },
    search::SearchKey,
    section::{Part, Section},
    sequence::{SeqOrUid, Sequence, SequenceSet},
    status::{StatusAttribute, StatusAttributeValue},
    utils::escape_quoted,
};

pub trait Encode {
    /// Create an [`Encoder`] for this message.
    fn encode(&self) -> Encoder;
}

#[allow(missing_debug_implementations)]
/// Message encoder.
///
/// This encoder facilitates the implementation of IMAP client- and server implementations by
/// yielding the encoding of a message through [`Action`]s. This is required, because the usage of
/// literals (and some other types) may change the IMAP message flow. Thus, in many cases, it is an
/// error to just "dump" a message and send it over the network.
///
/// # Example
///
/// ```rust
/// use imap_codec::{
///     codec::{Action, Encode},
///     command::{Command, CommandBody},
/// };
///
/// let cmd = Command::new("A", CommandBody::login("alice", "pass").unwrap()).unwrap();
///
/// for action in cmd.encode() {
///     match action {
///         Action::Send { data } => {}
///         Action::RecvContinuationRequest => {}
///         Action::Unknown => {}
///     }
/// }
/// ```
pub struct Encoder {
    items: Vec<Action>,
}

impl Encoder {
    /// Dump the (remaining) encoded data without being guided by [`Action`]s.
    ///
    /// Note: This method should (likely) not be used in a real implementation. It is often not
    /// possible to send a full message at once in IMAP. Notably, a client needs to wait for
    /// continuation requests before sending literals.
    ///
    /// Prefer to iterate over the encoder and follow the yielded actions:
    ///
    /// ```rust
    /// use imap_codec::{
    ///     codec::{Action, Encode},
    ///     command::{Command, CommandBody},
    /// };
    ///
    /// let cmd = Command::new("A", CommandBody::login("alice", "pass").unwrap()).unwrap();
    ///
    /// for action in cmd.encode() {
    ///     match action {
    ///         Action::Send { data } => {}
    ///         Action::RecvContinuationRequest => {}
    ///         Action::Unknown => {}
    ///     }
    /// }
    /// ```
    pub fn dump(self) -> Vec<u8> {
        let mut out = Vec::new();

        for action in self.items {
            match action {
                Action::Send { mut data } => out.append(&mut data),
                Action::RecvContinuationRequest | Action::Unknown => {
                    // Nothing to do.
                }
            }
        }

        out
    }
}

impl Iterator for Encoder {
    type Item = Action;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.items.is_empty() {
            Some(self.items.remove(0))
        } else {
            None
        }
    }
}

/// The intended action of a client or server.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    /// Send this data over the network.
    Send { data: Vec<u8> },
    /// (Maybe) wait for a continuation request.
    ///
    /// Note: This action MUST be ignored by a server.
    RecvContinuationRequest,
    /// Encoding of a non-supported type may (or may not) require further action.
    Unknown,
}

//--------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EncodeContext {
    accumulator: Vec<u8>,
    items: Vec<Action>,
}

impl EncodeContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_items(mut self) -> Vec<Action> {
        if !self.accumulator.is_empty() {
            self.items.push(Action::Send {
                data: self.accumulator,
            })
        }
        // Not needed, as it is dropped anyway.
        // self.accumulator.clear();

        self.items
    }

    pub fn dump(mut self) -> Vec<u8> {
        if !self.accumulator.is_empty() {
            self.items.push(Action::Send {
                data: self.accumulator.clone(),
            })
        }
        self.accumulator.clear();

        self.items
            .into_iter()
            .fold(vec![], |mut acc, action| match action {
                Action::Send { mut data } => {
                    acc.append(&mut data);
                    acc
                }
                Action::RecvContinuationRequest | Action::Unknown => acc,
            })
    }

    pub fn recv_continuation_request(&mut self) {
        self.items.push(Action::Send {
            data: self.accumulator.clone(),
        });
        self.items.push(Action::RecvContinuationRequest);
        self.accumulator.clear();
    }

    pub fn unknown(&mut self) {
        self.items.push(Action::Send {
            data: self.accumulator.clone(),
        });
        self.items.push(Action::Unknown);
        self.accumulator.clear();
    }
}

impl Write for EncodeContext {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.accumulator.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<T> Encode for T
where
    T: CoreEncode,
{
    fn encode(&self) -> Encoder {
        let mut encode_context = EncodeContext::new();
        T::core_encode(self, &mut encode_context).unwrap();

        Encoder {
            items: encode_context.into_items(),
        }
    }
}

// -------------------------------------------------------------------------------------------------

pub trait CoreEncode {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()>;
}

// ----- Primitive ---------------------------------------------------------------------------------

impl CoreEncode for u32 {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        writer.write_all(self.to_string().as_bytes())
    }
}

impl CoreEncode for u64 {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        writer.write_all(self.to_string().as_bytes())
    }
}

// ----- Command -----------------------------------------------------------------------------------

impl<'a> CoreEncode for Command<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        self.tag.core_encode(writer)?;
        writer.write_all(b" ")?;
        self.body.core_encode(writer)?;
        writer.write_all(b"\r\n")?;

        // Note: We need to do this here because we must do it after the final \r\n was sent.
        match &self.body {
            #[cfg(not(feature = "ext_sasl_ir"))]
            CommandBody::Authenticate { .. } => {
                writer.recv_continuation_request();
            }
            #[cfg(feature = "ext_sasl_ir")]
            CommandBody::Authenticate {
                mechanism,
                initial_response,
            } => {
                if initial_response.is_some() {
                    match mechanism {
                        AuthMechanism::Plain => {
                            // Nothing to wait for here.
                        }
                        AuthMechanism::Login => {
                            writer.recv_continuation_request();
                        }
                        _ => writer.unknown(),
                    }
                } else {
                    writer.recv_continuation_request();
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl<'a> CoreEncode for Tag<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        writer.write_all(self.inner().as_bytes())
    }
}

impl<'a> CoreEncode for CommandBody<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
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
                mechanism.core_encode(writer)?;

                #[cfg(feature = "ext_sasl_ir")]
                if let Some(ir) = initial_response {
                    writer.write_all(b" ")?;

                    // RFC 4959 (https://datatracker.ietf.org/doc/html/rfc4959#section-3)
                    // "To send a zero-length initial response, the client MUST send a single pad character ("=").
                    // This indicates that the response is present, but is a zero-length string."
                    if ir.declassify().is_empty() {
                        writer.write_all(b"=")?;
                    } else {
                        writer.write_all(base64.encode(ir.declassify()).as_bytes())?;
                    };
                };

                Ok(())
            }
            CommandBody::Login { username, password } => {
                writer.write_all(b"LOGIN")?;
                writer.write_all(b" ")?;
                username.core_encode(writer)?;
                writer.write_all(b" ")?;
                password.declassify().core_encode(writer)
            }
            CommandBody::Select { mailbox } => {
                writer.write_all(b"SELECT")?;
                writer.write_all(b" ")?;
                mailbox.core_encode(writer)
            }
            #[cfg(feature = "ext_unselect")]
            CommandBody::Unselect => writer.write_all(b"UNSELECT"),
            CommandBody::Examine { mailbox } => {
                writer.write_all(b"EXAMINE")?;
                writer.write_all(b" ")?;
                mailbox.core_encode(writer)
            }
            CommandBody::Create { mailbox } => {
                writer.write_all(b"CREATE")?;
                writer.write_all(b" ")?;
                mailbox.core_encode(writer)
            }
            CommandBody::Delete { mailbox } => {
                writer.write_all(b"DELETE")?;
                writer.write_all(b" ")?;
                mailbox.core_encode(writer)
            }
            CommandBody::Rename {
                from: mailbox,
                to: new_mailbox,
            } => {
                writer.write_all(b"RENAME")?;
                writer.write_all(b" ")?;
                mailbox.core_encode(writer)?;
                writer.write_all(b" ")?;
                new_mailbox.core_encode(writer)
            }
            CommandBody::Subscribe { mailbox } => {
                writer.write_all(b"SUBSCRIBE")?;
                writer.write_all(b" ")?;
                mailbox.core_encode(writer)
            }
            CommandBody::Unsubscribe { mailbox } => {
                writer.write_all(b"UNSUBSCRIBE")?;
                writer.write_all(b" ")?;
                mailbox.core_encode(writer)
            }
            CommandBody::List {
                reference,
                mailbox_wildcard,
            } => {
                writer.write_all(b"LIST")?;
                writer.write_all(b" ")?;
                reference.core_encode(writer)?;
                writer.write_all(b" ")?;
                mailbox_wildcard.core_encode(writer)
            }
            CommandBody::Lsub {
                reference,
                mailbox_wildcard,
            } => {
                writer.write_all(b"LSUB")?;
                writer.write_all(b" ")?;
                reference.core_encode(writer)?;
                writer.write_all(b" ")?;
                mailbox_wildcard.core_encode(writer)
            }
            CommandBody::Status {
                mailbox,
                attributes,
            } => {
                writer.write_all(b"STATUS")?;
                writer.write_all(b" ")?;
                mailbox.core_encode(writer)?;
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
                mailbox.core_encode(writer)?;

                if !flags.is_empty() {
                    writer.write_all(b" ")?;
                    writer.write_all(b"(")?;
                    join_serializable(flags, b" ", writer)?;
                    writer.write_all(b")")?;
                }

                if let Some(date) = date {
                    writer.write_all(b" ")?;
                    date.core_encode(writer)?;
                }

                writer.write_all(b" ")?;
                message.core_encode(writer)
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
                    charset.core_encode(writer)?;
                }
                writer.write_all(b" ")?;
                criteria.core_encode(writer)
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

                sequence_set.core_encode(writer)?;
                writer.write_all(b" ")?;
                attributes.core_encode(writer)
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

                sequence_set.core_encode(writer)?;
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
                sequence_set.core_encode(writer)?;
                writer.write_all(b" ")?;
                mailbox.core_encode(writer)
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
                algorithm.core_encode(writer)
            }
            #[cfg(feature = "ext_quota")]
            CommandBody::GetQuota { root } => {
                writer.write_all(b"GETQUOTA ")?;
                root.core_encode(writer)
            }
            #[cfg(feature = "ext_quota")]
            CommandBody::GetQuotaRoot { mailbox } => {
                writer.write_all(b"GETQUOTAROOT ")?;
                mailbox.core_encode(writer)
            }
            #[cfg(feature = "ext_quota")]
            CommandBody::SetQuota { root, quotas } => {
                writer.write_all(b"SETQUOTA ")?;
                root.core_encode(writer)?;
                writer.write_all(b" (")?;
                join_serializable(quotas.as_ref(), b" ", writer)?;
                writer.write_all(b")")
            }
            #[cfg(feature = "ext_move")]
            CommandBody::Move {
                sequence_set,
                mailbox,
                uid,
            } => {
                if *uid {
                    writer.write_all(b"UID MOVE ")?;
                } else {
                    writer.write_all(b"MOVE ")?;
                }
                sequence_set.core_encode(writer)?;
                writer.write_all(b" ")?;
                mailbox.core_encode(writer)
            }
        }
    }
}

impl<'a> CoreEncode for AuthMechanism<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match &self {
            AuthMechanism::Plain => writer.write_all(b"PLAIN"),
            AuthMechanism::Login => writer.write_all(b"LOGIN"),
            AuthMechanism::Other(other) => other.core_encode(writer),
        }
    }
}

impl<'a> CoreEncode for AuthMechanismOther<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        self.inner().core_encode(writer)
    }
}

impl CoreEncode for AuthenticateData {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        let encoded = base64.encode(self.0.declassify());
        writer.write_all(encoded.as_bytes())
    }
}

impl<'a> CoreEncode for AString<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            AString::Atom(atom) => atom.core_encode(writer),
            AString::String(imap_str) => imap_str.core_encode(writer),
        }
    }
}

impl<'a> CoreEncode for Atom<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        writer.write_all(self.inner().as_bytes())
    }
}

impl<'a> CoreEncode for AtomExt<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        writer.write_all(self.inner().as_bytes())
    }
}

impl<'a> CoreEncode for IString<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            Self::Literal(val) => val.core_encode(writer),
            Self::Quoted(val) => val.core_encode(writer),
        }
    }
}

impl<'a> CoreEncode for Literal<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        #[cfg(not(feature = "ext_literal"))]
        {
            write!(writer, "{{{}}}\r\n", self.as_ref().len())?;
            writer.recv_continuation_request();
        }

        #[cfg(feature = "ext_literal")]
        if self.sync {
            write!(writer, "{{{}}}\r\n", self.as_ref().len())?;
            writer.recv_continuation_request();
        } else {
            write!(writer, "{{{}+}}\r\n", self.as_ref().len())?;
        }

        writer.write_all(self.as_ref())
    }
}

impl<'a> CoreEncode for Quoted<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        write!(writer, "\"{}\"", escape_quoted(self.inner()))
    }
}

impl<'a> CoreEncode for Mailbox<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            Mailbox::Inbox => writer.write_all(b"INBOX"),
            Mailbox::Other(other) => other.core_encode(writer),
        }
    }
}

impl<'a> CoreEncode for MailboxOther<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        self.inner().core_encode(writer)
    }
}

impl<'a> CoreEncode for ListMailbox<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            ListMailbox::Token(lcs) => lcs.core_encode(writer),
            ListMailbox::String(istr) => istr.core_encode(writer),
        }
    }
}

impl<'a> CoreEncode for ListCharString<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        writer.write_all(self.as_ref())
    }
}

impl CoreEncode for StatusAttribute {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
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
            #[cfg(feature = "ext_condstore_qresync")]
            StatusAttribute::HighestModSeq => writer.write_all(b"HIGHESTMODSEQ"),
        }
    }
}

impl<'a> CoreEncode for Flag<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            Flag::Seen => writer.write_all(b"\\Seen"),
            Flag::Answered => writer.write_all(b"\\Answered"),
            Flag::Flagged => writer.write_all(b"\\Flagged"),
            Flag::Deleted => writer.write_all(b"\\Deleted"),
            Flag::Draft => writer.write_all(b"\\Draft"),
            Flag::Extension(other) => other.core_encode(writer),
            Flag::Keyword(atom) => atom.core_encode(writer),
        }
    }
}

impl<'a> CoreEncode for FlagFetch<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            Self::Flag(flag) => flag.core_encode(writer),
            Self::Recent => writer.write_all(b"\\Recent"),
        }
    }
}

impl<'a> CoreEncode for FlagPerm<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            Self::Flag(flag) => flag.core_encode(writer),
            Self::AllowNewKeywords => writer.write_all(b"\\*"),
        }
    }
}

impl<'a> CoreEncode for FlagExtension<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        writer.write_all(b"\\")?;
        writer.write_all(self.as_ref().as_bytes())
    }
}

impl CoreEncode for DateTime {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        self.as_ref().core_encode(writer)
    }
}

impl<'a> CoreEncode for Charset<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            Charset::Atom(atom) => atom.core_encode(writer),
            Charset::Quoted(quoted) => quoted.core_encode(writer),
        }
    }
}

impl<'a> CoreEncode for SearchKey<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            SearchKey::All => writer.write_all(b"ALL"),
            SearchKey::Answered => writer.write_all(b"ANSWERED"),
            SearchKey::Bcc(astring) => {
                writer.write_all(b"BCC ")?;
                astring.core_encode(writer)
            }
            SearchKey::Before(date) => {
                writer.write_all(b"BEFORE ")?;
                date.core_encode(writer)
            }
            SearchKey::Body(astring) => {
                writer.write_all(b"BODY ")?;
                astring.core_encode(writer)
            }
            SearchKey::Cc(astring) => {
                writer.write_all(b"CC ")?;
                astring.core_encode(writer)
            }
            SearchKey::Deleted => writer.write_all(b"DELETED"),
            SearchKey::Flagged => writer.write_all(b"FLAGGED"),
            SearchKey::From(astring) => {
                writer.write_all(b"FROM ")?;
                astring.core_encode(writer)
            }
            SearchKey::Keyword(flag_keyword) => {
                writer.write_all(b"KEYWORD ")?;
                flag_keyword.core_encode(writer)
            }
            SearchKey::New => writer.write_all(b"NEW"),
            SearchKey::Old => writer.write_all(b"OLD"),
            SearchKey::On(date) => {
                writer.write_all(b"ON ")?;
                date.core_encode(writer)
            }
            SearchKey::Recent => writer.write_all(b"RECENT"),
            SearchKey::Seen => writer.write_all(b"SEEN"),
            SearchKey::Since(date) => {
                writer.write_all(b"SINCE ")?;
                date.core_encode(writer)
            }
            SearchKey::Subject(astring) => {
                writer.write_all(b"SUBJECT ")?;
                astring.core_encode(writer)
            }
            SearchKey::Text(astring) => {
                writer.write_all(b"TEXT ")?;
                astring.core_encode(writer)
            }
            SearchKey::To(astring) => {
                writer.write_all(b"TO ")?;
                astring.core_encode(writer)
            }
            SearchKey::Unanswered => writer.write_all(b"UNANSWERED"),
            SearchKey::Undeleted => writer.write_all(b"UNDELETED"),
            SearchKey::Unflagged => writer.write_all(b"UNFLAGGED"),
            SearchKey::Unkeyword(flag_keyword) => {
                writer.write_all(b"UNKEYWORD ")?;
                flag_keyword.core_encode(writer)
            }
            SearchKey::Unseen => writer.write_all(b"UNSEEN"),
            SearchKey::Draft => writer.write_all(b"DRAFT"),
            SearchKey::Header(header_fld_name, astring) => {
                writer.write_all(b"HEADER ")?;
                header_fld_name.core_encode(writer)?;
                writer.write_all(b" ")?;
                astring.core_encode(writer)
            }
            SearchKey::Larger(number) => write!(writer, "LARGER {number}"),
            SearchKey::Not(search_key) => {
                writer.write_all(b"NOT ")?;
                search_key.core_encode(writer)
            }
            SearchKey::Or(search_key_a, search_key_b) => {
                writer.write_all(b"OR ")?;
                search_key_a.core_encode(writer)?;
                writer.write_all(b" ")?;
                search_key_b.core_encode(writer)
            }
            SearchKey::SentBefore(date) => {
                writer.write_all(b"SENTBEFORE ")?;
                date.core_encode(writer)
            }
            SearchKey::SentOn(date) => {
                writer.write_all(b"SENTON ")?;
                date.core_encode(writer)
            }
            SearchKey::SentSince(date) => {
                writer.write_all(b"SENTSINCE ")?;
                date.core_encode(writer)
            }
            SearchKey::Smaller(number) => write!(writer, "SMALLER {number}"),
            SearchKey::Uid(sequence_set) => {
                writer.write_all(b"UID ")?;
                sequence_set.core_encode(writer)
            }
            SearchKey::Undraft => writer.write_all(b"UNDRAFT"),
            SearchKey::SequenceSet(sequence_set) => sequence_set.core_encode(writer),
            SearchKey::And(search_keys) => {
                writer.write_all(b"(")?;
                join_serializable(search_keys.as_ref(), b" ", writer)?;
                writer.write_all(b")")
            }
        }
    }
}

impl CoreEncode for SequenceSet {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        join_serializable(self.0.as_ref(), b",", writer)
    }
}

impl CoreEncode for Sequence {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            Sequence::Single(seq_no) => seq_no.core_encode(writer),
            Sequence::Range(from, to) => {
                from.core_encode(writer)?;
                writer.write_all(b":")?;
                to.core_encode(writer)
            }
        }
    }
}

impl CoreEncode for SeqOrUid {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            SeqOrUid::Value(number) => write!(writer, "{number}"),
            SeqOrUid::Asterisk => writer.write_all(b"*"),
        }
    }
}

impl CoreEncode for NaiveDate {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        write!(writer, "\"{}\"", self.as_ref().format("%d-%b-%Y"))
    }
}

impl<'a> CoreEncode for MacroOrFetchAttributes<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            MacroOrFetchAttributes::Macro(m) => m.core_encode(writer),
            MacroOrFetchAttributes::FetchAttributes(attributes) => {
                if attributes.len() == 1 {
                    attributes[0].core_encode(writer)
                } else {
                    writer.write_all(b"(")?;
                    join_serializable(attributes.as_slice(), b" ", writer)?;
                    writer.write_all(b")")
                }
            }
        }
    }
}

impl CoreEncode for Macro {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            Macro::All => writer.write_all(b"ALL"),
            Macro::Fast => writer.write_all(b"FAST"),
            Macro::Full => writer.write_all(b"FULL"),
        }
    }
}

impl<'a> CoreEncode for FetchAttribute<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
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
                    section.core_encode(writer)?;
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

impl<'a> CoreEncode for Section<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            Section::Part(part) => part.core_encode(writer),
            Section::Header(maybe_part) => match maybe_part {
                Some(part) => {
                    part.core_encode(writer)?;
                    writer.write_all(b".HEADER")
                }
                None => writer.write_all(b"HEADER"),
            },
            Section::HeaderFields(maybe_part, header_list) => {
                match maybe_part {
                    Some(part) => {
                        part.core_encode(writer)?;
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
                        part.core_encode(writer)?;
                        writer.write_all(b".HEADER.FIELDS.NOT (")?;
                    }
                    None => writer.write_all(b"HEADER.FIELDS.NOT (")?,
                };
                join_serializable(header_list.as_ref(), b" ", writer)?;
                writer.write_all(b")")
            }
            Section::Text(maybe_part) => match maybe_part {
                Some(part) => {
                    part.core_encode(writer)?;
                    writer.write_all(b".TEXT")
                }
                None => writer.write_all(b"TEXT"),
            },
            Section::Mime(part) => {
                part.core_encode(writer)?;
                writer.write_all(b".MIME")
            }
        }
    }
}

impl CoreEncode for Part {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        join_serializable(self.0.as_ref(), b".", writer)
    }
}

impl CoreEncode for NonZeroU32 {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        write!(writer, "{self}")
    }
}

impl<'a> CoreEncode for Capability<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            Self::Imap4Rev1 => writer.write_all(b"IMAP4REV1"),
            Self::Auth(mechanism) => match mechanism {
                AuthMechanism::Plain => writer.write_all(b"AUTH=PLAIN"),
                AuthMechanism::Login => writer.write_all(b"AUTH=LOGIN"),
                AuthMechanism::Other(other) => {
                    writer.write_all(b"AUTH=")?;
                    other.core_encode(writer)
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
                resource.core_encode(writer)
            }
            #[cfg(feature = "ext_quota")]
            Self::QuotaSet => writer.write_all(b"QUOTASET"),
            #[cfg(feature = "ext_literal")]
            Self::Literal(literal_capability) => literal_capability.core_encode(writer),
            #[cfg(feature = "ext_move")]
            Self::Move => writer.write_all(b"MOVE"),
            Self::Other(other) => other.inner().core_encode(writer),
        }
    }
}

// ----- Responses ---------------------------------------------------------------------------------

impl<'a> CoreEncode for Response<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            Response::Status(status) => status.core_encode(writer),
            Response::Data(data) => data.core_encode(writer),
            Response::Continue(continue_request) => continue_request.core_encode(writer),
        }
    }
}

impl<'a> CoreEncode for Greeting<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        writer.write_all(b"* ")?;
        self.kind.core_encode(writer)?;
        writer.write_all(b" ")?;

        if let Some(ref code) = self.code {
            writer.write_all(b"[")?;
            code.core_encode(writer)?;
            writer.write_all(b"] ")?;
        }

        self.text.core_encode(writer)?;
        writer.write_all(b"\r\n")
    }
}

impl CoreEncode for GreetingKind {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            GreetingKind::Ok => writer.write_all(b"OK"),
            GreetingKind::PreAuth => writer.write_all(b"PREAUTH"),
            GreetingKind::Bye => writer.write_all(b"BYE"),
        }
    }
}

impl<'a> CoreEncode for Status<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        fn format_status(
            tag: &Option<Tag>,
            status: &str,
            code: &Option<Code>,
            comment: &Text,
            writer: &mut EncodeContext,
        ) -> std::io::Result<()> {
            match tag {
                Some(tag) => tag.core_encode(writer)?,
                None => writer.write_all(b"*")?,
            }
            writer.write_all(b" ")?;
            writer.write_all(status.as_bytes())?;
            writer.write_all(b" ")?;
            if let Some(code) = code {
                writer.write_all(b"[")?;
                code.core_encode(writer)?;
                writer.write_all(b"] ")?;
            }
            comment.core_encode(writer)?;
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

impl<'a> CoreEncode for Code<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
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
                next.core_encode(writer)
            }
            Code::UidValidity(validity) => {
                writer.write_all(b"UIDVALIDITY ")?;
                validity.core_encode(writer)
            }
            Code::Unseen(seq) => {
                writer.write_all(b"UNSEEN ")?;
                seq.core_encode(writer)
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
            #[cfg(feature = "ext_literal")]
            Code::TooBig => writer.write_all(b"TOOBIG"),
            Code::Other(unknown) => unknown.core_encode(writer),
        }
    }
}

impl<'a> CoreEncode for CodeOther<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        writer.write_all(self.inner())
    }
}

impl<'a> CoreEncode for Text<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        writer.write_all(self.inner().as_bytes())
    }
}

impl<'a> CoreEncode for Data<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
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
                    delimiter.core_encode(writer)?;
                    writer.write_all(b"\"")?;
                } else {
                    writer.write_all(b"NIL")?;
                }
                writer.write_all(b" ")?;
                mailbox.core_encode(writer)?;
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
                    delimiter.core_encode(writer)?;
                    writer.write_all(b"\"")?;
                } else {
                    writer.write_all(b"NIL")?;
                }
                writer.write_all(b" ")?;
                mailbox.core_encode(writer)?;
            }
            Data::Status {
                mailbox,
                attributes,
            } => {
                writer.write_all(b"* STATUS ")?;
                mailbox.core_encode(writer)?;
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
                    cap.core_encode(writer)?;
                }
            }
            #[cfg(feature = "ext_quota")]
            Data::Quota { root, quotas } => {
                writer.write_all(b"* QUOTA ")?;
                root.core_encode(writer)?;
                writer.write_all(b" (")?;
                join_serializable(quotas.as_ref(), b" ", writer)?;
                writer.write_all(b")")?;
            }
            #[cfg(feature = "ext_quota")]
            Data::QuotaRoot { mailbox, roots } => {
                writer.write_all(b"* QUOTAROOT ")?;
                mailbox.core_encode(writer)?;
                for root in roots {
                    writer.write_all(b" ")?;
                    root.core_encode(writer)?;
                }
            }
        }

        writer.write_all(b"\r\n")
    }
}

impl<'a> CoreEncode for FlagNameAttribute<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            Self::Noinferiors => writer.write_all(b"\\Noinferiors"),
            Self::Noselect => writer.write_all(b"\\Noselect"),
            Self::Marked => writer.write_all(b"\\Marked"),
            Self::Unmarked => writer.write_all(b"\\Unmarked"),
            Self::Extension(atom) => {
                writer.write_all(b"\\")?;
                atom.core_encode(writer)
            }
        }
    }
}

impl CoreEncode for QuotedChar {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self.inner() {
            '\\' => writer.write_all(b"\\\\"),
            '"' => writer.write_all(b"\\\""),
            other => writer.write_all(&[other as u8]),
        }
    }
}

impl CoreEncode for StatusAttributeValue {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            Self::Messages(count) => {
                writer.write_all(b"MESSAGES ")?;
                count.core_encode(writer)
            }
            Self::Recent(count) => {
                writer.write_all(b"RECENT ")?;
                count.core_encode(writer)
            }
            Self::UidNext(next) => {
                writer.write_all(b"UIDNEXT ")?;
                next.core_encode(writer)
            }
            Self::UidValidity(identifier) => {
                writer.write_all(b"UIDVALIDITY ")?;
                identifier.core_encode(writer)
            }
            Self::Unseen(count) => {
                writer.write_all(b"UNSEEN ")?;
                count.core_encode(writer)
            }
            #[cfg(feature = "ext_quota")]
            Self::Deleted(count) => {
                writer.write_all(b"DELETED ")?;
                count.core_encode(writer)
            }
            #[cfg(feature = "ext_quota")]
            Self::DeletedStorage(count) => {
                writer.write_all(b"DELETED-STORAGE ")?;
                count.core_encode(writer)
            }
        }
    }
}

impl<'a> CoreEncode for FetchAttributeValue<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            Self::BodyExt {
                section,
                origin,
                data,
            } => {
                writer.write_all(b"BODY[")?;
                if let Some(section) = section {
                    section.core_encode(writer)?;
                }
                writer.write_all(b"]")?;
                if let Some(origin) = origin {
                    write!(writer, "<{origin}>")?;
                }
                writer.write_all(b" ")?;
                data.core_encode(writer)
            }
            // FIXME: do not return body-ext-1part and body-ext-mpart here
            Self::Body(body) => {
                writer.write_all(b"BODY ")?;
                body.core_encode(writer)
            }
            Self::BodyStructure(body) => {
                writer.write_all(b"BODYSTRUCTURE ")?;
                body.core_encode(writer)
            }
            Self::Envelope(envelope) => {
                writer.write_all(b"ENVELOPE ")?;
                envelope.core_encode(writer)
            }
            Self::Flags(flags) => {
                writer.write_all(b"FLAGS (")?;
                join_serializable(flags, b" ", writer)?;
                writer.write_all(b")")
            }
            Self::InternalDate(datetime) => {
                writer.write_all(b"INTERNALDATE ")?;
                datetime.core_encode(writer)
            }
            Self::Rfc822(nstring) => {
                writer.write_all(b"RFC822 ")?;
                nstring.core_encode(writer)
            }
            Self::Rfc822Header(nstring) => {
                writer.write_all(b"RFC822.HEADER ")?;
                nstring.core_encode(writer)
            }
            Self::Rfc822Size(size) => write!(writer, "RFC822.SIZE {size}"),
            Self::Rfc822Text(nstring) => {
                writer.write_all(b"RFC822.TEXT ")?;
                nstring.core_encode(writer)
            }
            Self::Uid(uid) => write!(writer, "UID {uid}"),
        }
    }
}

impl<'a> CoreEncode for NString<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match &self.0 {
            Some(imap_str) => imap_str.core_encode(writer),
            None => writer.write_all(b"NIL"),
        }
    }
}

impl<'a> CoreEncode for BodyStructure<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        writer.write_all(b"(")?;
        match self {
            BodyStructure::Single {
                body,
                extension_data: extension,
            } => {
                body.core_encode(writer)?;
                if let Some(extension) = extension {
                    writer.write_all(b" ")?;
                    extension.core_encode(writer)?;
                }
            }
            BodyStructure::Multi {
                bodies,
                subtype,
                extension_data,
            } => {
                for body in bodies.as_ref() {
                    body.core_encode(writer)?;
                }
                writer.write_all(b" ")?;
                subtype.core_encode(writer)?;

                if let Some(extension) = extension_data {
                    writer.write_all(b" ")?;
                    extension.core_encode(writer)?;
                }
            }
        }
        writer.write_all(b")")
    }
}

impl<'a> CoreEncode for Body<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self.specific {
            SpecificFields::Basic {
                r#type: ref type_,
                ref subtype,
            } => {
                type_.core_encode(writer)?;
                writer.write_all(b" ")?;
                subtype.core_encode(writer)?;
                writer.write_all(b" ")?;
                self.basic.core_encode(writer)
            }
            SpecificFields::Message {
                ref envelope,
                ref body_structure,
                number_of_lines,
            } => {
                writer.write_all(b"\"MESSAGE\" \"RFC822\" ")?;
                self.basic.core_encode(writer)?;
                writer.write_all(b" ")?;
                envelope.core_encode(writer)?;
                writer.write_all(b" ")?;
                body_structure.core_encode(writer)?;
                writer.write_all(b" ")?;
                write!(writer, "{number_of_lines}")
            }
            SpecificFields::Text {
                ref subtype,
                number_of_lines,
            } => {
                writer.write_all(b"\"TEXT\" ")?;
                subtype.core_encode(writer)?;
                writer.write_all(b" ")?;
                self.basic.core_encode(writer)?;
                writer.write_all(b" ")?;
                write!(writer, "{number_of_lines}")
            }
        }
    }
}

impl<'a> CoreEncode for BasicFields<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        List1AttributeValueOrNil(&self.parameter_list).core_encode(writer)?;
        writer.write_all(b" ")?;
        self.id.core_encode(writer)?;
        writer.write_all(b" ")?;
        self.description.core_encode(writer)?;
        writer.write_all(b" ")?;
        self.content_transfer_encoding.core_encode(writer)?;
        writer.write_all(b" ")?;
        write!(writer, "{}", self.size)
    }
}

impl<'a> CoreEncode for Envelope<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        writer.write_all(b"(")?;
        self.date.core_encode(writer)?;
        writer.write_all(b" ")?;
        self.subject.core_encode(writer)?;
        writer.write_all(b" ")?;
        List1OrNil(&self.from, b"").core_encode(writer)?;
        writer.write_all(b" ")?;
        List1OrNil(&self.sender, b"").core_encode(writer)?;
        writer.write_all(b" ")?;
        List1OrNil(&self.reply_to, b"").core_encode(writer)?;
        writer.write_all(b" ")?;
        List1OrNil(&self.to, b"").core_encode(writer)?;
        writer.write_all(b" ")?;
        List1OrNil(&self.cc, b"").core_encode(writer)?;
        writer.write_all(b" ")?;
        List1OrNil(&self.bcc, b"").core_encode(writer)?;
        writer.write_all(b" ")?;
        self.in_reply_to.core_encode(writer)?;
        writer.write_all(b" ")?;
        self.message_id.core_encode(writer)?;
        writer.write_all(b")")
    }
}

impl<'a> CoreEncode for Address<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        writer.write_all(b"(")?;
        self.name.core_encode(writer)?;
        writer.write_all(b" ")?;
        self.adl.core_encode(writer)?;
        writer.write_all(b" ")?;
        self.mailbox.core_encode(writer)?;
        writer.write_all(b" ")?;
        self.host.core_encode(writer)?;
        writer.write_all(b")")?;

        Ok(())
    }
}

impl<'a> CoreEncode for SinglePartExtensionData<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        self.md5.core_encode(writer)?;

        if let Some(disposition) = &self.tail {
            writer.write_all(b" ")?;
            disposition.core_encode(writer)?;
        }

        Ok(())
    }
}

impl<'a> CoreEncode for MultiPartExtensionData<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        List1AttributeValueOrNil(&self.parameter_list).core_encode(writer)?;

        if let Some(disposition) = &self.tail {
            writer.write_all(b" ")?;
            disposition.core_encode(writer)?;
        }

        Ok(())
    }
}

impl<'a> CoreEncode for Disposition<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match &self.disposition {
            Some((s, param)) => {
                writer.write_all(b"(")?;
                s.core_encode(writer)?;
                writer.write_all(b" ")?;
                List1AttributeValueOrNil(param).core_encode(writer)?;
                writer.write_all(b")")?;
            }
            None => writer.write_all(b"NIL")?,
        }

        if let Some(language) = &self.tail {
            writer.write_all(b" ")?;
            language.core_encode(writer)?;
        }

        Ok(())
    }
}

impl<'a> CoreEncode for Language<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        List1OrNil(&self.language, b" ").core_encode(writer)?;

        if let Some(location) = &self.tail {
            writer.write_all(b" ")?;
            location.core_encode(writer)?;
        }

        Ok(())
    }
}

impl<'a> CoreEncode for Location<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        self.location.core_encode(writer)?;

        for body_extension in &self.extensions {
            writer.write_all(b" ")?;
            body_extension.core_encode(writer)?;
        }

        Ok(())
    }
}

impl<'a> CoreEncode for BodyExtension<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            BodyExtension::NString(nstring) => nstring.core_encode(writer),
            BodyExtension::Number(number) => number.core_encode(writer),
            BodyExtension::List(list) => {
                writer.write_all(b"(")?;
                join_serializable(list.as_ref(), b" ", writer)?;
                writer.write_all(b")")
            }
        }
    }
}

impl CoreEncode for ChronoDateTime<FixedOffset> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        write!(writer, "\"{}\"", self.format("%d-%b-%Y %H:%M:%S %z"))
    }
}

impl<'a> CoreEncode for Continue<'a> {
    fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
        match self {
            Continue::Basic(continue_basic) => match continue_basic.code() {
                Some(code) => {
                    writer.write_all(b"+ [")?;
                    code.core_encode(writer)?;
                    writer.write_all(b"] ")?;
                    continue_basic.text().core_encode(writer)?;
                    writer.write_all(b"\r\n")
                }
                None => {
                    writer.write_all(b"+ ")?;
                    continue_basic.text().core_encode(writer)?;
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

mod utils {
    use std::io::Write;

    use super::CoreEncode;
    use crate::codec::encode::EncodeContext;

    pub struct List1OrNil<'a, T>(pub &'a Vec<T>, pub &'a [u8]);

    pub struct List1AttributeValueOrNil<'a, T>(pub &'a Vec<(T, T)>);

    pub fn join_serializable<I: CoreEncode>(
        elements: &[I],
        sep: &[u8],
        writer: &mut EncodeContext,
    ) -> std::io::Result<()> {
        if let Some((last, head)) = elements.split_last() {
            for item in head {
                item.core_encode(writer)?;
                writer.write_all(sep)?;
            }

            last.core_encode(writer)
        } else {
            Ok(())
        }
    }

    impl<'a, T> CoreEncode for List1OrNil<'a, T>
    where
        T: CoreEncode,
    {
        fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
            if let Some((last, head)) = self.0.split_last() {
                writer.write_all(b"(")?;

                for item in head {
                    item.core_encode(writer)?;
                    writer.write_all(self.1)?;
                }

                last.core_encode(writer)?;

                writer.write_all(b")")
            } else {
                writer.write_all(b"NIL")
            }
        }
    }

    impl<'a, T> CoreEncode for List1AttributeValueOrNil<'a, T>
    where
        T: CoreEncode,
    {
        fn core_encode(&self, writer: &mut EncodeContext) -> std::io::Result<()> {
            if let Some((last, head)) = self.0.split_last() {
                writer.write_all(b"(")?;

                for (attribute, value) in head {
                    attribute.core_encode(writer)?;
                    writer.write_all(b" ")?;
                    value.core_encode(writer)?;
                    writer.write_all(b" ")?;
                }

                let (attribute, value) = last;
                attribute.core_encode(writer)?;
                writer.write_all(b" ")?;
                value.core_encode(writer)?;

                writer.write_all(b")")
            } else {
                writer.write_all(b"NIL")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::*;
    use crate::{
        auth::AuthMechanism,
        command::{Command, CommandBody},
        core::{AString, Literal, NString, NonEmptyVec},
        fetch::FetchAttributeValue,
        response::{Data, Response},
        utils::escape_byte_string,
    };

    #[test]
    fn test_api_encoder_usage() {
        let cmd = Command::new(
            "A",
            CommandBody::login(
                AString::from(Literal::unchecked(
                    b"alice".as_ref(),
                    #[cfg(feature = "ext_literal")]
                    false,
                )),
                "password",
            )
            .unwrap(),
        )
        .unwrap();

        // Dump.
        let got_encoded = cmd.encode().dump();

        // Encoding.
        let encoder = cmd.encode();

        let mut out = Vec::new();

        for x in encoder {
            match x {
                Action::Send { data } => {
                    println!("C: {}", escape_byte_string(&data));
                    out.extend_from_slice(&data);
                }
                Action::RecvContinuationRequest => {
                    println!("C: <Waiting for continuation request>");
                }
                Action::Unknown => {
                    println!("C: <Custom message flow required>");
                }
            }
        }

        assert_eq!(got_encoded, out);
    }

    #[test]
    fn test_encode_command() {
        kat_encoder(&[
            (
                Command::new("A", CommandBody::login("alice", "pass").unwrap()).unwrap(),
                [Action::Send {
                    data: b"A LOGIN alice pass\r\n".to_vec(),
                }]
                .as_ref(),
            ),
            (
                Command::new(
                    "A",
                    CommandBody::login("alice", b"\xCA\xFE".as_ref()).unwrap(),
                )
                .unwrap(),
                [
                    Action::Send {
                        data: b"A LOGIN alice {2}\r\n".to_vec(),
                    },
                    Action::RecvContinuationRequest,
                    Action::Send {
                        data: b"\xCA\xFE\r\n".to_vec(),
                    },
                ]
                .as_ref(),
            ),
            (
                Command::new(
                    "A",
                    CommandBody::authenticate(
                        AuthMechanism::Login,
                        #[cfg(feature = "ext_sasl_ir")]
                        None,
                    ),
                )
                .unwrap(),
                [
                    Action::Send {
                        data: b"A AUTHENTICATE LOGIN\r\n".to_vec(),
                    },
                    Action::RecvContinuationRequest,
                ]
                .as_ref(),
            ),
            #[cfg(feature = "ext_sasl_ir")]
            (
                Command::new(
                    "A",
                    CommandBody::authenticate(AuthMechanism::Login, Some(b"alice")),
                )
                .unwrap(),
                [
                    Action::Send {
                        data: b"A AUTHENTICATE LOGIN YWxpY2U=\r\n".to_vec(),
                    },
                    Action::RecvContinuationRequest,
                ]
                .as_ref(),
            ),
            #[cfg(feature = "ext_sasl_ir")]
            (
                Command::new("A", CommandBody::authenticate(AuthMechanism::Plain, None)).unwrap(),
                [
                    Action::Send {
                        data: b"A AUTHENTICATE PLAIN\r\n".to_vec(),
                    },
                    Action::RecvContinuationRequest,
                ]
                .as_ref(),
            ),
            #[cfg(feature = "ext_sasl_ir")]
            (
                Command::new(
                    "A",
                    CommandBody::authenticate(AuthMechanism::Plain, Some(b"\x00alice\x00pass")),
                )
                .unwrap(),
                [Action::Send {
                    data: b"A AUTHENTICATE PLAIN AGFsaWNlAHBhc3M=\r\n".to_vec(),
                }]
                .as_ref(),
            ),
        ]);
    }

    #[test]
    fn test_encode_response() {
        kat_encoder(&[
            (
                Response::Data(Data::Fetch {
                    seq_or_uid: NonZeroU32::new(12345).unwrap(),
                    attributes: NonEmptyVec::from(FetchAttributeValue::BodyExt {
                        section: None,
                        origin: None,
                        data: NString::from(Literal::unchecked(
                            b"ABCDE".as_ref(),
                            #[cfg(feature = "ext_literal")]
                            true,
                        )),
                    }),
                }),
                [
                    Action::Send {
                        data: b"* 12345 FETCH (BODY[] {5}\r\n".to_vec(),
                    },
                    Action::RecvContinuationRequest,
                    Action::Send {
                        data: b"ABCDE)\r\n".to_vec(),
                    },
                ]
                .as_ref(),
            ),
            #[cfg(feature = "ext_literal")]
            (
                Response::Data(Data::Fetch {
                    seq_or_uid: NonZeroU32::new(12345).unwrap(),
                    attributes: NonEmptyVec::from(FetchAttributeValue::BodyExt {
                        section: None,
                        origin: None,
                        data: NString::from(Literal::unchecked(b"ABCDE".as_ref(), false)),
                    }),
                }),
                [Action::Send {
                    data: b"* 12345 FETCH (BODY[] {5+}\r\nABCDE)\r\n".to_vec(),
                }]
                .as_ref(),
            ),
        ])
    }

    fn kat_encoder<Object, Actions>(tests: &[(Object, Actions)])
    where
        Object: Encode,
        Actions: AsRef<[Action]>,
    {
        for (i, (obj, actions)) in tests.iter().enumerate() {
            println!("# Testing {i}");

            let encoder = obj.encode();
            let actions = actions.as_ref();

            assert_eq!(encoder.collect::<Vec<_>>(), actions);
        }
    }
}
