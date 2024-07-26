//! # 7. Server Responses

use std::{
    borrow::Cow,
    fmt::{Debug, Display, Formatter},
    num::{NonZeroU32, TryFromIntError},
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use base64::{engine::general_purpose::STANDARD as _base64, Engine};
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "ext_id")]
use crate::core::{IString, NString};
#[cfg(feature = "ext_metadata")]
use crate::extensions::metadata::{MetadataCode, MetadataResponse};
use crate::{
    auth::AuthMechanism,
    core::{impl_try_from, AString, Atom, Charset, QuotedChar, Tag, Text, Vec1},
    error::ValidationError,
    extensions::{
        compress::CompressionAlgorithm,
        enable::CapabilityEnable,
        quota::{QuotaGet, Resource},
        sort::SortAlgorithm,
        thread::{Thread, ThreadingAlgorithm},
        uidplus::UidSet,
    },
    fetch::MessageDataItem,
    flag::{Flag, FlagNameAttribute, FlagPerm},
    mailbox::Mailbox,
    response::error::{ContinueError, FetchError},
    status::StatusDataItem,
};

/// Greeting.
///
/// Note: Don't use `code: None` *and* a `text` that starts with "[" as this would be ambiguous in IMAP.
/// We could fix this but the fix would make this type unconformable to use.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
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
    ) -> Result<Self, ValidationError> {
        Ok(Greeting {
            kind,
            code,
            text: text.try_into()?,
        })
    }

    pub fn ok(code: Option<Code<'a>>, text: &'a str) -> Result<Self, ValidationError> {
        Ok(Greeting {
            kind: GreetingKind::Ok,
            code,
            text: text.try_into()?,
        })
    }

    pub fn preauth(code: Option<Code<'a>>, text: &'a str) -> Result<Self, ValidationError> {
        Ok(Greeting {
            kind: GreetingKind::PreAuth,
            code,
            text: text.try_into()?,
        })
    }

    pub fn bye(code: Option<Code<'a>>, text: &'a str) -> Result<Self, ValidationError> {
        Ok(Greeting {
            kind: GreetingKind::Bye,
            code,
            text: text.try_into()?,
        })
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToStatic)]
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

