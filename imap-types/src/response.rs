//! # 7. Server Responses

use std::{
    borrow::Cow,
    fmt::{Debug, Display, Formatter},
    num::{NonZeroU32, TryFromIntError},
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use base64::{engine::general_purpose::STANDARD as _base64, Engine};
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "ext_compress")]
use crate::extensions::compress::CompressionAlgorithm;
#[cfg(feature = "ext_enable")]
use crate::extensions::enable::CapabilityEnable;
use crate::{
    auth::AuthMechanism,
    core::{impl_try_from, Atom, Charset, NonEmptyVec, QuotedChar, Tag, Text, TextError},
    fetch::MessageDataItem,
    flag::{Flag, FlagNameAttribute, FlagPerm},
    mailbox::Mailbox,
    status::StatusDataItem,
};
#[cfg(feature = "ext_quota")]
use crate::{
    core::AString,
    extensions::quota::{QuotaGet, Resource},
};

/// An IMAP greeting.
///
/// Note: Don't use `code: None` *and* a `text` that starts with "[" as this would be ambiguous in IMAP.
// TODO(301)
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Greeting<'a> {
    pub kind: GreetingKind,
    pub code: Option<Code<'a>>,
    pub text: Text<'a>,
}

impl<'a> Greeting<'a> {
    pub fn new(
        kind: GreetingKind,
        code: Option<Code<'a>>,
        text: &'a str,
    ) -> Result<Self, TextError> {
        Ok(Greeting {
            kind,
            code,
            text: text.try_into()?,
        })
    }

    pub fn ok(code: Option<Code<'a>>, text: &'a str) -> Result<Self, TextError> {
        Ok(Greeting {
            kind: GreetingKind::Ok,
            code,
            text: text.try_into()?,
        })
    }

    pub fn preauth(code: Option<Code<'a>>, text: &'a str) -> Result<Self, TextError> {
        Ok(Greeting {
            kind: GreetingKind::PreAuth,
            code,
            text: text.try_into()?,
        })
    }

