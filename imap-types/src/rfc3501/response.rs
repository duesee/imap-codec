//! # 7. Server Responses

use std::{borrow::Cow, convert::TryInto, num::NonZeroU32};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "ext_compress")]
use crate::extensions::rfc4987::CompressionAlgorithm;
use crate::{
    core::{Atom, Charset, NonEmptyVec, QuotedChar, Tag, Text},
    fetch_attributes::FetchAttributeValue,
    flag::{Flag, FlagNameAttribute},
    mailbox::Mailbox,
    status_attributes::StatusAttributeValue,
    AuthMechanism,
};

/// Server responses are in three forms.
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
    Continuation(Continuation<'a>),
}

/// ## 7.1. Server Responses - Status Responses
///
/// Status responses are OK, NO, BAD, PREAUTH and BYE.
/// OK, NO, and BAD can be tagged or untagged.
/// PREAUTH and BYE are always untagged.
/// Status responses MAY include an OPTIONAL "response code" (see [`Code`](crate::response::Code).)
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
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

    /// ### 7.1.4. PREAUTH Response
    ///
    /// The PREAUTH response is always untagged, and is one of three
    /// possible greetings at connection startup.  It indicates that the
    /// connection has already been authenticated by external means; thus
    /// no LOGIN command is needed.
    PreAuth {
        /// Response code (optional)
        code: Option<Code<'a>>,
        /// Human-readable text (must be at least 1 character!)
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
    pub fn greeting(code: Option<Code<'a>>, text: &'a str) -> Result<Self, ()> {
        Ok(Status::Ok {
            tag: None,
            code,
            text: text.try_into()?,
        })
    }

    pub fn ok(tag: Option<Tag<'a>>, code: Option<Code<'a>>, text: &'a str) -> Result<Self, ()> {
        Ok(Status::Ok {
            tag,
            code,
            text: text.try_into()?,
        })
    }

    pub fn no(tag: Option<Tag<'a>>, code: Option<Code<'a>>, text: &'a str) -> Result<Self, ()> {
        Ok(Status::No {
            tag,
            code,
            text: text.try_into()?,
        })
    }

    pub fn bad(tag: Option<Tag<'a>>, code: Option<Code<'a>>, text: &'a str) -> Result<Self, ()> {
        Ok(Status::Bad {
            tag,
            code,
            text: text.try_into()?,
        })
    }

    pub fn preauth(code: Option<Code<'a>>, text: &'a str) -> Result<Self, ()> {
        Ok(Status::PreAuth {
            code,
            text: text.try_into()?,
        })
    }

    pub fn bye(code: Option<Code<'a>>, text: &'a str) -> Result<Self, ()> {
        Ok(Status::Bye {
            code,
            text: text.try_into()?,
        })
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
        attributes: Vec<StatusAttributeValue>,
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
        /// Message SEQ or UID
        seq_or_uid: NonZeroU32,
        /// Message data
        attributes: NonEmptyVec<FetchAttributeValue<'a>>,
    },

    #[cfg(feature = "ext_enable")]
    Enabled { capabilities: Vec<Capability<'a>> },
}

impl<'a> Data<'a> {
    pub fn capability<C>(caps: C) -> Result<Data<'a>, C::Error>
    where
        C: TryInto<NonEmptyVec<Capability<'a>>>,
    {
        Ok(Data::Capability(caps.try_into()?))
    }

    // TODO: implement other methods

    pub fn fetch<I, A>(seq_or_uid: I, attributes: A) -> Result<Data<'a>, ()>
    where
        I: TryInto<NonZeroU32>,
        A: TryInto<NonEmptyVec<FetchAttributeValue<'a>>>,
    {
        Ok(Data::Fetch {
            seq_or_uid: seq_or_uid.try_into().map_err(|_| ())?, // TODO: better error
            attributes: attributes.try_into().map_err(|_| ())?, // TODO: better error
        })
    }

    // TODO: implement other methods
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
pub enum Continuation<'a> {
    Basic {
        code: Option<Code<'a>>,
        text: Text<'a>,
    },
    Base64(Cow<'a, [u8]>),
}

impl<'a> Continuation<'a> {
    pub fn basic(code: Option<Code<'a>>, text: &'a str) -> Result<Self, ()> {
        Ok(Continuation::Basic {
            code,
            text: text.try_into()?,
        })
    }

    pub fn base64(data: &'a [u8]) -> Self {
        Continuation::Base64(Cow::Borrowed(data))
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
    BadCharset(Vec<Charset<'a>>),

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
    PermanentFlags(Vec<Flag<'a>>),

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

    /// Additional response codes defined by particular client or server
    /// implementations SHOULD be prefixed with an "X" until they are
    /// added to a revision of this protocol.  Client implementations
    /// SHOULD ignore response codes that they do not recognize.
    Other(Atom<'a>, Option<Cow<'a, str>>),

    /// IMAP4 Login Referrals (RFC 2221)
    Referral(Cow<'a, str>), // TODO(misuse): the imap url is more complicated than that...

    #[cfg(feature = "ext_compress")]
    CompressionActive,
}

impl<'a> Code<'a> {
    pub fn capability<C>(caps: C) -> Result<Self, C::Error>
    where
        C: TryInto<NonEmptyVec<Capability<'a>>>,
    {
        Ok(Code::Capability(caps.try_into()?))
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability<'a> {
    Imap4Rev1,
    Auth(AuthMechanism<'a>),
    LoginDisabled,
    #[cfg(feature = "starttls")]
    StartTls,
    // ---
    #[cfg(feature = "ext_idle")]
    Idle, // RFC 2177
    MailboxReferrals, // RFC 2193
    LoginReferrals,   // RFC 2221
    SaslIr,           // RFC 4959
    #[cfg(feature = "ext_enable")]
    Enable, // RFC 5161
    #[cfg(feature = "ext_compress")]
    Compress {
        algorithm: CompressionAlgorithm,
    },
    // --- Other ---
    // TODO: Is this a good idea?
    // FIXME: mark this enum as non-exhaustive at least?
    // FIXME: case-sensitive when compared
    Other(Atom<'a>),
}

#[cfg(test)]
mod test {
    use std::convert::TryFrom;

    use super::*;
    use crate::codec::Encode;

    #[test]
    fn test_status() {
        let tests: Vec<(_, &[u8])> = vec![
            // tagged; Ok, No, Bad
            (
                Status::ok(
                    Some(Tag::try_from("A1").unwrap()),
                    Some(Code::Alert),
                    "hello",
                ),
                b"A1 OK [ALERT] hello\r\n",
            ),
            (
                Status::no(
                    Some(Tag::try_from("A1").unwrap()),
                    Some(Code::Alert),
                    "hello",
                ),
                b"A1 NO [ALERT] hello\r\n",
            ),
            (
                Status::bad(
                    Some(Tag::try_from("A1").unwrap()),
                    Some(Code::Alert),
                    "hello",
                ),
                b"A1 BAD [ALERT] hello\r\n",
            ),
            (
                Status::ok(Some(Tag::try_from("A1").unwrap()), None, "hello"),
                b"A1 OK hello\r\n",
            ),
            (
                Status::no(Some(Tag::try_from("A1").unwrap()), None, "hello"),
                b"A1 NO hello\r\n",
            ),
            (
                Status::bad(Some(Tag::try_from("A1").unwrap()), None, "hello"),
                b"A1 BAD hello\r\n",
            ),
            // untagged; Ok, No, Bad
            (
                Status::ok(None, Some(Code::Alert), "hello"),
                b"* OK [ALERT] hello\r\n",
            ),
            (
                Status::no(None, Some(Code::Alert), "hello"),
                b"* NO [ALERT] hello\r\n",
            ),
            (
                Status::bad(None, Some(Code::Alert), "hello"),
                b"* BAD [ALERT] hello\r\n",
            ),
            (Status::ok(None, None, "hello"), b"* OK hello\r\n"),
            (Status::no(None, None, "hello"), b"* NO hello\r\n"),
            (Status::bad(None, None, "hello"), b"* BAD hello\r\n"),
            // preauth
            (
                Status::preauth(Some(Code::Alert), "hello"),
                b"* PREAUTH [ALERT] hello\r\n",
            ),
            // bye
            (
                Status::bye(Some(Code::Alert), "hello"),
                b"* BYE [ALERT] hello\r\n",
            ),
        ];

        for (constructed, serialized) in tests {
            let constructed = constructed.unwrap();
            let mut out = Vec::new();
            constructed.encode(&mut out).unwrap();

            assert_eq!(out, serialized.to_vec());
            // FIXME(#30)
            //assert_eq!(
            //    <Status as Codec>::deserialize(serialized).unwrap().1,
            //    parsed
            //);
        }
    }

    #[test]
    fn test_data() {
        let tests: Vec<(_, &[u8])> = vec![
            (
                Data::Capability(NonEmptyVec::try_from(vec![Capability::Imap4Rev1]).unwrap()),
                b"* CAPABILITY IMAP4REV1\r\n",
            ),
            (
                Data::List {
                    items: vec![FlagNameAttribute::Noselect],
                    delimiter: Some(QuotedChar::try_from('/').unwrap()),
                    mailbox: "bbb".try_into().unwrap(),
                },
                b"* LIST (\\Noselect) \"/\" bbb\r\n",
            ),
            (
                Data::Search(vec![
                    1.try_into().unwrap(),
                    2.try_into().unwrap(),
                    3.try_into().unwrap(),
                    42.try_into().unwrap(),
                ]),
                b"* SEARCH 1 2 3 42\r\n",
            ),
            (Data::Exists(42), b"* 42 EXISTS\r\n"),
            (Data::Recent(12345), b"* 12345 RECENT\r\n"),
            (Data::Expunge(123.try_into().unwrap()), b"* 123 EXPUNGE\r\n"),
        ];

        for (parsed, serialized) in tests.into_iter() {
            eprintln!("{:?}", parsed);
            let mut out = Vec::new();
            parsed.encode(&mut out).unwrap();
            assert_eq!(out, serialized.to_vec());
            // FIXME(#30):
            //assert_eq!(parsed, Data::deserialize(serialized).unwrap().1);
        }
    }

    #[test]
    fn test_data_constructors() {
        let _ = Data::capability(vec![Capability::Imap4Rev1]).unwrap();
        let _ = Data::fetch(1, vec![FetchAttributeValue::Rfc822Size(123)]).unwrap();
    }

    #[test]
    fn test_continuation() {
        let tests: Vec<(_, &[u8])> = vec![
            (Continuation::basic(None, "hello"), b"+ hello\r\n".as_ref()),
            (
                Continuation::basic(Some(Code::ReadWrite), "hello"),
                b"+ [READ-WRITE] hello\r\n",
            ),
        ];

        for (constructed, serialized) in tests.into_iter() {
            let constructed = constructed.unwrap();
            let mut out = Vec::new();
            constructed.encode(&mut out).unwrap();
            assert_eq!(out, serialized.to_vec());
            // FIXME(#30):
            //assert_eq!(parsed, Continuation::deserialize(serialized).unwrap().1);
        }
    }

    #[test]
    fn test_continuation_fail() {
        let tests: Vec<_> = vec![
            Continuation::basic(None, ""),
            Continuation::basic(Some(Code::ReadWrite), ""),
        ];

        for test in tests.into_iter() {
            println!("{:?}", test);
            assert!(test.is_err());
        }
    }

    #[test]
    fn test_bodystructure() {
        /*
        let tests: Vec<(_, &[u8])> = vec![
            (
                BodyStructure::Single(Body {
                    parameter_list: vec![],
                    id: NString::String(IString::try_from("ares").unwrap()),
                    description: NString::Nil,
                    content_transfer_encoding: IString::try_from("xxx").unwrap(),
                    size: 123,
                    specific: SpecificFields::Basic {
                        type_: IString::try_from("application").unwrap(),
                        subtype: IString::try_from("voodoo").unwrap(),
                    },
                    extension: None,
                }),
                b"(\"application\" \"voodoo\" nil \"ares\" nil \"xxx\" 123)",
            ),
            (
                BodyStructure::Single(Body {
                    parameter_list: vec![],
                    id: NString::Nil,
                    description: NString::Nil,
                    content_transfer_encoding: IString::try_from("xxx").unwrap(),
                    size: 123,
                    specific: SpecificFields::Text {
                        subtype: IString::try_from("plain").unwrap(),
                        number_of_lines: 14,
                    },
                    extension: None,
                }),
                b"(\"text\" \"plain\" nil nil nil \"xxx\" 123 14)",
            ),
            (
                BodyStructure::Single(Body {
                    parameter_list: vec![],
                    id: NString::Nil,
                    description: NString::Nil,
                    content_transfer_encoding: IString::try_from("xxx").unwrap(),
                    size: 123,
                    specific: SpecificFields::MessageRfc822 {
                        envelope: Envelope {
                            date: IString::try_from("date").unwrap(),
                            subject: IString::try_from("subject").unwrap(),
                            from: vec![],
                            sender: vec![],
                            reply_to: vec![],
                            to: vec![],
                            cc: vec![],
                            bcc: vec![],
                            in_reply_to: IString::try_from("in-reply-to".to_string()).unwrap(),
                            message_id: IString::try_from("message-id".to_string()).unwrap(),
                        },
                        body_structure: Box::new(BodyStructure::Single(Body {
                            parameter_list: vec![],
                            id: NString::Nil,
                            description: NString::Nil,
                            content_transfer_encoding: IString::try_from("xxx").unwrap(),
                            size: 123,
                            specific: SpecificFields::Basic {
                                type_: IString::try_from("application").unwrap(),
                                subtype: IString::try_from("voodoo").unwrap(),
                            },
                            extension: None,
                        })),
                        number_of_lines: 14,
                    },
                    extension: None,
                }),
                b"(\"message\" \"rfc822\" nil nil nil \"xxx\" 123 ????????? (\"application\" \"voodoo\" nil nil nil \"xxx\" 123) 14)",
            ),
        ];

        for (parsed, serialized) in tests.into_iter() {
            assert_eq!(
                String::from_utf8(parsed.serialize()).unwrap(),
                String::from_utf8(serialized.to_vec()).unwrap()
            );
            //assert_eq!(parsed, BodyStructure::deserialize(serialized).unwrap().1);
        }
        */
    }
}