/// Response.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum Response<'a> {
    /// Command continuation request responses use the token "+" instead of a
    /// tag.  These responses are sent by the server to indicate acceptance
    /// of an incomplete client command and readiness for the remainder of
    /// the command.
    CommandContinuationRequest(CommandContinuationRequest<'a>),
    /// All server data is untagged. An untagged response is indicated by the
    /// token "*" instead of a tag. Untagged status responses indicate server
    /// greeting, or server status that does not indicate the completion of a
    /// command (for example, an impending system shutdown alert).
    Data(Data<'a>),
    /// Status responses can be tagged or untagged.  Tagged status responses
    /// indicate the completion result (OK, NO, or BAD status) of a client
    /// command, and have a tag matching the command.
    Status(Status<'a>),
}

/// Status response.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum Status<'a> {
    Untagged(StatusBody<'a>),
    Tagged(Tagged<'a>),
    Bye(Bye<'a>),
}

/// Status body.
///
/// Note: Don't use `code: None` *and* a `text` that starts with "[" as this would be ambiguous in IMAP.
/// We could fix this but the fix would make this type unconformable to use.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct StatusBody<'a> {
    /// Status kind.
    pub kind: StatusKind,
    /// Response code (optional).
    pub code: Option<Code<'a>>,
    /// Human-readable text that MAY be displayed to the user.
    ///
    /// Note: Must be at least 1 character.
    pub text: Text<'a>,
}

/// Status kind.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToStatic)]
pub enum StatusKind {
    /// Indicates an information from the server.
    ///
    /// * In [`Status::Tagged`], it indicates successful completion of the associated command.
    /// * In [`Status::Untagged`], it indicates an information-only message.
    Ok,
    /// Indicates an operational error from the server.
    ///
    /// * In [`Status::Tagged`], it indicates unsuccessful completion of the associated command.
    /// * In [`Status::Untagged`], it indicates a warning.
    No,
    /// Indicates a protocol-level error from the server.
    ///
    /// * In [`Status::Tagged`], it reports a protocol-level error in the client's command.
    /// * In [`Status::Untagged`], it indicates a protocol-level error for which the associated command can not be determined.
    Bad,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct Tagged<'a> {
    pub tag: Tag<'a>,
    pub body: StatusBody<'a>,
}

/// Indicates that the server is about to close the connection.
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct Bye<'a> {
    pub code: Option<Code<'a>>,
    pub text: Text<'a>,
}

impl<'a> Status<'a> {
    pub fn new<T>(
        tag: Option<Tag<'a>>,
        kind: StatusKind,
        code: Option<Code<'a>>,
        text: T,
    ) -> Result<Self, T::Error>
    where
        T: TryInto<Text<'a>>,
    {
        let body = StatusBody {
            kind,
            code,
            text: text.try_into()?,
        };

        match tag {
            Some(tag) => Ok(Self::Tagged(Tagged { tag, body })),
            None => Ok(Self::Untagged(body)),
        }
    }

    // FIXME(API)
    pub fn ok<T>(tag: Option<Tag<'a>>, code: Option<Code<'a>>, text: T) -> Result<Self, T::Error>
    where
        T: TryInto<Text<'a>>,
    {
        Self::new(tag, StatusKind::Ok, code, text)
    }

    // FIXME(API)
    pub fn no<T>(tag: Option<Tag<'a>>, code: Option<Code<'a>>, text: T) -> Result<Self, T::Error>
    where
        T: TryInto<Text<'a>>,
    {
        Self::new(tag, StatusKind::No, code, text)
    }

    // FIXME(API)
    pub fn bad<T>(tag: Option<Tag<'a>>, code: Option<Code<'a>>, text: T) -> Result<Self, T::Error>
    where
        T: TryInto<Text<'a>>,
    {
        Self::new(tag, StatusKind::Bad, code, text)
    }

    pub fn bye<T>(code: Option<Code<'a>>, text: T) -> Result<Self, T::Error>
    where
        T: TryInto<Text<'a>>,
    {
        Ok(Self::Bye(Bye {
            code,
            text: text.try_into()?,
        }))
    }

    // ---------------------------------------------------------------------------------------------

    pub fn tag(&self) -> Option<&Tag> {
        match self {
            Self::Tagged(Tagged { tag, .. }) => Some(tag),
            _ => None,
        }
    }

    pub fn code(&self) -> Option<&Code> {
        match self {
            Self::Untagged(StatusBody { code, .. })
            | Self::Tagged(Tagged {
                body: StatusBody { code, .. },
                ..
            })
            | Self::Bye(Bye { code, .. }) => code.as_ref(),
        }
    }

    pub fn text(&self) -> &Text {
        match self {
            Self::Untagged(StatusBody { text, .. })
            | Self::Tagged(Tagged {
                body: StatusBody { text, .. },
                ..
            })
            | Self::Bye(Bye { text, .. }) => text,
        }
    }
}

/// ## 7.2 - 7.4 Server and Mailbox Status; Mailbox Size; Message Status
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
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
    Capability(Vec1<Capability<'a>>),

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

    Sort(Vec<NonZeroU32>),

    Thread(Vec<Thread>),

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
        items: Vec1<MessageDataItem<'a>>,
    },

    Enabled {
        capabilities: Vec<CapabilityEnable<'a>>,
    },

    Quota {
        /// Quota root.
        root: AString<'a>,
        /// List of quotas.
        quotas: Vec1<QuotaGet<'a>>,
    },

    QuotaRoot {
        /// Mailbox name.
        mailbox: Mailbox<'a>,
        /// List of quota roots.
        roots: Vec<AString<'a>>,
    },

    #[cfg(feature = "ext_id")]
    /// ID Response
    Id {
        /// Parameters
        parameters: Option<Vec<(IString<'a>, NString<'a>)>>,
    },

    #[cfg(feature = "ext_metadata")]
    /// Metadata response
    Metadata {
        mailbox: Mailbox<'a>,
        items: MetadataResponse<'a>,
    },
}

impl<'a> Data<'a> {
    pub fn capability<C>(caps: C) -> Result<Self, C::Error>
    where
        C: TryInto<Vec1<Capability<'a>>>,
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
        I: TryInto<Vec1<MessageDataItem<'a>>>,
    {
        let seq = seq.try_into().map_err(FetchError::SeqOrUid)?;
        let items = items.try_into().map_err(FetchError::InvalidItems)?;

        Ok(Self::Fetch { seq, items })
    }
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
#[doc(alias = "Continue")]
#[doc(alias = "Continuation")]
#[doc(alias = "ContinuationRequest")]
pub enum CommandContinuationRequest<'a> {
    Basic(CommandContinuationRequestBasic<'a>),
    Base64(Cow<'a, [u8]>),
}

impl<'a> CommandContinuationRequest<'a> {
    pub fn basic<T>(code: Option<Code<'a>>, text: T) -> Result<Self, ContinueError<T::Error>>
    where
        T: TryInto<Text<'a>>,
    {
        Ok(Self::Basic(CommandContinuationRequestBasic::new(
            code, text,
        )?))
    }

    pub fn base64<'data: 'a, D>(data: D) -> Self
    where
        D: Into<Cow<'data, [u8]>>,
    {
        Self::Base64(data.into())
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(try_from = "CommandContinuationRequestBasicShadow")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct CommandContinuationRequestBasic<'a> {
    code: Option<Code<'a>>,
    text: Text<'a>,
}

/// Use shadow type to support validated deserialization
/// until `serde` provides built-in support for this case.
#[cfg(feature = "serde")]
#[derive(Deserialize, Debug)]
struct CommandContinuationRequestBasicShadow<'a> {
    code: Option<Code<'a>>,
    text: Text<'a>,
}

#[cfg(feature = "serde")]
impl<'a> TryFrom<CommandContinuationRequestBasicShadow<'a>>
    for CommandContinuationRequestBasic<'a>
{
    type Error = ContinueError<std::convert::Infallible>;

    fn try_from(value: CommandContinuationRequestBasicShadow<'a>) -> Result<Self, Self::Error> {
        Self::new(value.code, value.text)
    }
}

impl<'a> CommandContinuationRequestBasic<'a> {
    /// Create a basic continuation request.
    ///
    /// Note: To avoid ambiguities in the IMAP standard, this constructor ensures that:
    /// * if `code` is `None`, `text` must not start with `[`.
    /// * if `code` is `None`, `text` must *not* be valid according to base64.
    ///
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

/// A response code consists of data inside square brackets in the form of an atom,
/// possibly followed by a space and arguments.  The response code
/// contains additional information or status codes for client software
/// beyond the OK/NO/BAD condition, and are defined when there is a
/// specific action that a client can take based upon the additional
/// information.
///
/// The currently defined response codes are:
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
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
    BadCharset {
        allowed: Vec<Charset<'a>>,
    },

    /// `CAPABILITY`
    ///
    /// Followed by a list of capabilities.  This can appear in the
    /// initial OK or PREAUTH response to transmit an initial
    /// capabilities list.  This makes it unnecessary for a client to
    /// send a separate CAPABILITY command if it recognizes this
    /// response.
    Capability(Vec1<Capability<'a>>), // FIXME(misuse): List must contain IMAP4REV1

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

    CompressionActive,

    /// SHOULD be returned in the tagged NO response to an APPEND/COPY/MOVE when the addition of the
    /// message(s) puts the target mailbox over any one of its quota limits.
    OverQuota,

    /// Server got a non-synchronizing literal larger than 4096 bytes.
    TooBig,

    #[cfg(feature = "ext_metadata")]
    /// Metadata
    Metadata(MetadataCode),

    /// Server does not know how to decode the section's CTE.
    UnknownCte,

    /// Message has been appended to destination mailbox with that UID
    AppendUid {
        /// UIDVALIDITY of destination mailbox
        uid_validity: NonZeroU32,
        /// UID assigned to appended message in destination mailbox
        uid: NonZeroU32,
    },

    /// Message(s) have been copied to destination mailbox with stated UID(s)
    CopyUid {
        /// UIDVALIDITY of destination mailbox
        uid_validity: NonZeroU32,
        /// UIDs copied to destination mailbox
        source: UidSet,
        /// UIDs assigned in destination mailbox
        destination: UidSet,
    },

    UidNotSticky,

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
        C: TryInto<Vec1<Capability<'a>>>,
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Hash, ToStatic)]
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
    pub fn unvalidated<D>(data: D) -> Self
    where
        D: Into<Cow<'a, [u8]>>,
    {
        Self(data.into())
    }

    pub fn inner(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
#[non_exhaustive]
pub enum Capability<'a> {
    Imap4Rev1,
    Auth(AuthMechanism<'a>),
    LoginDisabled,
    #[cfg(feature = "starttls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "starttls")))]
    StartTls,
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
    SaslIr,
    /// See RFC 5161.
    Enable,
    Compress {
        algorithm: CompressionAlgorithm,
    },
    /// See RFC 2087 and RFC 9208
    Quota,
    /// See RFC 9208.
    QuotaRes(Resource<'a>),
    /// See RFC 9208.
    QuotaSet,
    /// See RFC 7888.
    LiteralPlus,
    LiteralMinus,
    /// See RFC 6851.
    Move,
    #[cfg(feature = "ext_id")]
    /// See RFC 2971.
    Id,
    /// See RFC 3691.
    Unselect,
    Sort(Option<SortAlgorithm<'a>>),
    Thread(ThreadingAlgorithm<'a>),
    #[cfg(feature = "ext_metadata")]
    /// Server supports (both) server annotations and mailbox annotations.
    Metadata,
    #[cfg(feature = "ext_metadata")]
    /// Server supports (only) server annotations.
    MetadataServer,
    /// IMAP4 Binary Content Extension
    Binary,
    /// UIDPLUS extension (RFC 4351)
    UidPlus,
    /// Other/Unknown
    Other(CapabilityOther<'a>),
}

impl<'a> Display for Capability<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::Imap4Rev1 => write!(f, "IMAP4REV1"),
            Self::Auth(mechanism) => write!(f, "AUTH={}", mechanism),
            Self::LoginDisabled => write!(f, "LOGINDISABLED"),
            #[cfg(feature = "starttls")]
            Self::StartTls => write!(f, "STARTTLS"),
            #[cfg(feature = "ext_mailbox_referrals")]
            Self::MailboxReferrals => write!(f, "MAILBOX-REFERRALS"),
            #[cfg(feature = "ext_login_referrals")]
            Self::LoginReferrals => write!(f, "LOGIN-REFERRALS"),
            Self::SaslIr => write!(f, "SASL-IR"),
            Self::Idle => write!(f, "IDLE"),
            Self::Enable => write!(f, "ENABLE"),
            Self::Compress { algorithm } => write!(f, "COMPRESS={}", algorithm),
            Self::Quota => write!(f, "QUOTA"),
            Self::QuotaRes(resource) => write!(f, "QUOTA=RES-{}", resource),
            Self::QuotaSet => write!(f, "QUOTASET"),
            Self::LiteralPlus => write!(f, "LITERAL+"),
            Self::LiteralMinus => write!(f, "LITERAL-"),
            Self::Move => write!(f, "MOVE"),
            #[cfg(feature = "ext_id")]
            Self::Id => write!(f, "ID"),
            Self::Unselect => write!(f, "UNSELECT"),
            Self::Sort(None) => write!(f, "SORT"),
            Self::Sort(Some(algorithm)) => write!(f, "SORT={}", algorithm),
            Self::Thread(algorithm) => write!(f, "THREAD={}", algorithm),
            #[cfg(feature = "ext_metadata")]
            Self::Metadata => write!(f, "METADATA"),
            #[cfg(feature = "ext_metadata")]
            Self::MetadataServer => write!(f, "METADATA-SERVER"),
            Self::Binary => write!(f, "BINARY"),
            Self::UidPlus => write!(f, "UIDPLUS"),
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
            "logindisabled" => Self::LoginDisabled,
            #[cfg(feature = "starttls")]
            "starttls" => Self::StartTls,
            "idle" => Self::Idle,
            #[cfg(feature = "ext_mailbox_referrals")]
            "mailbox-referrals" => Self::MailboxReferrals,
            #[cfg(feature = "ext_login_referrals")]
            "login-referrals" => Self::LoginReferrals,
            "sasl-ir" => Self::SaslIr,
            "enable" => Self::Enable,
            "quota" => Self::Quota,
            "quotaset" => Self::QuotaSet,
            "literal+" => Self::LiteralPlus,
            "literal-" => Self::LiteralMinus,
            "move" => Self::Move,
            #[cfg(feature = "ext_id")]
            "id" => Self::Id,
            "sort" => Self::Sort(None),
            #[cfg(feature = "ext_metadata")]
            "metadata" => Self::Metadata,
            #[cfg(feature = "ext_metadata")]
            "metadata-server" => Self::MetadataServer,
            "binary" => Self::Binary,
            "unselect" => Self::Unselect,
            "uidplus" => Self::UidPlus,
            _ => {
                // TODO(efficiency)
                if let Some((left, right)) = split_once_cow(cow.clone(), "=") {
                    match left.as_ref().to_ascii_lowercase().as_ref() {
                        "auth" => {
                            if let Ok(mechanism) = AuthMechanism::try_from(right) {
                                return Self::Auth(mechanism);
                            }
                        }
                        "compress" => {
                            if let Ok(atom) = Atom::try_from(right) {
                                if let Ok(algorithm) = CompressionAlgorithm::try_from(atom) {
                                    return Self::Compress { algorithm };
                                }
                            }
                        }
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
                        "sort" => {
                            if let Ok(atom) = Atom::try_from(right) {
                                return Self::Sort(Some(SortAlgorithm::from(atom)));
                            }
                        }
                        "thread" => {
                            if let Ok(atom) = Atom::try_from(right) {
                                return Self::Thread(ThreadingAlgorithm::from(atom));
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct CapabilityOther<'a>(Atom<'a>);

/// Error-related types.
pub mod error {
    use thiserror::Error;

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum ContinueError<T> {
        #[error("invalid text")]
        Text(T),
        #[error("ambiguity detected")]
        Ambiguity,
    }

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum FetchError<S, I> {
        #[error("Invalid sequence or UID: {0:?}")]
        SeqOrUid(S),
        #[error("Invalid items: {0:?}")]
        InvalidItems(I),
    }
}

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
            CommandContinuationRequest::basic(None, ""),
            CommandContinuationRequest::basic(Some(Code::ReadWrite), ""),
        ];

        for test in tests {
            println!("{:?}", test);
            assert!(test.is_err());
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialization_command_continuation_request_basic() {
        let valid_input = r#"{ "text": "Ready for additional command text" }"#;
        let invalid_input = r#"{ "text": "[Ready for additional command text" }"#;

        let request = serde_json::from_str::<CommandContinuationRequestBasic>(valid_input)
            .expect("valid input should deserialize successfully");
        assert_eq!(
            request,
            CommandContinuationRequestBasic {
                code: None,
                text: Text(Cow::Borrowed("Ready for additional command text"))
            }
        );

        let err = serde_json::from_str::<CommandContinuationRequestBasic>(invalid_input)
            .expect_err("invalid input should not deserialize successfully");
        assert_eq!(err.to_string(), r"ambiguity detected");
    }
}
