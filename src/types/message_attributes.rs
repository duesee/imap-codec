//! ## 2.3. Message Attributes
//!
//! In addition to message text, each message has several attributes
//! associated with it.  These attributes can be retrieved individually
//! or in conjunction with other attributes or message texts.

use serde::Deserialize;

// ### 2.3.1. Message Numbers
/*
Messages in IMAP4rev1 are accessed by one of two numbers; the unique
identifier or the message sequence number.
*/

// #### 2.3.1.1. Unique Identifier (UID) Message Attribute
/*
A 32-bit value assigned to each message, which when used with the
unique identifier validity value (see below) forms a 64-bit value
that MUST NOT refer to any other message in the mailbox or any
subsequent mailbox with the same name forever.  Unique identifiers
are assigned in a strictly ascending fashion in the mailbox; as each
message is added to the mailbox it is assigned a higher UID than the
message(s) which were added previously.  Unlike message sequence
numbers, unique identifiers are not necessarily contiguous.

The unique identifier of a message MUST NOT change during the
session, and SHOULD NOT change between sessions.  Any change of
unique identifiers between sessions MUST be detectable using the
UIDVALIDITY mechanism discussed below.  Persistent unique identifiers
are required for a client to resynchronize its state from a previous
session with the server (e.g., disconnected or offline access
clients); this is discussed further in [IMAP-DISC].

Associated with every mailbox are two values which aid in unique
identifier handling: the next unique identifier value and the unique
identifier validity value.

The next unique identifier value is the predicted value that will be
assigned to a new message in the mailbox.  Unless the unique
identifier validity also changes (see below), the next unique
identifier value MUST have the following two characteristics.  First,
the next unique identifier value MUST NOT change unless new messages
are added to the mailbox; and second, the next unique identifier
value MUST change whenever new messages are added to the mailbox,
even if those new messages are subsequently expunged.

     Note: The next unique identifier value is intended to
     provide a means for a client to determine whether any
     messages have been delivered to the mailbox since the
     previous time it checked this value.  It is not intended to
     provide any guarantee that any message will have this
     unique identifier.  A client can only assume, at the time
     that it obtains the next unique identifier value, that
     messages arriving after that time will have a UID greater
     than or equal to that value.

The unique identifier validity value is sent in a UIDVALIDITY
response code in an OK untagged response at mailbox selection time.
If unique identifiers from an earlier session fail to persist in this
session, the unique identifier validity value MUST be greater than
the one used in the earlier session.

     Note: Ideally, unique identifiers SHOULD persist at all
     times.  Although this specification recognizes that failure
     to persist can be unavoidable in certain server
     environments, it STRONGLY ENCOURAGES message store
     implementation techniques that avoid this problem.  For
     example:

      1) Unique identifiers MUST be strictly ascending in the
         mailbox at all times.  If the physical message store is
         re-ordered by a non-IMAP agent, this requires that the
         unique identifiers in the mailbox be regenerated, since
         the former unique identifiers are no longer strictly
         ascending as a result of the re-ordering.

      2) If the message store has no mechanism to store unique
         identifiers, it must regenerate unique identifiers at
         each session, and each session must have a unique
         UIDVALIDITY value.

      3) If the mailbox is deleted and a new mailbox with the
         same name is created at a later date, the server must
         either keep track of unique identifiers from the
         previous instance of the mailbox, or it must assign a
         new UIDVALIDITY value to the new instance of the
         mailbox.  A good UIDVALIDITY value to use in this case
         is a 32-bit representation of the creation date/time of
         the mailbox.  It is alright to use a constant such as
         1, but only if it guaranteed that unique identifiers
         will never be reused, even in the case of a mailbox
         being deleted (or renamed) and a new mailbox by the
         same name created at some future time.

      4) The combination of mailbox name, UIDVALIDITY, and UID
         must refer to a single immutable message on that server
         forever.  In particular, the internal date, [RFC-2822]
         size, envelope, body structure, and message texts
         (RFC822, RFC822.HEADER, RFC822.TEXT, and all BODY[...]
         fetch data items) must never change.  This does not
         include message numbers, nor does it include attributes
         that can be set by a STORE command (e.g., FLAGS).
*/