    pub fn bye(code: Option<Code<'a>>, text: &'a str) -> Result<Self, TextError> {
        Ok(Greeting {
            kind: GreetingKind::Bye,
            code,
            text: text.try_into()?,
        })
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// IMAP4rev1 defines three possible greetings at connection startup.
pub enum GreetingKind {
    /// The connection is not yet authenticated.
    ///
    /// (Advice: A LOGIN command is needed.)
    Ok,
    /// The connection has already been authenticated by external means.
    ///
    /// (Advice: No LOGIN command is needed.)
    PreAuth,
    /// The server is not willing to accept a connection from this client.
    ///
    /// (Advice: The server closes the connection immediately.)
    Bye,
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Response<'a> {
    /// Status responses can be tagged or untagged.  Tagged status responses
    /// indicate the completion result (OK, NO, or BAD status) of a client
    /// command, and have a tag matching the command.
    Status(Status<'a>),
    /// All server data is untagged. An untagged response is indicated by the
    /// token "*" instead of a tag. Untagged status responses indicate server
    /// greeting, or server status that does not indicate the completion of a
    /// command (for example, an impending system shutdown alert).
    Data(Data<'a>),
    /// Command continuation request responses use the token "+" instead of a
    /// tag.  These responses are sent by the server to indicate acceptance
    /// of an incomplete client command and readiness for the remainder of
    /// the command.
    Continue(Continue<'a>),
}

/// ## 7.1. Server Responses - Status Responses
///
/// Status responses are OK, NO, BAD, PREAUTH and BYE.
/// OK, NO, and BAD can be tagged or untagged.
/// PREAUTH and BYE are always untagged.
/// Status responses MAY include an OPTIONAL "response code" (see [`Code`](crate::response::Code).)
///
/// Note: Don't use `code: None` *and* a `text` that starts with "[" as this would be ambiguous in IMAP.
// TODO(301)
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Status<'a> {
    /// ### 7.1.1. OK Response
    ///
    /// The OK response indicates an information message from the server.
    Ok {
        /// When tagged, it indicates successful completion of the associated
        /// command.  The human-readable text MAY be presented to the user as
        /// an information message.
        ///
        /// The untagged form indicates an information-only message; the nature
        /// of the information MAY be indicated by a response code.
        ///
        /// The untagged form is also used as one of three possible greetings
        /// at connection startup.  It indicates that the connection is not
        /// yet authenticated and that a LOGIN command is needed.
        tag: Option<Tag<'a>>,
        /// Response code (optional)
        code: Option<Code<'a>>,
        /// Human-readable text (must be at least 1 character!)
        text: Text<'a>,
    },

    /// ### 7.1.2. NO Response
    ///
    /// The NO response indicates an operational error message from the server.
    No {
        /// When tagged, it indicates unsuccessful completion of the
        /// associated command.  The untagged form indicates a warning; the
        /// command can still complete successfully.
        tag: Option<Tag<'a>>,
        /// Response code (optional)
        code: Option<Code<'a>>,
        /// The human-readable text describes the condition. (must be at least 1 character!)
        text: Text<'a>,
    },

    /// ### 7.1.3. BAD Response
    ///
    /// The BAD response indicates an error message from the server.
    Bad {
        /// When tagged, it reports a protocol-level error in the client's command;
        /// the tag indicates the command that caused the error.  The untagged
        /// form indicates a protocol-level error for which the associated
        /// command can not be determined; it can also indicate an internal
        /// server failure.
        tag: Option<Tag<'a>>,
        /// Response code (optional)
        code: Option<Code<'a>>,
        /// The human-readable text describes the condition. (must be at least 1 character!)
        text: Text<'a>,
    },

    /// ### 7.1.5. BYE Response
    ///
    /// The BYE response is always untagged, and indicates that the server
    /// is about to close the connection.
    ///
    /// The BYE response is sent under one of four conditions:
    ///
    ///    1) as part of a normal logout sequence.  The server will close
    ///       the connection after sending the tagged OK response to the
    ///       LOGOUT command.
    ///
    ///    2) as a panic shutdown announcement.  The server closes the
    ///       connection immediately.
    ///
    ///    3) as an announcement of an inactivity autologout.  The server
    ///       closes the connection immediately.
    ///
    ///    4) as one of three possible greetings at connection startup,
    ///       indicating that the server is not willing to accept a
    ///       connection from this client.  The server closes the
    ///       connection immediately.
    ///
    /// The difference between a BYE that occurs as part of a normal
    /// LOGOUT sequence (the first case) and a BYE that occurs because of
    /// a failure (the other three cases) is that the connection closes
    /// immediately in the failure case.  In all cases the client SHOULD
    /// continue to read response data from the server until the
    /// connection is closed; this will ensure that any pending untagged
    /// or completion responses are read and processed.
    Bye {
        /// Response code (optional)
        code: Option<Code<'a>>,
        /// The human-readable text MAY be displayed to the user in a status
        /// report by the client. (must be at least 1 character!)
        text: Text<'a>,
    },
}

impl<'a> Status<'a> {
    // FIXME(API)
    pub fn ok<T>(tag: Option<Tag<'a>>, code: Option<Code<'a>>, text: T) -> Result<Self, T::Error>
    where
        T: TryInto<Text<'a>>,
    {
        Ok(Status::Ok {
            tag,
            code,
            text: text.try_into()?,
        })
    }

    // FIXME(API)
    pub fn no<T>(tag: Option<Tag<'a>>, code: Option<Code<'a>>, text: T) -> Result<Self, T::Error>
    where
        T: TryInto<Text<'a>>,
    {
        Ok(Status::No {
            tag,
            code,
            text: text.try_into()?,
        })
    }

    // FIXME(API)
    pub fn bad<T>(tag: Option<Tag<'a>>, code: Option<Code<'a>>, text: T) -> Result<Self, T::Error>
    where
        T: TryInto<Text<'a>>,
    {
        Ok(Status::Bad {
            tag,
            code,
            text: text.try_into()?,
        })
    }

    pub fn bye<T>(code: Option<Code<'a>>, text: T) -> Result<Self, T::Error>
    where
        T: TryInto<Text<'a>>,
    {
        Ok(Status::Bye {
            code,
            text: text.try_into()?,
        })
    }

    // ---------------------------------------------------------------------------------------------

    pub fn tag(&self) -> Option<&Tag> {
        match self {
            Status::Ok { tag, .. } | Status::No { tag, .. } | Status::Bad { tag, .. } => {
                tag.as_ref()
            }
            Status::Bye { .. } => None,
        }
    }

    pub fn code(&self) -> Option<&Code> {
        match self {
            Status::Ok { code, .. }
            | Status::No { code, .. }
            | Status::Bad { code, .. }
            | Status::Bye { code, .. } => code.as_ref(),
        }
    }

    pub fn text(&self) -> &Text {
        match self {
            Status::Ok { text, .. }
            | Status::No { text, .. }
            | Status::Bad { text, .. }
            | Status::Bye { text, .. } => text,
        }
    }
}

/// ## 7.2 - 7.4 Server and Mailbox Status; Mailbox Size; Message Status
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Data<'a> {
    // ## 7.2. Server Responses - Server and Mailbox Status
    //
    // These responses are always untagged.  This is how server and mailbox
    // status data are transmitted from the server to the client.  Many of
    // these responses typically result from a command with the same name.
    /// ### 7.2.1. CAPABILITY Response
    ///
    /// * Contents: capability listing
    ///
    /// The CAPABILITY response occurs as a result of a CAPABILITY
    /// command.  The capability listing contains a space-separated
    /// listing of capability names that the server supports.  The
    /// capability listing MUST include the atom "IMAP4rev1".
    ///
    /// In addition, client and server implementations MUST implement the
    /// STARTTLS, LOGINDISABLED, and AUTH=PLAIN (described in [IMAP-TLS])
    /// capabilities.  See the Security Considerations section for
    /// important information.
    ///
    /// A capability name which begins with "AUTH=" indicates that the
    /// server supports that particular authentication mechanism.
    ///
    /// The LOGINDISABLED capability indicates that the LOGIN command is
    /// disabled, and that the server will respond with a tagged NO
    /// response to any attempt to use the LOGIN command even if the user
    /// name and password are valid.  An IMAP client MUST NOT issue the
    /// LOGIN command if the server advertises the LOGINDISABLED
    /// capability.
    ///
    /// Other capability names indicate that the server supports an
    /// extension, revision, or amendment to the IMAP4rev1 protocol.
    /// Server responses MUST conform to this document until the client
    /// issues a command that uses the associated capability.
    ///
    /// Capability names MUST either begin with "X" or be standard or
    /// standards-track IMAP4rev1 extensions, revisions, or amendments
    /// registered with IANA.  A server MUST NOT offer unregistered or
    /// non-standard capability names, unless such names are prefixed with
    /// an "X".
    ///
    /// Client implementations SHOULD NOT require any capability name
    /// other than "IMAP4rev1", and MUST ignore any unknown capability
    /// names.
    ///
    /// A server MAY send capabilities automatically, by using the
    /// CAPABILITY response code in the initial PREAUTH or OK responses,
    /// and by sending an updated CAPABILITY response code in the tagged
    /// OK response as part of a successful authentication.  It is
    /// unnecessary for a client to send a separate CAPABILITY command if
    /// it recognizes these automatic capabilities.
    Capability(NonEmptyVec<Capability<'a>>),

    /// ### 7.2.2. LIST Response
    ///
    /// The LIST response occurs as a result of a LIST command.  It
    /// returns a single name that matches the LIST specification.  There
    /// can be multiple LIST responses for a single LIST command.
    ///
    /// The hierarchy delimiter is a character used to delimit levels of
    /// hierarchy in a mailbox name.  A client can use it to create child
    /// mailboxes, and to search higher or lower levels of naming
    /// hierarchy.  All children of a top-level hierarchy node MUST use
    /// the same separator character.  A NIL hierarchy delimiter means
    /// that no hierarchy exists; the name is a "flat" name.
    ///
    /// The name represents an unambiguous left-to-right hierarchy, and
    /// MUST be valid for use as a reference in LIST and LSUB commands.
    /// Unless \Noselect is indicated, the name MUST also be valid as an
    /// argument for commands, such as SELECT, that accept mailbox names.
    List {
        /// Name attributes
        items: Vec<FlagNameAttribute<'a>>,
        /// Hierarchy delimiter
        delimiter: Option<QuotedChar>,
        /// Name
        mailbox: Mailbox<'a>,
    },

    /// ### 7.2.3. LSUB Response
    ///
    /// The LSUB response occurs as a result of an LSUB command.  It
    /// returns a single name that matches the LSUB specification.  There
    /// can be multiple LSUB responses for a single LSUB command.  The
    /// data is identical in format to the LIST response.
    Lsub {
        /// Name attributes
        items: Vec<FlagNameAttribute<'a>>,
        /// Hierarchy delimiter
        delimiter: Option<QuotedChar>,
        /// Name
        mailbox: Mailbox<'a>,
    },

    /// ### 7.2.4 STATUS Response
    ///
    /// The STATUS response occurs as a result of an STATUS command.  It
    /// returns the mailbox name that matches the STATUS specification and
    /// the requested mailbox status information.
    Status {
        /// Name
        mailbox: Mailbox<'a>,
        /// Status parenthesized list
        items: Cow<'a, [StatusDataItem]>,
    },

    /// ### 7.2.5. SEARCH Response
    ///
    /// * Contents: zero or more numbers
    ///
    /// The SEARCH response occurs as a result of a SEARCH or UID SEARCH
    /// command.  The number(s) refer to those messages that match the
    /// search criteria.  For SEARCH, these are message sequence numbers;
    /// for UID SEARCH, these are unique identifiers.  Each number is
    /// delimited by a space.
    Search(Vec<NonZeroU32>),

    /// ### 7.2.6.  FLAGS Response
    ///
    /// * Contents: flag parenthesized list
    ///
    /// The FLAGS response occurs as a result of a SELECT or EXAMINE
    /// command.  The flag parenthesized list identifies the flags (at a
    /// minimum, the system-defined flags) that are applicable for this
    /// mailbox.  Flags other than the system flags can also exist,
    /// depending on server implementation.
    ///
    /// The update from the FLAGS response MUST be recorded by the client.
    Flags(Vec<Flag<'a>>),

    // ## 7.3. Server Responses - Mailbox Size
    //
    // These responses are always untagged.  This is how changes in the size
    // of the mailbox are transmitted from the server to the client.
    // Immediately following the "*" token is a number that represents a
    // message count.
    /// ### 7.3.1. EXISTS Response
    ///
    /// The EXISTS response reports the number of messages in the mailbox.
    /// This response occurs as a result of a SELECT or EXAMINE command,
    /// and if the size of the mailbox changes (e.g., new messages).
    ///
    /// The update from the EXISTS response MUST be recorded by the client.
    Exists(u32),

    /// ### 7.3.2. RECENT Response
    ///
    /// The RECENT response reports the number of messages with the
    /// \Recent flag set.  This response occurs as a result of a SELECT or
    /// EXAMINE command, and if the size of the mailbox changes (e.g., new
    /// messages).
    ///
    ///   Note: It is not guaranteed that the message sequence
    ///   numbers of recent messages will be a contiguous range of
    ///   the highest n messages in the mailbox (where n is the
    ///   value reported by the RECENT response).  Examples of
    ///   situations in which this is not the case are: multiple
    ///   clients having the same mailbox open (the first session
    ///   to be notified will see it as recent, others will
    ///   probably see it as non-recent), and when the mailbox is
    ///   re-ordered by a non-IMAP agent.
    ///
    ///   The only reliable way to identify recent messages is to
    ///   look at message flags to see which have the \Recent flag
    ///   set, or to do a SEARCH RECENT.
    ///
    /// The update from the RECENT response MUST be recorded by the client.
    Recent(u32),

    // ## 7.4. Server Responses - Message Status
    //
    // These responses are always untagged.  This is how message data are
    // transmitted from the server to the client, often as a result of a
    // command with the same name.  Immediately following the "*" token is a
    // number that represents a message sequence number.
    /// ### 7.4.1. EXPUNGE Response
    ///
    /// The EXPUNGE response reports that the specified message sequence
    /// number has been permanently removed from the mailbox.  The message
    /// sequence number for each successive message in the mailbox is
    /// immediately decremented by 1, and this decrement is reflected in
    /// message sequence numbers in subsequent responses (including other
    /// untagged EXPUNGE responses).
    ///
    /// The EXPUNGE response also decrements the number of messages in the
    /// mailbox; it is not necessary to send an EXISTS response with the
    /// new value.
    ///
    /// As a result of the immediate decrement rule, message sequence
    /// numbers that appear in a set of successive EXPUNGE responses
    /// depend upon whether the messages are removed starting from lower
    /// numbers to higher numbers, or from higher numbers to lower
    /// numbers.  For example, if the last 5 messages in a 9-message
    /// mailbox are expunged, a "lower to higher" server will send five
    /// untagged EXPUNGE responses for message sequence number 5, whereas
    /// a "higher to lower server" will send successive untagged EXPUNGE
    /// responses for message sequence numbers 9, 8, 7, 6, and 5.
    ///
    /// An EXPUNGE response MUST NOT be sent when no command is in
    /// progress, nor while responding to a FETCH, STORE, or SEARCH
    /// command.  This rule is necessary to prevent a loss of
    /// synchronization of message sequence numbers between client and
    /// server.  A command is not "in progress" until the complete command
    /// has been received; in particular, a command is not "in progress"
    /// during the negotiation of command continuation.
    ///
    ///   Note: UID FETCH, UID STORE, and UID SEARCH are different
    ///   commands from FETCH, STORE, and SEARCH.  An EXPUNGE
    ///   response MAY be sent during a UID command.
    ///
    /// The update from the EXPUNGE response MUST be recorded by the client.
    Expunge(NonZeroU32),

    /// ### 7.4.2. FETCH Response
    ///
    /// The FETCH response returns data about a message to the client.
    /// The data are pairs of data item names and their values in
    /// parentheses.  This response occurs as the result of a FETCH or
    /// STORE command, as well as by unilateral server decision (e.g.,
    /// flag updates).
    Fetch {
        /// Sequence number.
        seq: NonZeroU32,
        /// Message data items.
        items: NonEmptyVec<MessageDataItem<'a>>,
    },

    #[cfg(feature = "ext_enable")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_enable")))]
    Enabled {
        capabilities: Vec<CapabilityEnable<'a>>,
    },

    #[cfg(feature = "ext_quota")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_quota")))]
    Quota {
        /// Quota root.
        root: AString<'a>,
        /// List of quotas.
        quotas: NonEmptyVec<QuotaGet<'a>>,
    },

    #[cfg(feature = "ext_quota")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_quota")))]
    QuotaRoot {
        /// Mailbox name.
        mailbox: Mailbox<'a>,
        /// List of quota roots.
        roots: Vec<AString<'a>>,
    },
}

impl<'a> Data<'a> {
    pub fn capability<C>(caps: C) -> Result<Self, C::Error>
    where
        C: TryInto<NonEmptyVec<Capability<'a>>>,
    {
        Ok(Self::Capability(caps.try_into()?))
    }

    // TODO
    // pub fn list() -> Self {
    //     unimplemented!()
    // }

    // TODO
    // pub fn lsub() -> Self {
    //     unimplemented!()
    // }

    // TODO
    // pub fn status() -> Self {
    //     unimplemented!()
    // }

    // TODO
    // pub fn search() -> Self {
    //     unimplemented!()
    // }

    // TODO
    // pub fn flags() -> Self {
    //     unimplemented!()
    // }

    pub fn expunge(seq: u32) -> Result<Self, TryFromIntError> {
        Ok(Self::Expunge(NonZeroU32::try_from(seq)?))
    }

    pub fn fetch<S, I>(seq: S, items: I) -> Result<Self, FetchError<S::Error, I::Error>>
    where
        S: TryInto<NonZeroU32>,
        I: TryInto<NonEmptyVec<MessageDataItem<'a>>>,
    {
        let seq = seq.try_into().map_err(FetchError::SeqOrUid)?;
        let items = items.try_into().map_err(FetchError::InvalidItems)?;

        Ok(Self::Fetch { seq, items })
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum FetchError<S, I> {
    #[error("Invalid sequence or UID: {0:?}")]
    SeqOrUid(S),
    #[error("Invalid items: {0:?}")]
    InvalidItems(I),
}

/// ## 7.5. Server Responses - Command Continuation Request
///
/// The command continuation request response is indicated by a "+" token
/// instead of a tag.  This form of response indicates that the server is
/// ready to accept the continuation of a command from the client.  The
/// remainder of this response is a line of text.
///
/// This response is used in the AUTHENTICATE command to transmit server
/// data to the client, and request additional client data.  This
/// response is also used if an argument to any command is a literal.
///
/// The client is not permitted to send the octets of the literal unless
/// the server indicates that it is expected.  This permits the server to
/// process commands and reject errors on a line-by-line basis.  The
/// remainder of the command, including the CRLF that terminates a
/// command, follows the octets of the literal.  If there are any
/// additional command arguments, the literal octets are followed by a
/// space and those arguments.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[doc(alias = "Continuation")]
#[doc(alias = "ContinuationRequest")]
#[doc(alias = "CommandContinuationRequest")]
pub enum Continue<'a> {
    Basic(ContinueBasic<'a>),
    Base64(Cow<'a, [u8]>),
}

impl<'a> Continue<'a> {
    pub fn basic<T>(code: Option<Code<'a>>, text: T) -> Result<Self, ContinueError<T::Error>>
    where
        T: TryInto<Text<'a>>,
    {
        Ok(Continue::Basic(ContinueBasic::new(code, text)?))
    }

    pub fn base64<'data: 'a, D>(data: D) -> Self
    where
        D: Into<Cow<'data, [u8]>>,
    {
        Continue::Base64(data.into())
    }
}

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContinueBasic<'a> {
    code: Option<Code<'a>>,
    text: Text<'a>,
}

impl<'a> ContinueBasic<'a> {
    /// Create a basic continuation request.
    ///
    /// Note: To avoid ambiguities in the IMAP standard, this constructor ensures that:
    /// * iff `code` is `None`, `text` must not start with `[`.
    /// * iff `code` is `None`, `text` must *not* be valid according to base64.
    /// Otherwise, we could send a `Continue::Basic` that is interpreted as `Continue::Base64`.
    pub fn new<T>(code: Option<Code<'a>>, text: T) -> Result<Self, ContinueError<T::Error>>
    where
        T: TryInto<Text<'a>>,
    {
        let text = text.try_into().map_err(ContinueError::Text)?;

        // Ambiguity #1
        if code.is_none() && text.as_ref().starts_with('[') {
            return Err(ContinueError::Ambiguity);
        }

        // Ambiguity #2
        if code.is_none() && _base64.decode(text.inner()).is_ok() {
            return Err(ContinueError::Ambiguity);
        }

        Ok(Self { code, text })
    }

    pub fn code(&self) -> Option<&Code<'a>> {
        self.code.as_ref()
    }

    pub fn text(&self) -> &Text<'a> {
        &self.text
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum ContinueError<T> {
    #[error("invalid text")]
    Text(T),
    #[error("ambiguity detected")]
    Ambiguity,
}

/// A response code consists of data inside square brackets in the form of an atom,
/// possibly followed by a space and arguments.  The response code
/// contains additional information or status codes for client software
/// beyond the OK/NO/BAD condition, and are defined when there is a
/// specific action that a client can take based upon the additional
/// information.
///
/// The currently defined response codes are:
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Code<'a> {
    /// `ALERT`
    ///
    /// The human-readable text contains a special alert that MUST be
    /// presented to the user in a fashion that calls the user's
    /// attention to the message.
    Alert,

    /// `BADCHARSET`
    ///
    /// Optionally followed by a parenthesized list of charsets.  A
    /// SEARCH failed because the given charset is not supported by
    /// this implementation.  If the optional list of charsets is
    /// given, this lists the charsets that are supported by this
    /// implementation.
    BadCharset { allowed: Vec<Charset<'a>> },

    /// `CAPABILITY`
    ///
    /// Followed by a list of capabilities.  This can appear in the
    /// initial OK or PREAUTH response to transmit an initial
    /// capabilities list.  This makes it unnecessary for a client to
    /// send a separate CAPABILITY command if it recognizes this
    /// response.
    Capability(NonEmptyVec<Capability<'a>>), // FIXME(misuse): List must contain IMAP4REV1

    /// `PARSE`
    ///
    /// The human-readable text represents an error in parsing the
    /// [RFC-2822] header or [MIME-IMB] headers of a message in the
    /// mailbox.
    Parse,

    /// `PERMANENTFLAGS`
    ///
    /// Followed by a parenthesized list of flags, indicates which of
    /// the known flags the client can change permanently.  Any flags
    /// that are in the FLAGS untagged response, but not the
    /// PERMANENTFLAGS list, can not be set permanently.  If the client
    /// attempts to STORE a flag that is not in the PERMANENTFLAGS
    /// list, the server will either ignore the change or store the
    /// state change for the remainder of the current session only.
    /// The PERMANENTFLAGS list can also include the special flag \*,
    /// which indicates that it is possible to create new keywords by
    /// attempting to store those flags in the mailbox.
    PermanentFlags(Vec<FlagPerm<'a>>),

    /// `READ-ONLY`
    ///
    /// The mailbox is selected read-only, or its access while selected
    /// has changed from read-write to read-only.
    ReadOnly,

    /// `READ-WRITE`
    ///
    /// The mailbox is selected read-write, or its access while
    /// selected has changed from read-only to read-write.
    ReadWrite,

    /// `TRYCREATE`
    ///
    /// An APPEND or COPY attempt is failing because the target mailbox
    /// does not exist (as opposed to some other reason).  This is a
    /// hint to the client that the operation can succeed if the
    /// mailbox is first created by the CREATE command.
    TryCreate,

    /// `UIDNEXT`
    ///
    /// Followed by a decimal number, indicates the next unique
    /// identifier value.  Refer to section 2.3.1.1 for more
    /// information.
    UidNext(NonZeroU32),

    /// `UIDVALIDITY`
    ///
    /// Followed by a decimal number, indicates the unique identifier
    /// validity value.  Refer to section 2.3.1.1 for more information.
    UidValidity(NonZeroU32),

    /// `UNSEEN`
    ///
    /// Followed by a decimal number, indicates the number of the first
    /// message without the \Seen flag set.
    Unseen(NonZeroU32),

    /// IMAP4 Login Referrals (RFC 2221)
    // TODO(misuse): the imap url is more complicated than that...
    #[cfg(any(feature = "ext_mailbox_referrals", feature = "ext_login_referrals"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "ext_mailbox_referrals", feature = "ext_login_referrals")))
    )]
    Referral(Cow<'a, str>),

    #[cfg(feature = "ext_compress")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_compress")))]
    CompressionActive,

    /// SHOULD be returned in the tagged NO response to an APPEND/COPY/MOVE when the addition of the
    /// message(s) puts the target mailbox over any one of its quota limits.
    #[cfg(feature = "ext_quota")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_quota")))]
    OverQuota,

    /// Server got a non-synchronizing literal larger than 4096 bytes.
    #[cfg(feature = "ext_literal")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_literal")))]
    TooBig,

    /// Additional response codes defined by particular client or server
    /// implementations SHOULD be prefixed with an "X" until they are
    /// added to a revision of this protocol.  Client implementations
    /// SHOULD ignore response codes that they do not recognize.
    ///
    /// ---
    ///
    /// ```abnf
    /// atom [SP 1*<any TEXT-CHAR except "]">]`
    /// ```
    ///
    /// Note: We use this as a fallback for everything that was not recognized as
    ///       `Code`. This includes, e.g., variants with missing parameters, etc.
    Other(CodeOther<'a>),
}

impl<'a> Code<'a> {
    pub fn badcharset(allowed: Vec<Charset<'a>>) -> Self {
        Self::BadCharset { allowed }
    }

    pub fn capability<C>(caps: C) -> Result<Self, C::Error>
    where
        C: TryInto<NonEmptyVec<Capability<'a>>>,
    {
        Ok(Self::Capability(caps.try_into()?))
    }

    pub fn permanentflags(flags: Vec<FlagPerm<'a>>) -> Self {
        Self::PermanentFlags(flags)
    }

    pub fn uidnext(uidnext: u32) -> Result<Self, TryFromIntError> {
        Ok(Self::UidNext(NonZeroU32::try_from(uidnext)?))
    }

    pub fn uidvalidity(uidnext: u32) -> Result<Self, TryFromIntError> {
        Ok(Self::UidValidity(NonZeroU32::try_from(uidnext)?))
    }

    pub fn unseen(uidnext: u32) -> Result<Self, TryFromIntError> {
        Ok(Self::Unseen(NonZeroU32::try_from(uidnext)?))
    }
}

/// An (unknown) code.
///
/// It's guaranteed that this type can't represent any code from [`Code`].
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CodeOther<'a>(Cow<'a, [u8]>);

// We want a more readable `Debug` implementation.
impl<'a> Debug for CodeOther<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        struct BStr<'a>(&'a Cow<'a, [u8]>);

        impl<'a> Debug for BStr<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "b\"{}\"",
                    crate::utils::escape_byte_string(self.0.as_ref())
                )
            }
        }

        f.debug_tuple("CodeOther").field(&BStr(&self.0)).finish()
    }
}

impl<'a> CodeOther<'a> {
    /// Constructs an unsupported code without validation.
    ///
    /// # Warning: IMAP conformance
    ///
    /// The caller must ensure that `data` is valid. Failing to do so may create invalid/unparsable
    /// IMAP messages, or even produce unintended protocol flows. Do not call this constructor with
    /// untrusted data.
    #[cfg(feature = "unvalidated")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unvalidated")))]
    pub fn unvalidated<D: 'a>(data: D) -> Self
    where
        D: Into<Cow<'a, [u8]>>,
    {
        Self(data.into())
    }

    pub fn inner(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Capability<'a> {
    Imap4Rev1,
    Auth(AuthMechanism<'a>),
    #[cfg(feature = "starttls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "starttls")))]
    LoginDisabled,
    #[cfg(feature = "starttls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "starttls")))]
    StartTls,
    #[cfg(feature = "ext_idle")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_idle")))]
    /// See RFC 2177.
    Idle,
    /// See RFC 2193.
    #[cfg(feature = "ext_mailbox_referrals")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_mailbox_referrals")))]
    MailboxReferrals,
    /// See RFC 2221.
    #[cfg(feature = "ext_login_referrals")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_login_referrals")))]
    LoginReferrals,
    #[cfg(feature = "ext_sasl_ir")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_sasl_ir")))]
    SaslIr,
    /// See RFC 5161.
    #[cfg(feature = "ext_enable")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_enable")))]
    Enable,
    #[cfg(feature = "ext_compress")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_compress")))]
    Compress {
        algorithm: CompressionAlgorithm,
    },
    /// See RFC 2087 and RFC 9208
    #[cfg(feature = "ext_quota")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_quota")))]
    Quota,
    /// See RFC 9208.
    #[cfg(feature = "ext_quota")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_quota")))]
    QuotaRes(Resource<'a>),
    /// See RFC 9208.
    #[cfg(feature = "ext_quota")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_quota")))]
    QuotaSet,
    /// See RFC 7888.
    #[cfg(feature = "ext_literal")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_literal")))]
    LiteralPlus,
    #[cfg(feature = "ext_literal")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_literal")))]
    LiteralMinus,
    /// See RFC 6851.
    #[cfg(feature = "ext_move")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_move")))]
    Move,
    /// Other/Unknown
    Other(CapabilityOther<'a>),
}

impl<'a> Display for Capability<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::Imap4Rev1 => write!(f, "IMAP4REV1"),
            Self::Auth(mechanism) => write!(f, "AUTH={}", mechanism),
            #[cfg(feature = "starttls")]
            Self::LoginDisabled => write!(f, "LOGINDISABLED"),
            #[cfg(feature = "starttls")]
            Self::StartTls => write!(f, "STARTTLS"),
            #[cfg(feature = "ext_mailbox_referrals")]
            Self::MailboxReferrals => write!(f, "MAILBOX-REFERRALS"),
            #[cfg(feature = "ext_login_referrals")]
            Self::LoginReferrals => write!(f, "LOGIN-REFERRALS"),
            #[cfg(feature = "ext_sasl_ir")]
            Self::SaslIr => write!(f, "SASL-IR"),
            #[cfg(feature = "ext_idle")]
            Self::Idle => write!(f, "IDLE"),
            #[cfg(feature = "ext_enable")]
            Self::Enable => write!(f, "ENABLE"),
            #[cfg(feature = "ext_compress")]
            Self::Compress { algorithm } => write!(f, "COMPRESS={}", algorithm),
            #[cfg(feature = "ext_quota")]
            Self::Quota => write!(f, "QUOTA"),
            #[cfg(feature = "ext_quota")]
            Self::QuotaRes(resource) => write!(f, "QUOTA=RES-{}", resource),
            #[cfg(feature = "ext_quota")]
            Self::QuotaSet => write!(f, "QUOTASET"),
            #[cfg(feature = "ext_literal")]
            Self::LiteralPlus => write!(f, "LITERAL+"),
            #[cfg(feature = "ext_literal")]
            Self::LiteralMinus => write!(f, "LITERAL-"),
            #[cfg(feature = "ext_move")]
            Self::Move => write!(f, "MOVE"),
            Self::Other(other) => write!(f, "{}", other.0),
        }
    }
}

impl_try_from!(Atom<'a>, 'a, &'a [u8], Capability<'a>);
impl_try_from!(Atom<'a>, 'a, Vec<u8>, Capability<'a>);
impl_try_from!(Atom<'a>, 'a, &'a str, Capability<'a>);
impl_try_from!(Atom<'a>, 'a, String, Capability<'a>);

impl<'a> From<Atom<'a>> for Capability<'a> {
    fn from(atom: Atom<'a>) -> Self {
        fn split_once_cow<'a>(
            cow: Cow<'a, str>,
            pattern: &str,
        ) -> Option<(Cow<'a, str>, Cow<'a, str>)> {
            match cow {
                Cow::Borrowed(str) => {
                    if let Some((left, right)) = str.split_once(pattern) {
                        return Some((Cow::Borrowed(left), Cow::Borrowed(right)));
                    }

                    None
                }
                Cow::Owned(string) => {
                    // TODO(efficiency)
                    if let Some((left, right)) = string.split_once(pattern) {
                        return Some((Cow::Owned(left.to_owned()), Cow::Owned(right.to_owned())));
                    }

                    None
                }
            }
        }

        let cow = atom.into_inner();

        match cow.to_ascii_lowercase().as_ref() {
            "imap4rev1" => Self::Imap4Rev1,
            #[cfg(feature = "starttls")]
            "logindisabled" => Self::LoginDisabled,
            #[cfg(feature = "starttls")]
            "starttls" => Self::StartTls,
            #[cfg(feature = "ext_idle")]
            "idle" => Self::Idle,
            #[cfg(feature = "ext_mailbox_referrals")]
            "mailbox-referrals" => Self::MailboxReferrals,
            #[cfg(feature = "ext_login_referrals")]
            "login-referrals" => Self::LoginReferrals,
            #[cfg(feature = "ext_sasl_ir")]
            "sasl-ir" => Self::SaslIr,
            #[cfg(feature = "ext_enable")]
            "enable" => Self::Enable,
            #[cfg(feature = "ext_quota")]
            "quota" => Self::Quota,
            #[cfg(feature = "ext_quota")]
            "quotaset" => Self::QuotaSet,
            #[cfg(feature = "ext_literal")]
            "literal+" => Self::LiteralPlus,
            #[cfg(feature = "ext_literal")]
            "literal-" => Self::LiteralMinus,
            #[cfg(feature = "ext_move")]
            "move" => Self::Move,
            _ => {
                // TODO(efficiency)
                if let Some((left, right)) = split_once_cow(cow.clone(), "=") {
                    match left.as_ref().to_ascii_lowercase().as_ref() {
                        "auth" => {
                            if let Ok(mechanism) = AuthMechanism::try_from(right) {
                                return Self::Auth(mechanism);
                            }
                        }
                        #[cfg(feature = "ext_compress")]
                        "compress" => {
                            if let Ok(atom) = Atom::try_from(right) {
                                if let Ok(algorithm) = CompressionAlgorithm::try_from(atom) {
                                    return Self::Compress { algorithm };
                                }
                            }
                        }
                        #[cfg(feature = "ext_quota")]
                        "quota" => {
                            if let Some((_, right)) =
                                right.as_ref().to_ascii_lowercase().split_once("res-")
                            {
                                // TODO(efficiency)
                                if let Ok(resource) = Resource::try_from(right.to_owned()) {
                                    return Self::QuotaRes(resource);
                                }
                            }
                        }
                        _ => {}
                    }
                }

                Self::Other(CapabilityOther(Atom(cow)))
            }
        }
    }
}

/// An (unknown) capability.
///
/// It's guaranteed that this type can't represent any capability from [`Capability`].
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityOther<'a>(Atom<'a>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_data() {
        let _ = Data::capability(vec![Capability::Imap4Rev1]).unwrap();
        let _ = Data::fetch(1, vec![MessageDataItem::Rfc822Size(123)]).unwrap();
    }

    #[test]
    fn test_conversion_continue_failing() {
        let tests = [
            Continue::basic(None, ""),
            Continue::basic(Some(Code::ReadWrite), ""),
        ];

        for test in tests {
            println!("{:?}", test);
            assert!(test.is_err());
        }
    }
}