// #### 2.3.1.2. Message Sequence Number Message Attribute
/*
A relative position from 1 to the number of messages in the mailbox.
This position MUST be ordered by ascending unique identifier.  As
each new message is added, it is assigned a message sequence number
that is 1 higher than the number of messages in the mailbox before
that new message was added.

Message sequence numbers can be reassigned during the session.  For
example, when a message is permanently removed (expunged) from the
mailbox, the message sequence number for all subsequent messages is
decremented.  The number of messages in the mailbox is also
decremented.  Similarly, a new message can be assigned a message
sequence number that was once held by some other message prior to an
expunge.

In addition to accessing messages by relative position in the
mailbox, message sequence numbers can be used in mathematical
calculations.  For example, if an untagged "11 EXISTS" is received,
and previously an untagged "8 EXISTS" was received, three new
messages have arrived with message sequence numbers of 9, 10, and 11.
Another example, if message 287 in a 523 message mailbox has UID
12345, there are exactly 286 messages which have lesser UIDs and 236
messages which have greater UIDs.
*/

// ### 2.3.2. Flags Message Attribute

use crate::types::core::Atom;

/// A list of zero or more named tokens associated with the message.  A
/// flag is set by its addition to this list, and is cleared by its
/// removal.  There are two types of flags in IMAP4rev1. A flag of either
/// type can be permanent or session-only.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Flag {
    System(SystemFlag),
    Keyword(Keyword),
    Extension(Atom),
}

impl std::fmt::Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::System(system_flag) => write!(f, "{}", system_flag),
            Self::Keyword(_keyword) => unimplemented!(),
            Self::Extension(_atom) => unimplemented!(),
        }
    }
}

/// A system flag is a flag name that is pre-defined in this
/// specification.  All system flags begin with "\".  Certain system
/// flags (\Deleted and \Seen) have special semantics described
/// elsewhere.  The currently-defined system flags are:
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum SystemFlag {
    /// \Seen Message has been read
    Seen,
    /// \Answered Message has been answered
    Answered,
    /// \Flagged Message is "flagged" for urgent/special attention
    Flagged,
    /// \Deleted Message is "deleted" for removal by later EXPUNGE
    Deleted,
    /// \Draft Message has not completed composition (marked as a draft).
    Draft,
    /// \Recent Message is "recently" arrived in this mailbox.
    ///
    /// This session is the first session to have been notified about this
    /// message; if the session is read-write, subsequent sessions
    /// will not see \Recent set for this message.  This flag can not
    /// be altered by the client.
    ///
    /// If it is not possible to determine whether or not this
    /// session is the first session to be notified about a message,
    /// then that message SHOULD be considered recent.
    ///
    /// If multiple connections have the same mailbox selected
    /// simultaneously, it is undefined which of these connections
    /// will see newly-arrived messages with \Recent set and which
    /// will see it without \Recent set.
    Recent,
}

impl std::fmt::Display for SystemFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            SystemFlag::Seen => write!(f, "\\Seen"),
            SystemFlag::Answered => write!(f, "\\Answered"),
            SystemFlag::Flagged => write!(f, "\\Flagged"),
            SystemFlag::Deleted => write!(f, "\\Deleted"),
            SystemFlag::Draft => write!(f, "\\Draft"),
            SystemFlag::Recent => write!(f, "\\Recent"),
        }
    }
}

/// A keyword is defined by the server implementation.  Keywords do not
/// begin with "\".  Servers MAY permit the client to define new keywords
/// in the mailbox (see the description of the PERMANENTFLAGS response
/// code for more information).
pub type Keyword = Atom;

/// A flag can be permanent or session-only on a per-flag basis.
pub enum FlagType {
    /// Permanent flags are those which the client can add or remove from the
    /// message flags permanently; that is, concurrent and subsequent
    Permanent,
    /// sessions will see any change in permanent flags.  Changes to session
    /// flags are valid only in that session.
    SessionOnly,
}

// Note: The \Recent system flag is a special case of a
// session flag.  \Recent can not be used as an argument in a
// STORE or APPEND command, and thus can not be changed at
// all.

// ### 2.3.3. Internal Date Message Attribute
/*
The internal date and time of the message on the server.  This
is not the date and time in the [RFC-2822] header, but rather a
date and time which reflects when the message was received.  In
the case of messages delivered via [SMTP], this SHOULD be the
date and time of final delivery of the message as defined by
[SMTP].  In the case of messages delivered by the IMAP4rev1 COPY
command, this SHOULD be the internal date and time of the source
message.  In the case of messages delivered by the IMAP4rev1
APPEND command, this SHOULD be the date and time as specified in
the APPEND command description.  All other cases are
implementation defined.
*/

// ### 2.3.4. [RFC-2822] Size Message Attribute
/*
The number of octets in the message, as expressed in [RFC-2822]
format.
*/

// ### 2.3.5. Envelope Structure Message Attribute
/*
A parsed representation of the [RFC-2822] header of the message.
Note that the IMAP Envelope structure is not the same as an
[SMTP] envelope.
*/

// ### 2.3.6. Body Structure Message Attribute
/*
A parsed representation of the [MIME-IMB] body structure
information of the message.
*/
