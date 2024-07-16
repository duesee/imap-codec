//! Client Commands.
//!
//! See <https://tools.ietf.org/html/rfc3501#section-6>.

use std::borrow::Cow;

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "ext_id")]
use crate::core::{IString, NString};
#[cfg(feature = "ext_metadata")]
use crate::extensions::metadata::{Entry, EntryValue, GetMetadataOption};
use crate::{
    auth::AuthMechanism,
    command::error::{AppendError, CopyError, ListError, LoginError, RenameError},
    core::{AString, Charset, Literal, Tag, Vec1},
    datetime::DateTime,
    extensions::{
        binary::LiteralOrLiteral8, compress::CompressionAlgorithm, enable::CapabilityEnable,
        quota::QuotaSet, sort::SortCriterion, thread::ThreadingAlgorithm,
    },
    fetch::MacroOrMessageDataItemNames,
    flag::{Flag, StoreResponse, StoreType},
    mailbox::{ListMailbox, Mailbox},
    search::SearchKey,
    secret::Secret,
    sequence::SequenceSet,
    status::StatusDataItemName,
};

/// Command.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct Command<'a> {
    /// Tag.
    pub tag: Tag<'a>,
    /// Body, e.g., CAPABILITY, LOGIN, SELECT, etc.
    pub body: CommandBody<'a>,
}

impl<'a> Command<'a> {
    /// Create a new command.
    pub fn new<T>(tag: T, body: CommandBody<'a>) -> Result<Self, T::Error>
    where
        T: TryInto<Tag<'a>>,
    {
        Ok(Self {
            tag: tag.try_into()?,
            body,
        })
    }

    /// Get the command name.
    pub fn name(&self) -> &'static str {
        self.body.name()
    }
}

/// Command body.
///
/// This enum is used to encode all the different commands.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum CommandBody<'a> {
    // ----- Any State (see https://tools.ietf.org/html/rfc3501#section-6.1) -----
    /// ### 6.1.1.  CAPABILITY Command
    ///
    /// * Arguments:  none
    /// * Responses:  REQUIRED untagged response: CAPABILITY
    /// * Result:
    ///   * OK - capability completed
    ///   * BAD - command unknown or arguments invalid
    ///
    /// The CAPABILITY command requests a listing of capabilities that the
    /// server supports.  The server MUST send a single untagged
    /// CAPABILITY response with "IMAP4rev1" as one of the listed
    /// capabilities before the (tagged) OK response.
    ///
    /// A capability name which begins with "AUTH=" indicates that the
    /// server supports that particular authentication mechanism.  All
    /// such names are, by definition, part of this specification.  For
    /// example, the authorization capability for an experimental
    /// "blurdybloop" authenticator would be "AUTH=XBLURDYBLOOP" and not
    /// "XAUTH=BLURDYBLOOP" or "XAUTH=XBLURDYBLOOP".
    ///
    /// Other capability names refer to extensions, revisions, or
    /// amendments to this specification.  See the documentation of the
    /// CAPABILITY response for additional information.  No capabilities,
    /// beyond the base IMAP4rev1 set defined in this specification, are
    /// enabled without explicit client action to invoke the capability.
    ///
    /// Client and server implementations MUST implement the STARTTLS,
    /// LOGINDISABLED, and AUTH=PLAIN (described in [IMAP-TLS])
    /// capabilities.  See the Security Considerations section for
    /// important information.
    ///
    /// See the section entitled "Client Commands -
    /// Experimental/Expansion" for information about the form of site or
    /// implementation-specific capabilities.
    Capability,

    /// ### 6.1.2.  NOOP Command
    ///
    /// * Arguments:  none
    /// * Responses:  no specific responses for this command (but see below)
    /// * Result:
    ///   * OK - noop completed
    ///   * BAD - command unknown or arguments invalid
    ///
    /// The NOOP command always succeeds.  It does nothing.
    ///
    /// Since any command can return a status update as untagged data, the
    /// NOOP command can be used as a periodic poll for new messages or
    /// message status updates during a period of inactivity (this is the
    /// preferred method to do this).  The NOOP command can also be used
    /// to reset any inactivity autologout timer on the server.
    Noop,

    /// ### 6.1.3.  LOGOUT Command
    ///
    /// * Arguments:  none
    /// * Responses:  REQUIRED untagged response: BYE
    /// * Result:
    ///   * OK - logout completed
    ///   * BAD - command unknown or arguments invalid
    ///
    /// The LOGOUT command informs the server that the client is done with
    /// the connection.  The server MUST send a BYE untagged response
    /// before the (tagged) OK response, and then close the network
    /// connection.
    Logout,

    // ----- Not Authenticated State (https://tools.ietf.org/html/rfc3501#section-6.2) -----
    /// ### 6.2.1.  STARTTLS Command
    ///
    /// * Arguments:  none
    /// * Responses:  no specific response for this command
    /// * Result:
    ///   * OK - starttls completed, begin TLS negotiation
    ///   * BAD - command unknown or arguments invalid
    ///
    /// A \[TLS\] negotiation begins immediately after the CRLF at the end
    /// of the tagged OK response from the server.  Once a client issues a
    /// STARTTLS command, it MUST NOT issue further commands until a
    /// server response is seen and the \[TLS\] negotiation is complete.
    ///
    /// The server remains in the non-authenticated state, even if client
    /// credentials are supplied during the \[TLS\] negotiation.  This does
    /// not preclude an authentication mechanism such as EXTERNAL (defined
    /// in \[SASL\]) from using client identity determined by the \[TLS\]
    /// negotiation.
    ///
    /// Once \[TLS\] has been started, the client MUST discard cached
    /// information about server capabilities and SHOULD re-issue the
    /// CAPABILITY command.  This is necessary to protect against man-in-
    /// the-middle attacks which alter the capabilities list prior to
    /// STARTTLS.  The server MAY advertise different capabilities after
    /// STARTTLS.
    ///
    /// <div class="warning">
    /// This must only be used when the server advertised support for it sending the STARTTLS capability.
    ///
    /// Try to avoid STARTTLS using implicit TLS on port 993.
    /// </div>
    #[cfg(feature = "starttls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "starttls")))]
    StartTLS,

    /// ### 6.2.2.  AUTHENTICATE Command
    ///
    /// * Arguments:  authentication mechanism name
    /// * Responses:  continuation data can be requested
    /// * Result:
    ///   * OK - authenticate completed, now in authenticated state
    ///   * NO - authenticate failure: unsupported authentication
    ///          mechanism, credentials rejected
    ///   * BAD - command unknown or arguments invalid,
    ///           authentication exchange cancelled
    ///
    /// The AUTHENTICATE command indicates a \[SASL\] authentication
    /// mechanism to the server.  If the server supports the requested
    /// authentication mechanism, it performs an authentication protocol
    /// exchange to authenticate and identify the client.  It MAY also
    /// negotiate an OPTIONAL security layer for subsequent protocol
    /// interactions.  If the requested authentication mechanism is not
    /// supported, the server SHOULD reject the AUTHENTICATE command by
    /// sending a tagged NO response.
    ///
    /// The AUTHENTICATE command does not support the optional "initial
    /// response" feature of \[SASL\].  Section 5.1 of \[SASL\] specifies how
    /// to handle an authentication mechanism which uses an initial
    /// response.
    ///
    /// The service name specified by this protocol's profile of \[SASL\] is
    /// "imap".
    ///
    /// The authentication protocol exchange consists of a series of
    /// server challenges and client responses that are specific to the
    /// authentication mechanism.  A server challenge consists of a
    /// command continuation request response with the "+" token followed
    /// by a BASE64 encoded string.  The client response consists of a
    /// single line consisting of a BASE64 encoded string.  If the client
    /// wishes to cancel an authentication exchange, it issues a line
    /// consisting of a single "*".  If the server receives such a
    /// response, it MUST reject the AUTHENTICATE command by sending a
    /// tagged BAD response.
    ///
    /// If a security layer is negotiated through the \[SASL\]
    /// authentication exchange, it takes effect immediately following the
    /// CRLF that concludes the authentication exchange for the client,
    /// and the CRLF of the tagged OK response for the server.
    ///
    /// While client and server implementations MUST implement the
    /// AUTHENTICATE command itself, it is not required to implement any
    /// authentication mechanisms other than the PLAIN mechanism described
    /// in [IMAP-TLS].  Also, an authentication mechanism is not required
    /// to support any security layers.
    ///
    ///   Note: a server implementation MUST implement a
    ///   configuration in which it does NOT permit any plaintext
    ///   password mechanisms, unless either the STARTTLS command
    ///   has been negotiated or some other mechanism that
    ///   protects the session from password snooping has been
    ///   provided.  Server sites SHOULD NOT use any configuration
    ///   which permits a plaintext password mechanism without
    ///   such a protection mechanism against password snooping.
    ///   Client and server implementations SHOULD implement
    ///   additional \[SASL\] mechanisms that do not use plaintext
    ///   passwords, such the GSSAPI mechanism described in \[SASL\]
    ///   and/or the [DIGEST-MD5] mechanism.
    ///
    /// Servers and clients can support multiple authentication
    /// mechanisms.  The server SHOULD list its supported authentication
    /// mechanisms in the response to the CAPABILITY command so that the
    /// client knows which authentication mechanisms to use.
    ///
    /// A server MAY include a CAPABILITY response code in the tagged OK
    /// response of a successful AUTHENTICATE command in order to send
    /// capabilities automatically.  It is unnecessary for a client to
    /// send a separate CAPABILITY command if it recognizes these
    /// automatic capabilities.  This should only be done if a security
    /// layer was not negotiated by the AUTHENTICATE command, because the
    /// tagged OK response as part of an AUTHENTICATE command is not
    /// protected by encryption/integrity checking.  \[SASL\] requires the
    /// client to re-issue a CAPABILITY command in this case.
    ///
    /// If an AUTHENTICATE command fails with a NO response, the client
    /// MAY try another authentication mechanism by issuing another
    /// AUTHENTICATE command.  It MAY also attempt to authenticate by
    /// using the LOGIN command (see section 6.2.3 for more detail).  In
    /// other words, the client MAY request authentication types in
    /// decreasing order of preference, with the LOGIN command as a last
    /// resort.
    ///
    /// The authorization identity passed from the client to the server
    /// during the authentication exchange is interpreted by the server as
    /// the user name whose privileges the client is requesting.
    Authenticate {
        /// Authentication mechanism.
        mechanism: AuthMechanism<'a>,
        /// Initial response (if any).
        ///
        /// This type holds the raw binary data, i.e., a `Vec<u8>`, *not* the BASE64 string.
        ///
        /// <div class="warning">
        /// This extension must only be used when the server advertised support for it sending the SASL-IR capability.
        /// </div>
        initial_response: Option<Secret<Cow<'a, [u8]>>>,
    },

    /// ### 6.2.3.  LOGIN Command
    ///
    /// * Arguments:
    ///   * user name
    ///   * password
    /// * Responses:  no specific responses for this command
    /// * Result:
    ///   * OK - login completed, now in authenticated state
    ///   * NO - login failure: user name or password rejected
    ///   * BAD - command unknown or arguments invalid
    ///
    /// The LOGIN command identifies the client to the server and carries
    /// the plaintext password authenticating this user.
    ///
    /// A server MAY include a CAPABILITY response code in the tagged OK
    /// response to a successful LOGIN command in order to send
    /// capabilities automatically.  It is unnecessary for a client to
    /// send a separate CAPABILITY command if it recognizes these
    /// automatic capabilities.
    ///
    ///   Note: Use of the LOGIN command over an insecure network
    ///   (such as the Internet) is a security risk, because anyone
    ///   monitoring network traffic can obtain plaintext passwords.
    ///   The LOGIN command SHOULD NOT be used except as a last
    ///   resort, and it is recommended that client implementations
    ///   have a means to disable any automatic use of the LOGIN
    ///   command.
    ///
    ///   Unless either the STARTTLS command has been negotiated or
    ///   some other mechanism that protects the session from
    ///   password snooping has been provided, a server
    ///   implementation MUST implement a configuration in which it
    ///   advertises the LOGINDISABLED capability and does NOT permit
    ///   the LOGIN command.  Server sites SHOULD NOT use any
    ///   configuration which permits the LOGIN command without such
    ///   a protection mechanism against password snooping.  A client
    ///   implementation MUST NOT send a LOGIN command if the
    ///   LOGINDISABLED capability is advertised.
    Login {
        /// Username.
        username: AString<'a>,
        /// Password.
        password: Secret<AString<'a>>,
    },

    // ----- Authenticated State (https://tools.ietf.org/html/rfc3501#section-6.3) -----
    /// ### 6.3.1.  SELECT Command
    ///
    /// * Arguments:  mailbox name
    /// * Responses:
    ///   * REQUIRED untagged responses: FLAGS, EXISTS, RECENT
    ///   * REQUIRED OK untagged responses: UNSEEN, PERMANENTFLAGS, UIDNEXT, UIDVALIDITY
    /// * Result:
    ///   * OK - select completed, now in selected state
    ///   * NO - select failure, now in authenticated state: no such mailbox, can't access mailbox
    ///   * BAD - command unknown or arguments invalid
    ///
    /// The SELECT command selects a mailbox so that messages in the
    /// mailbox can be accessed.  Before returning an OK to the client,
    /// the server MUST send the following untagged data to the client.
    /// Note that earlier versions of this protocol only required the
    /// FLAGS, EXISTS, and RECENT untagged data; consequently, client
    /// implementations SHOULD implement default behavior for missing data
    /// as discussed with the individual item.
    ///
    ///   FLAGS       Defined flags in the mailbox.  See the description
    ///               of the FLAGS response for more detail.
    ///
    ///   \<n\> EXISTS  The number of messages in the mailbox.  See the
    ///               description of the EXISTS response for more detail.
    ///
    ///   \<n\> RECENT  The number of messages with the \Recent flag set.
    ///               See the description of the RECENT response for more
    ///               detail.
    ///
    ///   OK [UNSEEN \<n\>]
    ///               The message sequence number of the first unseen
    ///               message in the mailbox.  If this is missing, the
    ///               client can not make any assumptions about the first
    ///               unseen message in the mailbox, and needs to issue a
    ///               SEARCH command if it wants to find it.
    ///
    ///   OK [PERMANENTFLAGS (\<list of flags\>)]
    ///               A list of message flags that the client can change
    ///               permanently.  If this is missing, the client should
    ///               assume that all flags can be changed permanently.
    ///
    ///   OK [UIDNEXT \<n\>]
    ///               The next unique identifier value.  Refer to section
    ///               2.3.1.1 for more information.  If this is missing,
    ///               the client can not make any assumptions about the
    ///               next unique identifier value.
    ///
    ///   OK [UIDVALIDITY \<n\>]
    ///               The unique identifier validity value.  Refer to
    ///               section 2.3.1.1 for more information.  If this is
    ///               missing, the server does not support unique
    ///               identifiers.
    ///
    /// Only one mailbox can be selected at a time in a connection;
    /// simultaneous access to multiple mailboxes requires multiple
    /// connections.  The SELECT command automatically deselects any
    /// currently selected mailbox before attempting the new selection.
    /// Consequently, if a mailbox is selected and a SELECT command that
    /// fails is attempted, no mailbox is selected.
    ///
    /// If the client is permitted to modify the mailbox, the server
    /// SHOULD prefix the text of the tagged OK response with the
    /// "[READ-WRITE]" response code.
    ///
    /// If the client is not permitted to modify the mailbox but is
    /// permitted read access, the mailbox is selected as read-only, and
    /// the server MUST prefix the text of the tagged OK response to
    /// SELECT with the "[READ-ONLY]" response code.  Read-only access
    /// through SELECT differs from the EXAMINE command in that certain
    /// read-only mailboxes MAY permit the change of permanent state on a
    /// per-user (as opposed to global) basis.  Netnews messages marked in
    /// a server-based .newsrc file are an example of such per-user
    /// permanent state that can be modified with read-only mailboxes.
    Select {
        /// Mailbox.
        mailbox: Mailbox<'a>,
    },

    /// Unselect a mailbox.
    ///
    /// This should bring the client back to the AUTHENTICATED state.
    ///
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the UNSELECT capability.
    /// </div>
    Unselect,

    /// 6.3.2.  EXAMINE Command
    ///
    /// Arguments:  mailbox name
    /// Responses:  REQUIRED untagged responses: FLAGS, EXISTS, RECENT
    ///             REQUIRED OK untagged responses:  UNSEEN,  PERMANENTFLAGS,
    ///             UIDNEXT, UIDVALIDITY
    /// Result:     OK - examine completed, now in selected state
    ///             NO - examine failure, now in authenticated state: no
    ///                  such mailbox, can't access mailbox
    ///             BAD - command unknown or arguments invalid
    ///
    /// The EXAMINE command is identical to SELECT and returns the same
    /// output; however, the selected mailbox is identified as read-only.
    /// No changes to the permanent state of the mailbox, including
    /// per-user state, are permitted; in particular, EXAMINE MUST NOT
    /// cause messages to lose the \Recent flag.
    ///
    /// The text of the tagged OK response to the EXAMINE command MUST
    /// begin with the "[READ-ONLY]" response code.
    Examine {
        /// Mailbox.
        mailbox: Mailbox<'a>,
    },

    /// ### 6.3.3.  CREATE Command
    ///
    /// * Arguments:  mailbox name
    /// * Responses:  no specific responses for this command
    /// * Result:
    ///   * OK - create completed
    ///   * NO - create failure: can't create mailbox with that name
    ///   * BAD - command unknown or arguments invalid
    ///
    /// The CREATE command creates a mailbox with the given name.  An OK
    /// response is returned only if a new mailbox with that name has been
    /// created.  It is an error to attempt to create INBOX or a mailbox
    /// with a name that refers to an extant mailbox.  Any error in
    /// creation will return a tagged NO response.
    ///
    /// If the mailbox name is suffixed with the server's hierarchy
    /// separator character (as returned from the server by a LIST
    /// command), this is a declaration that the client intends to create
    /// mailbox names under this name in the hierarchy.  Server
    /// implementations that do not require this declaration MUST ignore
    /// the declaration.  In any case, the name created is without the
    /// trailing hierarchy delimiter.
    ///
    /// If the server's hierarchy separator character appears elsewhere in
    /// the name, the server SHOULD create any superior hierarchical names
    /// that are needed for the CREATE command to be successfully
    /// completed.  In other words, an attempt to create "foo/bar/zap" on
    /// a server in which "/" is the hierarchy separator character SHOULD
    /// create foo/ and foo/bar/ if they do not already exist.
    ///
    /// If a new mailbox is created with the same name as a mailbox which
    /// was deleted, its unique identifiers MUST be greater than any
    /// unique identifiers used in the previous incarnation of the mailbox
    /// UNLESS the new incarnation has a different unique identifier
    /// validity value.  See the description of the UID command for more
    /// detail.
    ///
    ///   Note: The interpretation of this example depends on whether
    ///   "/" was returned as the hierarchy separator from LIST.  If
    ///   "/" is the hierarchy separator, a new level of hierarchy
    ///   named "owatagusiam" with a member called "blurdybloop" is
    ///   created.  Otherwise, two mailboxes at the same hierarchy
    ///   level are created.
    Create {
        /// Mailbox.
        mailbox: Mailbox<'a>,
    },

    /// 6.3.4.  DELETE Command
    ///
    /// Arguments:  mailbox name
    /// Responses:  no specific responses for this command
    /// Result:     OK - delete completed
    ///             NO - delete failure: can't delete mailbox with that name
    ///             BAD - command unknown or arguments invalid
    ///
    /// The DELETE command permanently removes the mailbox with the given
    /// name.  A tagged OK response is returned only if the mailbox has
    /// been deleted.  It is an error to attempt to delete INBOX or a
    /// mailbox name that does not exist.
    ///
    /// The DELETE command MUST NOT remove inferior hierarchical names.
    /// For example, if a mailbox "foo" has an inferior "foo.bar"
    /// (assuming "." is the hierarchy delimiter character), removing
    /// "foo" MUST NOT remove "foo.bar".  It is an error to attempt to
    /// delete a name that has inferior hierarchical names and also has
    /// the \Noselect mailbox name attribute (see the description of the
    /// LIST response for more details).
    ///
    /// It is permitted to delete a name that has inferior hierarchical
    /// names and does not have the \Noselect mailbox name attribute.  In
    /// this case, all messages in that mailbox are removed, and the name
    /// will acquire the \Noselect mailbox name attribute.
    ///
    /// The value of the highest-used unique identifier of the deleted
    /// mailbox MUST be preserved so that a new mailbox created with the
    /// same name will not reuse the identifiers of the former
    /// incarnation, UNLESS the new incarnation has a different unique
    /// identifier validity value.  See the description of the UID command
    /// for more detail.
    Delete {
        /// Mailbox.
        mailbox: Mailbox<'a>,
    },

    /// 6.3.5.  RENAME Command
    ///
    /// Arguments:  existing mailbox name
    ///             new mailbox name
    /// Responses:  no specific responses for this command
    /// Result:     OK - rename completed
    ///             NO - rename failure: can't rename mailbox with that name,
    ///                  can't rename to mailbox with that name
    ///             BAD - command unknown or arguments invalid
    ///
    /// The RENAME command changes the name of a mailbox.  A tagged OK
    /// response is returned only if the mailbox has been renamed.  It is
    /// an error to attempt to rename from a mailbox name that does not
    /// exist or to a mailbox name that already exists.  Any error in
    /// renaming will return a tagged NO response.
    ///
    /// If the name has inferior hierarchical names, then the inferior
    /// hierarchical names MUST also be renamed.  For example, a rename of
    /// "foo" to "zap" will rename "foo/bar" (assuming "/" is the
    /// hierarchy delimiter character) to "zap/bar".
    ///
    /// If the server's hierarchy separator character appears in the name,
    /// the server SHOULD create any superior hierarchical names that are
    /// needed for the RENAME command to complete successfully.  In other
    /// words, an attempt to rename "foo/bar/zap" to baz/rag/zowie on a
    /// server in which "/" is the hierarchy separator character SHOULD
    /// create baz/ and baz/rag/ if they do not already exist.
    ///
    /// The value of the highest-used unique identifier of the old mailbox
    /// name MUST be preserved so that a new mailbox created with the same
    /// name will not reuse the identifiers of the former incarnation,
    /// UNLESS the new incarnation has a different unique identifier
    /// validity value.  See the description of the UID command for more
    /// detail.
    ///
    /// Renaming INBOX is permitted, and has special behavior.  It moves
    /// all messages in INBOX to a new mailbox with the given name,
    /// leaving INBOX empty.  If the server implementation supports
    /// inferior hierarchical names of INBOX, these are unaffected by a
    /// rename of INBOX.
    Rename {
        /// Current name.
        from: Mailbox<'a>,
        /// New name.
        to: Mailbox<'a>,
    },

    /// ### 6.3.6.  SUBSCRIBE Command
    ///
    /// * Arguments:  mailbox
    /// * Responses:  no specific responses for this command
    /// * Result:
    ///   * OK - subscribe completed
    ///   * NO - subscribe failure: can't subscribe to that name
    ///   * BAD - command unknown or arguments invalid
    ///
    /// The SUBSCRIBE command adds the specified mailbox name to the
    /// server's set of "active" or "subscribed" mailboxes as returned by
    /// the LSUB command.  This command returns a tagged OK response only
    /// if the subscription is successful.
    ///
    /// A server MAY validate the mailbox argument to SUBSCRIBE to verify
    /// that it exists.  However, it MUST NOT unilaterally remove an
    /// existing mailbox name from the subscription list even if a mailbox
    /// by that name no longer exists.
    ///
    ///   Note: This requirement is because a server site can
    ///   choose to routinely remove a mailbox with a well-known
    ///   name (e.g., "system-alerts") after its contents expire,
    ///   with the intention of recreating it when new contents
    ///   are appropriate.
    Subscribe {
        /// Mailbox.
        mailbox: Mailbox<'a>,
    },

    /// 6.3.7.  UNSUBSCRIBE Command
    ///
    /// Arguments:  mailbox name
    /// Responses:  no specific responses for this command
    /// Result:     OK - unsubscribe completed
    ///             NO - unsubscribe failure: can't unsubscribe that name
    ///             BAD - command unknown or arguments invalid
    ///
    /// The UNSUBSCRIBE command removes the specified mailbox name from
    /// the server's set of "active" or "subscribed" mailboxes as returned
    /// by the LSUB command.  This command returns a tagged OK response
    /// only if the unsubscription is successful.
    Unsubscribe {
        /// Mailbox.
        mailbox: Mailbox<'a>,
    },

    /// ### 6.3.8.  LIST Command
    ///
    /// * Arguments:
    ///   * reference name
    ///   * mailbox name with possible wildcards
    /// * Responses:  untagged responses: LIST
    /// * Result:
    ///   * OK - list completed
    ///   * NO - list failure: can't list that reference or name
    ///   * BAD - command unknown or arguments invalid
    ///
    /// The LIST command returns a subset of names from the complete set
    /// of all names available to the client.  Zero or more untagged LIST
    /// replies are returned, containing the name attributes, hierarchy
    /// delimiter, and name; see the description of the LIST reply for
    /// more detail.
    ///
    /// The LIST command SHOULD return its data quickly, without undue
    /// delay.  For example, it SHOULD NOT go to excess trouble to
    /// calculate the \Marked or \Unmarked status or perform other
    /// processing; if each name requires 1 second of processing, then a
    /// list of 1200 names would take 20 minutes!
    ///
    /// An empty ("" string) reference name argument indicates that the
    /// mailbox name is interpreted as by SELECT.  The returned mailbox
    /// names MUST match the supplied mailbox name pattern.  A non-empty
    /// reference name argument is the name of a mailbox or a level of
    /// mailbox hierarchy, and indicates the context in which the mailbox
    /// name is interpreted.
    ///
    /// An empty ("" string) mailbox name argument is a special request to
    /// return the hierarchy delimiter and the root name of the name given
    /// in the reference.  The value returned as the root MAY be the empty
    /// string if the reference is non-rooted or is an empty string.  In
    /// all cases, a hierarchy delimiter (or NIL if there is no hierarchy)
    /// is returned.  This permits a client to get the hierarchy delimiter
    /// (or find out that the mailbox names are flat) even when no
    /// mailboxes by that name currently exist.
    ///
    /// The reference and mailbox name arguments are interpreted into a
    /// canonical form that represents an unambiguous left-to-right
    /// hierarchy.  The returned mailbox names will be in the interpreted
    /// form.
    ///
    ///   Note: The interpretation of the reference argument is
    ///   implementation-defined.  It depends upon whether the
    ///   server implementation has a concept of the "current
    ///   working directory" and leading "break out characters",
    ///   which override the current working directory.
    ///
    ///   For example, on a server which exports a UNIX or NT
    ///   filesystem, the reference argument contains the current
    ///   working directory, and the mailbox name argument would
    ///   contain the name as interpreted in the current working
    ///   directory.
    ///
    /// If a server implementation has no concept of break out
    /// characters, the canonical form is normally the reference
    /// name appended with the mailbox name.  Note that if the
    /// server implements the namespace convention (section
    /// 5.1.2), "#" is a break out character and must be treated
    /// as such.
    ///
    /// If the reference argument is not a level of mailbox
    /// hierarchy (that is, it is a \NoInferiors name), and/or
    /// the reference argument does not end with the hierarchy
    /// delimiter, it is implementation-dependent how this is
    /// interpreted.  For example, a reference of "foo/bar" and
    /// mailbox name of "rag/baz" could be interpreted as
    /// "foo/bar/rag/baz", "foo/barrag/baz", or "foo/rag/baz".
    /// A client SHOULD NOT use such a reference argument except
    /// at the explicit request of the user.  A hierarchical
    /// browser MUST NOT make any assumptions about server
    /// interpretation of the reference unless the reference is
    /// a level of mailbox hierarchy AND ends with the hierarchy
    /// delimiter.
    ///
    /// Any part of the reference argument that is included in the
    /// interpreted form SHOULD prefix the interpreted form.  It SHOULD
    /// also be in the same form as the reference name argument.  This
    /// rule permits the client to determine if the returned mailbox name
    /// is in the context of the reference argument, or if something about
    /// the mailbox argument overrode the reference argument.  Without
    /// this rule, the client would have to have knowledge of the server's
    /// naming semantics including what characters are "breakouts" that
    /// override a naming context.
    ///
    ///   For example, here are some examples of how references
    ///   and mailbox names might be interpreted on a UNIX-based
    ///   server:
    ///
    /// ```text
    /// Reference     Mailbox Name  Interpretation
    /// ------------  ------------  --------------
    /// ~smith/Mail/  foo.*         ~smith/Mail/foo.*
    /// archive/      %             archive/%
    /// #news.        comp.mail.*   #news.comp.mail.*
    /// ~smith/Mail/  /usr/doc/foo  /usr/doc/foo
    /// archive/      ~fred/Mail/*  ~fred/Mail/*
    /// ```
    ///
    ///   The first three examples demonstrate interpretations in
    ///   the context of the reference argument.  Note that
    ///   "~smith/Mail" SHOULD NOT be transformed into something
    ///   like "/u2/users/smith/Mail", or it would be impossible
    ///   for the client to determine that the interpretation was
    ///   in the context of the reference.
    ///
    /// The character "*" is a wildcard, and matches zero or more
    /// characters at this position.  The character "%" is similar to "*",
    /// but it does not match a hierarchy delimiter.  If the "%" wildcard
    /// is the last character of a mailbox name argument, matching levels
    /// of hierarchy are also returned.  If these levels of hierarchy are
    /// not also selectable mailboxes, they are returned with the
    /// \Noselect mailbox name attribute (see the description of the LIST
    /// response for more details).
    ///
    /// Server implementations are permitted to "hide" otherwise
    /// accessible mailboxes from the wildcard characters, by preventing
    /// certain characters or names from matching a wildcard in certain
    /// situations.  For example, a UNIX-based server might restrict the
    /// interpretation of "*" so that an initial "/" character does not
    /// match.
    ///
    /// The special name INBOX is included in the output from LIST, if
    /// INBOX is supported by this server for this user and if the
    /// uppercase string "INBOX" matches the interpreted reference and
    /// mailbox name arguments with wildcards as described above.  The
    /// criteria for omitting INBOX is whether SELECT INBOX will return
    /// failure; it is not relevant whether the user's real INBOX resides
    /// on this or some other server.
    List {
        /// Reference.
        reference: Mailbox<'a>,
        /// Mailbox (wildcard).
        mailbox_wildcard: ListMailbox<'a>,
    },

    /// ### 6.3.9.  LSUB Command
    ///
    /// * Arguments:
    ///   * reference name
    ///   * mailbox name with possible wildcards
    /// * Responses:  untagged responses: LSUB
    /// * Result:
    ///   * OK - lsub completed
    ///   * NO - lsub failure: can't list that reference or name
    ///   * BAD - command unknown or arguments invalid
    ///
    /// The LSUB command returns a subset of names from the set of names
    /// that the user has declared as being "active" or "subscribed".
    /// Zero or more untagged LSUB replies are returned.  The arguments to
    /// LSUB are in the same form as those for LIST.
    ///
    /// The returned untagged LSUB response MAY contain different mailbox
    /// flags from a LIST untagged response.  If this should happen, the
    /// flags in the untagged LIST are considered more authoritative.
    ///
    /// A special situation occurs when using LSUB with the % wildcard.
    /// Consider what happens if "foo/bar" (with a hierarchy delimiter of
    /// "/") is subscribed but "foo" is not.  A "%" wildcard to LSUB must
    /// return foo, not foo/bar, in the LSUB response, and it MUST be
    /// flagged with the \Noselect attribute.
    ///
    /// The server MUST NOT unilaterally remove an existing mailbox name
    /// from the subscription list even if a mailbox by that name no
    /// longer exists.
    Lsub {
        /// Reference.
        reference: Mailbox<'a>,
        /// Mailbox (wildcard).
        mailbox_wildcard: ListMailbox<'a>,
    },

    /// ### 6.3.10. STATUS Command
    ///
    /// * Arguments:
    ///   * mailbox name
    ///   * status data item names
    /// * Responses:  untagged responses: STATUS
    /// * Result:
    ///   * OK - status completed
    ///   * NO - status failure: no status for that name
    ///   * BAD - command unknown or arguments invalid
    ///
    /// The STATUS command requests the status of the indicated mailbox.
    /// It does not change the currently selected mailbox, nor does it
    /// affect the state of any messages in the queried mailbox (in
    /// particular, STATUS MUST NOT cause messages to lose the \Recent
    /// flag).
    ///
    /// The STATUS command provides an alternative to opening a second
    /// IMAP4rev1 connection and doing an EXAMINE command on a mailbox to
    /// query that mailbox's status without deselecting the current
    /// mailbox in the first IMAP4rev1 connection.
    ///
    /// Unlike the LIST command, the STATUS command is not guaranteed to
    /// be fast in its response.  Under certain circumstances, it can be
    /// quite slow.  In some implementations, the server is obliged to
    /// open the mailbox read-only internally to obtain certain status
    /// information.  Also unlike the LIST command, the STATUS command
    /// does not accept wildcards.
    ///
    ///   Note: The STATUS command is intended to access the
    ///   status of mailboxes other than the currently selected
    ///   mailbox.  Because the STATUS command can cause the
    ///   mailbox to be opened internally, and because this
    ///   information is available by other means on the selected
    ///   mailbox, the STATUS command SHOULD NOT be used on the
    ///   currently selected mailbox.
    ///
    ///   The STATUS command MUST NOT be used as a "check for new
    ///   messages in the selected mailbox" operation (refer to
    ///   sections 7, 7.3.1, and 7.3.2 for more information about
    ///   the proper method for new message checking).
    ///
    ///   Because the STATUS command is not guaranteed to be fast
    ///   in its results, clients SHOULD NOT expect to be able to
    ///   issue many consecutive STATUS commands and obtain
    ///   reasonable performance.
    Status {
        /// Mailbox.
        mailbox: Mailbox<'a>,
        /// Status data items.
        item_names: Cow<'a, [StatusDataItemName]>,
    },

    /// 6.3.11. APPEND Command
    ///
    /// Arguments:  mailbox name
    ///             OPTIONAL flag parenthesized list
    ///             OPTIONAL date/time string
    ///             message literal
    /// Responses:  no specific responses for this command
    /// Result:     OK - append completed
    ///             NO - append error: can't append to that mailbox, error
    ///                  in flags or date/time or message text
    ///             BAD - command unknown or arguments invalid
    ///
    /// The APPEND command appends the literal argument as a new message
    /// to the end of the specified destination mailbox.  This argument
    /// SHOULD be in the format of an [RFC-2822] message.  8-bit
    /// characters are permitted in the message.  A server implementation
    /// that is unable to preserve 8-bit data properly MUST be able to
    /// reversibly convert 8-bit APPEND data to 7-bit using a [MIME-IMB]
    /// content transfer encoding.
    ///
    ///   Note: There MAY be exceptions, e.g., draft messages, in
    ///   which required [RFC-2822] header lines are omitted in
    ///   the message literal argument to APPEND.  The full
    ///   implications of doing so MUST be understood and
    ///   carefully weighed.
    ///
    /// If a flag parenthesized list is specified, the flags SHOULD be set
    /// in the resulting message; otherwise, the flag list of the
    /// resulting message is set to empty by default.  In either case, the
    /// Recent flag is also set.
    ///
    /// If a date-time is specified, the internal date SHOULD be set in
    /// the resulting message; otherwise, the internal date of the
    /// resulting message is set to the current date and time by default.
    ///
    /// If the append is unsuccessful for any reason, the mailbox MUST be
    /// restored to its state before the APPEND attempt; no partial
    /// appending is permitted.
    ///
    /// If the destination mailbox does not exist, a server MUST return an
    /// error, and MUST NOT automatically create the mailbox.  Unless it
    /// is certain that the destination mailbox can not be created, the
    /// server MUST send the response code "\[TRYCREATE\]" as the prefix of
    /// the text of the tagged NO response.  This gives a hint to the
    /// client that it can attempt a CREATE command and retry the APPEND
    /// if the CREATE is successful.
    ///
    /// If the mailbox is currently selected, the normal new message
    /// actions SHOULD occur.  Specifically, the server SHOULD notify the
    /// client immediately via an untagged EXISTS response.  If the server
    /// does not do so, the client MAY issue a NOOP command (or failing
    /// that, a CHECK command) after one or more APPEND commands.
    ///
    ///   Note: The APPEND command is not used for message delivery,
    ///   because it does not provide a mechanism to transfer \[SMTP\]
    ///   envelope information.
    Append {
        /// Mailbox.
        mailbox: Mailbox<'a>,
        /// Flags.
        flags: Vec<Flag<'a>>,
        /// Datetime.
        date: Option<DateTime>,
        /// Message to append.
        ///
        /// <div class="warning">
        /// Use [`LiteralOrLiteral8::Literal8`] only when the server advertised [`Capability::Binary`](crate::response::Capability::Binary).
        /// </div>
        message: LiteralOrLiteral8<'a>,
    },

    // ----- Selected State (https://tools.ietf.org/html/rfc3501#section-6.4) -----
    /// ### 6.4.1.  CHECK Command
    ///
    /// * Arguments:  none
    /// * Responses:  no specific responses for this command
    /// * Result:
    ///   * OK - check completed
    ///   * BAD - command unknown or arguments invalid
    ///
    /// The CHECK command requests a checkpoint of the currently selected
    /// mailbox.  A checkpoint refers to any implementation-dependent
    /// housekeeping associated with the mailbox (e.g., resolving the
    /// server's in-memory state of the mailbox with the state on its
    /// disk) that is not normally executed as part of each command.  A
    /// checkpoint MAY take a non-instantaneous amount of real time to
    /// complete.  If a server implementation has no such housekeeping
    /// considerations, CHECK is equivalent to NOOP.
    ///
    /// There is no guarantee that an EXISTS untagged response will happen
    /// as a result of CHECK.  NOOP, not CHECK, SHOULD be used for new
    /// message polling.
    Check,

    /// ### 6.4.2.  CLOSE Command
    ///
    ///    * Arguments:  none
    ///    * Responses:  no specific responses for this command
    ///    * Result:
    ///      * OK - close completed, now in authenticated state
    ///      * BAD - command unknown or arguments invalid
    ///
    ///       The CLOSE command permanently removes all messages that have the
    ///       \Deleted flag set from the currently selected mailbox, and returns
    ///       to the authenticated state from the selected state.  No untagged
    ///       EXPUNGE responses are sent.
    ///
    ///       No messages are removed, and no error is given, if the mailbox is
    ///       selected by an EXAMINE command or is otherwise selected read-only.
    ///
    ///       Even if a mailbox is selected, a SELECT, EXAMINE, or LOGOUT
    ///       command MAY be issued without previously issuing a CLOSE command.
    ///       The SELECT, EXAMINE, and LOGOUT commands implicitly close the
    ///       currently selected mailbox without doing an expunge.  However,
    ///       when many messages are deleted, a CLOSE-LOGOUT or CLOSE-SELECT
    ///       sequence is considerably faster than an EXPUNGE-LOGOUT or
    ///       EXPUNGE-SELECT because no untagged EXPUNGE responses (which the
    ///       client would probably ignore) are sent.
    Close,

    /// 6.4.3.  EXPUNGE Command
    ///
    /// Arguments: none
    /// Responses: untagged responses: EXPUNGE
    /// Result:    OK - expunge completed
    ///            NO - expunge failure: can't expunge (e.g., permission denied)
    ///            BAD - command unknown or arguments invalid
    ///
    /// The EXPUNGE command permanently removes all messages that have the
    /// \Deleted flag set from the currently selected mailbox.  Before
    /// returning an OK to the client, an untagged EXPUNGE response is
    /// sent for each message that is removed.
    Expunge,

    /// 2.1.  UID EXPUNGE Command (RFC 4315)
    ///
    /// Arguments: sequence set
    /// Data:      untagged responses: EXPUNGE
    /// Result:    OK - expunge completed
    ///            NO - expunge failure (e.g., permission denied)
    ///            BAD - command unknown or arguments invalid
    ///
    /// The UID EXPUNGE command permanently removes all messages that both
    /// have the \Deleted flag set and have a UID that is included in the
    /// specified sequence set from the currently selected mailbox.  If a
    /// message either does not have the \Deleted flag set or has a UID
    /// that is not included in the specified sequence set, it is not
    /// affected.
    ///
    /// This command is particularly useful for disconnected use clients.
    /// By using UID EXPUNGE instead of EXPUNGE when resynchronizing with
    /// the server, the client can ensure that it does not inadvertantly
    /// remove any messages that have been marked as \Deleted by other
    /// clients between the time that the client was last connected and
    /// the time the client resynchronizes.
    ///
    /// If the server does not support the UIDPLUS capability, the client
    /// should fall back to using the STORE command to temporarily remove
    /// the \Deleted flag from messages it does not want to remove, then
    /// issuing the EXPUNGE command.  Finally, the client should use the
    /// STORE command to restore the \Deleted flag on the messages in
    /// which it was temporarily removed.
    ///
    /// Alternatively, the client may fall back to using just the EXPUNGE
    /// command, risking the unintended removal of some messages.
    ExpungeUid { sequence_set: SequenceSet },

    /// ### 6.4.4.  SEARCH Command
    ///
    /// * Arguments:
    ///   * OPTIONAL \[CHARSET\] specification
    ///   * searching criteria (one or more)
    /// * Responses:  REQUIRED untagged response: SEARCH
    /// * Result:
    ///   * OK - search completed
    ///   * NO - search error: can't search that \[CHARSET\] or criteria
    ///   * BAD - command unknown or arguments invalid
    ///
    /// The SEARCH command searches the mailbox for messages that match
    /// the given searching criteria.  Searching criteria consist of one
    /// or more search keys.  The untagged SEARCH response from the server
    /// contains a listing of message sequence numbers corresponding to
    /// those messages that match the searching criteria.
    ///
    /// When multiple keys are specified, the result is the intersection
    /// (AND function) of all the messages that match those keys.  For
    /// example, the criteria DELETED FROM "SMITH" SINCE 1-Feb-1994 refers
    /// to all deleted messages from Smith that were placed in the mailbox
    /// since February 1, 1994.  A search key can also be a parenthesized
    /// list of one or more search keys (e.g., for use with the OR and NOT
    /// keys).
    ///
    /// Server implementations MAY exclude [MIME-IMB] body parts with
    /// terminal content media types other than TEXT and MESSAGE from
    /// consideration in SEARCH matching.
    ///
    /// The OPTIONAL \[CHARSET\] specification consists of the word
    /// "CHARSET" followed by a registered \[CHARSET\].  It indicates the
    /// \[CHARSET\] of the strings that appear in the search criteria.
    /// [MIME-IMB] content transfer encodings, and [MIME-HDRS] strings in
    /// [RFC-2822]/[MIME-IMB] headers, MUST be decoded before comparing
    /// text in a \[CHARSET\] other than US-ASCII.  US-ASCII MUST be
    /// supported; other \[CHARSET\]s MAY be supported.
    ///
    /// If the server does not support the specified \[CHARSET\], it MUST
    /// return a tagged NO response (not a BAD).  This response SHOULD
    /// contain the BADCHARSET response code, which MAY list the
    /// \[CHARSET\]s supported by the server.
    ///
    /// In all search keys that use strings, a message matches the key if
    /// the string is a substring of the field.  The matching is
    /// case-insensitive.
    ///
    /// See [SearchKey] enum.
    ///
    /// Note: Since this document is restricted to 7-bit ASCII
    /// text, it is not possible to show actual UTF-8 data.  The
    /// "XXXXXX" is a placeholder for what would be 6 octets of
    /// 8-bit data in an actual transaction.
    Search {
        /// Charset.
        charset: Option<Charset<'a>>,
        /// Criteria.
        criteria: Vec1<SearchKey<'a>>,
        /// Use UID variant.
        uid: bool,
    },

    /// SORT command.
    ///
    /// The SORT command is a variant of SEARCH with sorting semantics for the results.
    ///
    /// Data:
    /// * untagged responses: SORT
    ///
    /// Result:
    /// * OK - sort completed
    /// * NO - sort error: can't sort that charset or criteria
    /// * BAD - command unknown or arguments invalid
    ///
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the SORT capability.
    /// </div>
    Sort {
        /// Sort criteria.
        sort_criteria: Vec1<SortCriterion>,
        /// Charset.
        charset: Charset<'a>,
        /// Search criteria.
        search_criteria: Vec1<SearchKey<'a>>,
        /// Use UID variant.
        uid: bool,
    },

    /// THREAD command.
    ///
    /// The THREAD command is a variant of SEARCH with threading semantics for the results.
    ///
    /// Data:
    /// * untagged responses: THREAD
    ///
    /// Result:
    /// * OK - thread completed
    /// * NO - thread error: can't thread that charset or criteria
    /// * BAD - command unknown or arguments invalid
    ///
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the THREAD capability.
    /// </div>
    Thread {
        /// Threading algorithm.
        algorithm: ThreadingAlgorithm<'a>,
        /// Charset.
        charset: Charset<'a>,
        /// Search criteria.
        search_criteria: Vec1<SearchKey<'a>>,
        /// Use UID variant.
        uid: bool,
    },

    /// ### 6.4.5.  FETCH Command
    ///
    /// * Arguments:
    ///   * sequence set
    ///   * message data item names or macro
    /// * Responses:  untagged responses: FETCH
    /// * Result:
    ///   * OK - fetch completed
    ///   * NO - fetch error: can't fetch that data
    ///   * BAD - command unknown or arguments invalid
    ///
    /// The FETCH command retrieves data associated with a message in the
    /// mailbox.  The data items to be fetched can be either a single atom
    /// or a parenthesized list.
    ///
    /// Most data items, identified in the formal syntax under the
    /// msg-att-static rule, are static and MUST NOT change for any
    /// particular message.  Other data items, identified in the formal
    /// syntax under the msg-att-dynamic rule, MAY change, either as a
    /// result of a STORE command or due to external events.
    ///
    ///   For example, if a client receives an ENVELOPE for a
    ///   message when it already knows the envelope, it can
    ///   safely ignore the newly transmitted envelope.
    Fetch {
        /// Set of messages.
        sequence_set: SequenceSet,
        /// Message data items (or a macro).
        macro_or_item_names: MacroOrMessageDataItemNames<'a>,
        /// Use UID variant.
        uid: bool,
    },

    /// ### 6.4.6.  STORE Command
    ///
    /// * Arguments:
    ///   * sequence set
    ///   * message data item name
    ///   * value for message data item
    /// * Responses:  untagged responses: FETCH
    /// * Result:
    ///   * OK - store completed
    ///   * NO - store error: can't store that data
    ///   * BAD - command unknown or arguments invalid
    ///
    /// The STORE command alters data associated with a message in the
    /// mailbox.  Normally, STORE will return the updated value of the
    /// data with an untagged FETCH response.  A suffix of ".SILENT" in
    /// the data item name prevents the untagged FETCH, and the server
    /// SHOULD assume that the client has determined the updated value
    /// itself or does not care about the updated value.
    ///
    ///   Note: Regardless of whether or not the ".SILENT" suffix
    ///   was used, the server SHOULD send an untagged FETCH
    ///   response if a change to a message's flags from an
    ///   external source is observed.  The intent is that the
    ///   status of the flags is determinate without a race
    ///   condition.
    ///
    /// The currently defined data items that can be stored are:
    ///
    /// FLAGS \<flag list\>
    ///    Replace the flags for the message (other than \Recent) with the
    ///    argument.  The new value of the flags is returned as if a FETCH
    ///    of those flags was done.
    ///
    /// FLAGS.SILENT \<flag list\>
    ///    Equivalent to FLAGS, but without returning a new value.
    ///
    /// +FLAGS \<flag list\>
    ///    Add the argument to the flags for the message.  The new value
    ///    of the flags is returned as if a FETCH of those flags was done.
    ///
    /// +FLAGS.SILENT \<flag list\>
    ///    Equivalent to +FLAGS, but without returning a new value.
    ///
    /// -FLAGS \<flag list\>
    ///    Remove the argument from the flags for the message.  The new
    ///    value of the flags is returned as if a FETCH of those flags was
    ///    done.
    ///
    /// -FLAGS.SILENT \<flag list\>
    ///    Equivalent to -FLAGS, but without returning a new value.
    Store {
        /// Set of messages.
        sequence_set: SequenceSet,
        /// Kind of storage, i.e., replace, add, or remove.
        kind: StoreType,
        /// Kind of response, i.e., answer or silent.
        response: StoreResponse,
        /// Flags.
        flags: Vec<Flag<'a>>, // FIXME(misuse): must not accept "\*" or "\Recent"
        /// Use UID variant.
        uid: bool,
    },

    /// 6.4.7.  COPY Command
    ///
    /// Arguments:  sequence set
    ///             mailbox name
    /// Responses:  no specific responses for this command
    /// Result:     OK - copy completed
    ///             NO - copy error: can't copy those messages or to that
    ///                  name
    ///             BAD - command unknown or arguments invalid
    ///
    /// The COPY command copies the specified message(s) to the end of the
    /// specified destination mailbox.  The flags and internal date of the
    /// message(s) SHOULD be preserved, and the Recent flag SHOULD be set,
    /// in the copy.
    ///
    /// If the destination mailbox does not exist, a server SHOULD return
    /// an error.  It SHOULD NOT automatically create the mailbox.  Unless
    /// it is certain that the destination mailbox can not be created, the
    /// server MUST send the response code "\[TRYCREATE\]" as the prefix of
    /// the text of the tagged NO response.  This gives a hint to the
    /// client that it can attempt a CREATE command and retry the COPY if
    /// the CREATE is successful.
    ///
    /// If the COPY command is unsuccessful for any reason, server
    /// implementations MUST restore the destination mailbox to its state
    /// before the COPY attempt.
    Copy {
        /// Set of messages.
        sequence_set: SequenceSet,
        /// Destination mailbox.
        mailbox: Mailbox<'a>,
        /// Use UID variant.
        uid: bool,
    },

    /// The UID mechanism was inlined into copy, fetch, store, and search.
    /// as an additional parameter.
    ///
    /// ### 6.4.8.  UID Command
    ///
    /// * Arguments:
    ///   * command name
    ///   * command arguments
    /// * Responses:  untagged responses: FETCH, SEARCH
    /// * Result:
    ///   * OK - UID command completed
    ///   * NO - UID command error
    ///   * BAD - command unknown or arguments invalid
    ///
    /// The UID command has two forms.  In the first form, it takes as its
    /// arguments a COPY, FETCH, or STORE command with arguments
    /// appropriate for the associated command.  However, the numbers in
    /// the sequence set argument are unique identifiers instead of
    /// message sequence numbers.  Sequence set ranges are permitted, but
    /// there is no guarantee that unique identifiers will be contiguous.
    ///
    /// A non-existent unique identifier is ignored without any error
    /// message generated.  Thus, it is possible for a UID FETCH command
    /// to return an OK without any data or a UID COPY or UID STORE to
    /// return an OK without performing any operations.
    ///
    /// In the second form, the UID command takes a SEARCH command with
    /// SEARCH command arguments.  The interpretation of the arguments is
    /// the same as with SEARCH; however, the numbers returned in a SEARCH
    /// response for a UID SEARCH command are unique identifiers instead
    /// of message sequence numbers.  For example, the command UID SEARCH
    /// 1:100 UID 443:557 returns the unique identifiers corresponding to
    /// the intersection of two sequence sets, the message sequence number
    /// range 1:100 and the UID range 443:557.
    ///
    ///   Note: in the above example, the UID range 443:557
    ///   appears.  The same comment about a non-existent unique
    ///   identifier being ignored without any error message also
    ///   applies here.  Hence, even if neither UID 443 or 557
    ///   exist, this range is valid and would include an existing
    ///   UID 495.
    ///
    ///   Also note that a UID range of 559:* always includes the
    ///   UID of the last message in the mailbox, even if 559 is
    ///   higher than any assigned UID value.  This is because the
    ///   contents of a range are independent of the order of the
    ///   range endpoints.  Thus, any UID range with * as one of
    ///   the endpoints indicates at least one message (the
    ///   message with the highest numbered UID), unless the
    ///   mailbox is empty.
    ///
    ///   The number after the "*" in an untagged FETCH response is always a
    ///   message sequence number, not a unique identifier, even for a UID
    ///   command response.  However, server implementations MUST implicitly
    ///   include the UID message data item as part of any FETCH response
    ///   caused by a UID command, regardless of whether a UID was specified
    ///   as a message data item to the FETCH.
    ///
    ///   Note: The rule about including the UID message data item as part
    ///   of a FETCH response primarily applies to the UID FETCH and UID
    ///   STORE commands, including a UID FETCH command that does not
    ///   include UID as a message data item.  Although it is unlikely that
    ///   the other UID commands will cause an untagged FETCH, this rule
    ///   applies to these commands as well.

    // ----- Experimental/Expansion (https://tools.ietf.org/html/rfc3501#section-6.5) -----

    // ### 6.5.1.  X<atom> Command
    //
    // * Arguments:  implementation defined
    // * Responses:  implementation defined
    // * Result:
    //   * OK - command completed
    //   * NO - failure
    //   * BAD - command unknown or arguments invalid
    //
    // Any command prefixed with an X is an experimental command.
    // Commands which are not part of this specification, a standard or
    // standards-track revision of this specification, or an
    // IESG-approved experimental protocol, MUST use the X prefix.
    //
    // Any added untagged responses issued by an experimental command
    // MUST also be prefixed with an X.  Server implementations MUST NOT
    // send any such untagged responses, unless the client requested it
    // by issuing the associated experimental command.
    //X,
    /// IDLE command.
    ///
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the IDLE capability.
    /// </div>
    Idle,

    /// ENABLE command.
    ///
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the ENABLE capability.
    /// </div>
    Enable {
        /// Capabilities to enable.
        capabilities: Vec1<CapabilityEnable<'a>>,
    },

    /// COMPRESS command.
    ///
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the COMPRESS capability.
    /// </div>
    Compress {
        /// Compression algorithm.
        algorithm: CompressionAlgorithm,
    },

    /// Takes the name of a quota root and returns the quota root's resource usage and limits in an untagged QUOTA response.
    ///
    /// Arguments:
    /// * quota root
    ///
    /// Responses:
    /// * REQUIRED untagged responses: QUOTA
    ///
    /// Result:
    /// * OK - getquota completed
    /// * NO - getquota error: no such quota root, permission denied
    /// * BAD - command unknown or arguments invalid
    ///
    /// # Example (IMAP)
    ///
    /// ```imap
    /// S: * CAPABILITY [...] QUOTA QUOTA=RES-STORAGE [...]
    /// [...]
    /// C: G0001 GETQUOTA "!partition/sda4"
    /// S: * QUOTA "!partition/sda4" (STORAGE 104 10923847)
    /// S: G0001 OK Getquota complete
    /// ```
    ///
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the QUOTA* capability.
    /// </div>
    GetQuota {
        /// Name of quota root.
        root: AString<'a>,
    },

    /// Takes a mailbox name and returns the list of quota roots for the mailbox in an untagged QUOTAROOT response.
    /// For each listed quota root, it also returns the quota root's resource usage and limits in an untagged QUOTA response.
    ///
    /// Arguments:
    /// * mailbox name
    ///
    /// Responses:
    /// * REQUIRED untagged responses: QUOTAROOT, QUOTA
    ///
    /// Result:
    /// * OK - getquotaroot completed
    /// * NO - getquotaroot error: permission denied
    /// * BAD - command unknown or arguments invalid
    ///
    /// Note that the mailbox name parameter doesn't have to reference an existing mailbox.
    /// This can be handy in order to determine which quota root would apply to a mailbox when it gets created
    ///
    /// # Example (IMAP)
    ///
    /// ```imap
    /// S: * CAPABILITY [...] QUOTA QUOTA=RES-STORAGE QUOTA=RES-MESSAGE
    /// [...]
    /// C: G0002 GETQUOTAROOT INBOX
    /// S: * QUOTAROOT INBOX "#user/alice" "!partition/sda4"
    /// S: * QUOTA "#user/alice" (MESSAGE 42 1000)
    /// S: * QUOTA "!partition/sda4" (STORAGE 104 10923847)
    /// S: G0002 OK Getquotaroot complete
    /// ```
    ///
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the QUOTA* capability.
    /// </div>
    GetQuotaRoot {
        /// Name of mailbox.
        mailbox: Mailbox<'a>,
    },

    /// Changes the mailbox quota root resource limits to the specified limits.
    ///
    /// Arguments:
    /// * quota root list of resource limits
    ///
    /// Responses:
    /// * untagged responses: QUOTA
    ///
    /// Result:
    ///
    /// * OK - setquota completed
    /// * NO - setquota error: can't set that data
    /// * BAD - command unknown or arguments invalid
    ///
    /// Note: requires the server to advertise the "QUOTASET" capability.
    ///
    /// # Example (IMAP)
    ///
    /// ```imap
    /// S: * CAPABILITY [...] QUOTA QUOTASET QUOTA=RES-STORAGE QUOTA=RES-
    /// MESSAGE [...]
    /// [...]
    /// C: S0000 GETQUOTA "#user/alice"
    /// S: * QUOTA "#user/alice" (STORAGE 54 111 MESSAGE 42 1000)
    /// S: S0000 OK Getquota completed
    /// C: S0001 SETQUOTA "#user/alice" (STORAGE 510)
    /// S: * QUOTA "#user/alice" (STORAGE 58 512)
    /// // The server has rounded the STORAGE quota limit requested to
    /// the nearest 512 blocks of 1024 octets; otherwise, another client
    /// has performed a near-simultaneous SETQUOTA using a limit of 512.
    /// S: S0001 OK Rounded quota
    /// C: S0002 SETQUOTA "!partition/sda4" (STORAGE 99999999)
    /// S: * QUOTA "!partition/sda4" (STORAGE 104 10923847)
    /// // The server has not changed the quota, since this is a
    /// filesystem limit, and it cannot be changed. The QUOTA
    /// response here is entirely optional.
    /// S: S0002 NO Cannot change system limit
    /// ```
    ///
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the QUOTA* capability.
    /// </div>
    SetQuota {
        /// Name of quota root.
        root: AString<'a>,
        /// List of resource limits.
        quotas: Vec<QuotaSet<'a>>,
    },

    /// MOVE command.
    ///
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the MOVE capability.
    /// </div>
    Move {
        /// Set of messages.
        sequence_set: SequenceSet,
        /// Destination mailbox.
        mailbox: Mailbox<'a>,
        /// Use UID variant.
        uid: bool,
    },

    #[cfg(feature = "ext_id")]
    /// ID command.
    ///
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the ID capability.
    /// </div>
    Id {
        /// Parameters.
        parameters: Option<Vec<(IString<'a>, NString<'a>)>>,
    },

    #[cfg(feature = "ext_metadata")]
    /// Set annotation(s).
    ///
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the METADATA* capability.
    /// </div>
    SetMetadata {
        mailbox: Mailbox<'a>,
        entry_values: Vec1<EntryValue<'a>>,
    },

    #[cfg(feature = "ext_metadata")]
    /// Retrieve server or mailbox annotation(s).
    ///
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the METADATA* capability.
    /// </div>
    GetMetadata {
        options: Vec<GetMetadataOption>,
        mailbox: Mailbox<'a>,
        entries: Vec1<Entry<'a>>,
    },
}

impl<'a> CommandBody<'a> {
    /// Prepend a tag to finalize the command body to a command.
    pub fn tag<T>(self, tag: T) -> Result<Command<'a>, T::Error>
    where
        T: TryInto<Tag<'a>>,
    {
        Ok(Command {
            tag: tag.try_into()?,
            body: self,
        })
    }

    // ----- Constructors -----

    /// Construct an AUTHENTICATE command.
    pub fn authenticate(mechanism: AuthMechanism<'a>) -> Self {
        CommandBody::Authenticate {
            mechanism,
            initial_response: None,
        }
    }

    /// Construct an AUTHENTICATE command (with an initial response, SASL-IR).
    ///
    /// Note: Use this only when the server advertised the `SASL-IR` capability.
    ///
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the SASL-IR capability.
    /// </div>
    pub fn authenticate_with_ir<I>(mechanism: AuthMechanism<'a>, initial_response: I) -> Self
    where
        I: Into<Cow<'a, [u8]>>,
    {
        CommandBody::Authenticate {
            mechanism,
            initial_response: Some(Secret::new(initial_response.into())),
        }
    }

    /// Construct a LOGIN command.
    pub fn login<U, P>(username: U, password: P) -> Result<Self, LoginError<U::Error, P::Error>>
    where
        U: TryInto<AString<'a>>,
        P: TryInto<AString<'a>>,
    {
        Ok(CommandBody::Login {
            username: username.try_into().map_err(LoginError::Username)?,
            password: Secret::new(password.try_into().map_err(LoginError::Password)?),
        })
    }

    /// Construct a SELECT command.
    pub fn select<M>(mailbox: M) -> Result<Self, M::Error>
    where
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::Select {
            mailbox: mailbox.try_into()?,
        })
    }

    /// Construct an EXAMINE command.
    pub fn examine<M>(mailbox: M) -> Result<Self, M::Error>
    where
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::Examine {
            mailbox: mailbox.try_into()?,
        })
    }

    /// Construct a CREATE command.
    pub fn create<M>(mailbox: M) -> Result<Self, M::Error>
    where
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::Create {
            mailbox: mailbox.try_into()?,
        })
    }

    /// Construct a DELETE command.
    pub fn delete<M>(mailbox: M) -> Result<Self, M::Error>
    where
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::Delete {
            mailbox: mailbox.try_into()?,
        })
    }

    /// Construct a RENAME command.
    pub fn rename<F, T>(mailbox: F, new_mailbox: T) -> Result<Self, RenameError<F::Error, T::Error>>
    where
        F: TryInto<Mailbox<'a>>,
        T: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::Rename {
            from: mailbox.try_into().map_err(RenameError::From)?,
            to: new_mailbox.try_into().map_err(RenameError::To)?,
        })
    }

    /// Construct a SUBSCRIBE command.
    pub fn subscribe<M>(mailbox: M) -> Result<Self, M::Error>
    where
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::Subscribe {
            mailbox: mailbox.try_into()?,
        })
    }

    /// Construct an UNSUBSCRIBE command.
    pub fn unsubscribe<M>(mailbox: M) -> Result<Self, M::Error>
    where
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::Unsubscribe {
            mailbox: mailbox.try_into()?,
        })
    }

    /// Construct a LIST command.
    pub fn list<A, B>(
        reference: A,
        mailbox_wildcard: B,
    ) -> Result<Self, ListError<A::Error, B::Error>>
    where
        A: TryInto<Mailbox<'a>>,
        B: TryInto<ListMailbox<'a>>,
    {
        Ok(CommandBody::List {
            reference: reference.try_into().map_err(ListError::Reference)?,
            mailbox_wildcard: mailbox_wildcard.try_into().map_err(ListError::Mailbox)?,
        })
    }

    /// Construct a LSUB command.
    pub fn lsub<A, B>(
        reference: A,
        mailbox_wildcard: B,
    ) -> Result<Self, ListError<A::Error, B::Error>>
    where
        A: TryInto<Mailbox<'a>>,
        B: TryInto<ListMailbox<'a>>,
    {
        Ok(CommandBody::Lsub {
            reference: reference.try_into().map_err(ListError::Reference)?,
            mailbox_wildcard: mailbox_wildcard.try_into().map_err(ListError::Mailbox)?,
        })
    }

    /// Construct a STATUS command.
    pub fn status<M, I>(mailbox: M, item_names: I) -> Result<Self, M::Error>
    where
        M: TryInto<Mailbox<'a>>,
        I: Into<Cow<'a, [StatusDataItemName]>>,
    {
        let mailbox = mailbox.try_into()?;

        Ok(CommandBody::Status {
            mailbox,
            item_names: item_names.into(),
        })
    }

    /// Construct an APPEND command.
    pub fn append<M, D>(
        mailbox: M,
        flags: Vec<Flag<'a>>,
        date: Option<DateTime>,
        message: D,
    ) -> Result<Self, AppendError<M::Error, D::Error>>
    where
        M: TryInto<Mailbox<'a>>,
        D: TryInto<Literal<'a>>,
    {
        Ok(CommandBody::Append {
            mailbox: mailbox.try_into().map_err(AppendError::Mailbox)?,
            flags,
            date,
            message: LiteralOrLiteral8::Literal(message.try_into().map_err(AppendError::Data)?),
        })
    }

    /// Construct a SEARCH command.
    pub fn search(charset: Option<Charset<'a>>, criteria: Vec1<SearchKey<'a>>, uid: bool) -> Self {
        CommandBody::Search {
            charset,
            criteria,
            uid,
        }
    }

    /// Construct a FETCH command.
    pub fn fetch<S, I>(sequence_set: S, macro_or_item_names: I, uid: bool) -> Result<Self, S::Error>
    where
        S: TryInto<SequenceSet>,
        I: Into<MacroOrMessageDataItemNames<'a>>,
    {
        let sequence_set = sequence_set.try_into()?;

        Ok(CommandBody::Fetch {
            sequence_set,
            macro_or_item_names: macro_or_item_names.into(),
            uid,
        })
    }

    /// Construct a STORE command.
    pub fn store<S>(
        sequence_set: S,
        kind: StoreType,
        response: StoreResponse,
        flags: Vec<Flag<'a>>,
        uid: bool,
    ) -> Result<Self, S::Error>
    where
        S: TryInto<SequenceSet>,
    {
        let sequence_set = sequence_set.try_into()?;

        Ok(CommandBody::Store {
            sequence_set,
            kind,
            response,
            flags,
            uid,
        })
    }

    /// Construct a COPY command.
    pub fn copy<S, M>(
        sequence_set: S,
        mailbox: M,
        uid: bool,
    ) -> Result<Self, CopyError<S::Error, M::Error>>
    where
        S: TryInto<SequenceSet>,
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::Copy {
            sequence_set: sequence_set.try_into().map_err(CopyError::Sequence)?,
            mailbox: mailbox.try_into().map_err(CopyError::Mailbox)?,
            uid,
        })
    }

    /// Get the name of the command.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Capability => "CAPABILITY",
            Self::Noop => "NOOP",
            Self::Logout => "LOGOUT",
            #[cfg(feature = "starttls")]
            Self::StartTLS => "STARTTLS",
            Self::Authenticate { .. } => "AUTHENTICATE",
            Self::Login { .. } => "LOGIN",
            Self::Select { .. } => "SELECT",
            Self::Sort { .. } => "SORT",
            Self::Thread { .. } => "THREAD",
            Self::Unselect => "UNSELECT",
            Self::Examine { .. } => "EXAMINE",
            Self::Create { .. } => "CREATE",
            Self::Delete { .. } => "DELETE",
            Self::Rename { .. } => "RENAME",
            Self::Subscribe { .. } => "SUBSCRIBE",
            Self::Unsubscribe { .. } => "UNSUBSCRIBE",
            Self::List { .. } => "LIST",
            Self::Lsub { .. } => "LSUB",
            Self::Status { .. } => "STATUS",
            Self::Append { .. } => "APPEND",
            Self::Check => "CHECK",
            Self::Close => "CLOSE",
            Self::Expunge => "EXPUNGE",
            Self::ExpungeUid { .. } => "EXPUNGE",
            Self::Search { .. } => "SEARCH",
            Self::Fetch { .. } => "FETCH",
            Self::Store { .. } => "STORE",
            Self::Copy { .. } => "COPY",
            Self::Idle => "IDLE",
            Self::Enable { .. } => "ENABLE",
            Self::Compress { .. } => "COMPRESS",
            Self::GetQuota { .. } => "GETQUOTA",
            Self::GetQuotaRoot { .. } => "GETQUOTAROOT",
            Self::SetQuota { .. } => "SETQUOTA",
            Self::Move { .. } => "MOVE",
            #[cfg(feature = "ext_id")]
            Self::Id { .. } => "ID",
            #[cfg(feature = "ext_metadata")]
            Self::SetMetadata { .. } => "SETMETADATA",
            #[cfg(feature = "ext_metadata")]
            Self::GetMetadata { .. } => "GETMETADATA",
        }
    }
}

/// Error-related types.
pub mod error {
    use thiserror::Error;

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum LoginError<U, P> {
        #[error("Invalid username: {0}")]
        Username(U),
        #[error("Invalid password: {0}")]
        Password(P),
    }

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum RenameError<F, T> {
        #[error("Invalid (from) mailbox: {0}")]
        From(F),
        #[error("Invalid (to) mailbox: {0}")]
        To(T),
    }

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum ListError<R, M> {
        #[error("Invalid reference: {0}")]
        Reference(R),
        #[error("Invalid mailbox: {0}")]
        Mailbox(M),
    }

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum AppendError<M, D> {
        #[error("Invalid mailbox: {0}")]
        Mailbox(M),
        #[error("Invalid data: {0}")]
        Data(D),
    }

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum CopyError<S, M> {
        #[error("Invalid sequence: {0}")]
        Sequence(S),
        #[error("Invalid mailbox: {0}")]
        Mailbox(M),
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime as ChronoDateTime;

    use super::*;
    use crate::{
        auth::AuthMechanism,
        core::{AString, Charset, IString, Literal, LiteralMode, Vec1},
        datetime::DateTime,
        extensions::{
            binary::Literal8,
            compress::CompressionAlgorithm,
            enable::{CapabilityEnable, Utf8Kind},
        },
        fetch::{Macro, MacroOrMessageDataItemNames, MessageDataItemName, Part, Section},
        flag::{Flag, StoreType},
        mailbox::{ListMailbox, Mailbox},
        search::SearchKey,
        secret::Secret,
        sequence::{SeqOrUid, Sequence, SequenceSet},
        status::StatusDataItemName,
    };

    #[test]
    fn test_conversion_command_body() {
        let cmds = vec![
            CommandBody::Capability,
            CommandBody::Noop,
            CommandBody::Logout,
            #[cfg(feature = "starttls")]
            CommandBody::StartTLS,
            CommandBody::authenticate(AuthMechanism::Plain),
            CommandBody::authenticate(AuthMechanism::Login),
            CommandBody::authenticate_with_ir(AuthMechanism::Plain, b"XXXXXXXX".as_ref()),
            CommandBody::authenticate_with_ir(AuthMechanism::Login, b"YYYYYYYY".as_ref()),
            CommandBody::login("alice", "I_am_an_atom").unwrap(),
            CommandBody::login("alice", "I am \\ \"quoted\"").unwrap(),
            CommandBody::login("alice", "I am a literal²").unwrap(),
            CommandBody::login(
                AString::Atom("alice".try_into().unwrap()),
                AString::String(crate::core::IString::Literal(
                    vec![0xff, 0xff, 0xff].try_into().unwrap(),
                )),
            )
            .unwrap(),
            CommandBody::select("inbox").unwrap(),
            CommandBody::select("atom").unwrap(),
            CommandBody::select("C:\\").unwrap(),
            CommandBody::select("²").unwrap(),
            CommandBody::select("Trash").unwrap(),
            CommandBody::examine("inbox").unwrap(),
            CommandBody::examine("atom").unwrap(),
            CommandBody::examine("C:\\").unwrap(),
            CommandBody::examine("²").unwrap(),
            CommandBody::examine("Trash").unwrap(),
            CommandBody::create("inBoX").unwrap(),
            CommandBody::delete("inBOX").unwrap(),
            CommandBody::rename("iNBoS", "INboX").unwrap(),
            CommandBody::subscribe("inbox").unwrap(),
            CommandBody::unsubscribe("INBOX").unwrap(),
            CommandBody::list("iNbOx", "test").unwrap(),
            CommandBody::list("inbox", ListMailbox::Token("test".try_into().unwrap())).unwrap(),
            CommandBody::lsub(
                "inbox",
                ListMailbox::String(IString::Quoted("\x7f".try_into().unwrap())),
            )
            .unwrap(),
            CommandBody::list("inBoX", ListMailbox::Token("test".try_into().unwrap())).unwrap(),
            CommandBody::lsub(
                "INBOX",
                ListMailbox::String(IString::Quoted("\x7f".try_into().unwrap())),
            )
            .unwrap(),
            CommandBody::status("inbox", vec![StatusDataItemName::Messages]).unwrap(),
            CommandBody::append(
                "inbox",
                vec![],
                Some(
                    DateTime::try_from(
                        ChronoDateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0200")
                            .unwrap(),
                    )
                    .unwrap(),
                ),
                vec![0xff, 0xff, 0xff],
            )
            .unwrap(),
            CommandBody::append(
                "inbox",
                vec![Flag::Keyword("test".try_into().unwrap())],
                Some(
                    DateTime::try_from(
                        ChronoDateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0200")
                            .unwrap(),
                    )
                    .unwrap(),
                ),
                vec![0xff, 0xff, 0xff],
            )
            .unwrap(),
            CommandBody::Check,
            CommandBody::Close,
            CommandBody::Expunge,
            CommandBody::search(
                None,
                Vec1::from(SearchKey::And(
                    vec![SearchKey::All, SearchKey::New, SearchKey::Unseen]
                        .try_into()
                        .unwrap(),
                )),
                false,
            ),
            CommandBody::search(
                None,
                Vec1::from(SearchKey::And(
                    vec![SearchKey::All, SearchKey::New, SearchKey::Unseen]
                        .try_into()
                        .unwrap(),
                )),
                true,
            ),
            CommandBody::search(
                None,
                Vec1::from(SearchKey::And(
                    vec![SearchKey::SequenceSet(SequenceSet(
                        vec![Sequence::Single(SeqOrUid::Value(42.try_into().unwrap()))]
                            .try_into()
                            .unwrap(),
                    ))]
                    .try_into()
                    .unwrap(),
                )),
                true,
            ),
            CommandBody::search(
                None,
                Vec1::from(SearchKey::SequenceSet("42".try_into().unwrap())),
                true,
            ),
            CommandBody::search(
                None,
                Vec1::from(SearchKey::SequenceSet("*".try_into().unwrap())),
                true,
            ),
            CommandBody::search(
                None,
                Vec1::from(SearchKey::Or(
                    Box::new(SearchKey::Draft),
                    Box::new(SearchKey::All),
                )),
                true,
            ),
            CommandBody::search(
                Some(Charset::try_from("UTF-8").unwrap()),
                Vec1::from(SearchKey::Or(
                    Box::new(SearchKey::Draft),
                    Box::new(SearchKey::All),
                )),
                true,
            ),
            CommandBody::fetch(
                "1",
                vec![MessageDataItemName::BodyExt {
                    partial: None,
                    section: Some(Section::Part(Part(
                        vec![1.try_into().unwrap(), 1.try_into().unwrap()]
                            .try_into()
                            .unwrap(),
                    ))),
                    peek: true,
                }],
                false,
            )
            .unwrap(),
            CommandBody::fetch("1:*,2,3", Macro::Full, true).unwrap(),
            CommandBody::store(
                "1,2:*",
                StoreType::Remove,
                StoreResponse::Answer,
                vec![Flag::Seen, Flag::Draft],
                false,
            )
            .unwrap(),
            CommandBody::store(
                "1:5",
                StoreType::Add,
                StoreResponse::Answer,
                vec![Flag::Keyword("TEST".try_into().unwrap())],
                true,
            )
            .unwrap(),
            CommandBody::copy("1", "inbox", false).unwrap(),
            CommandBody::copy("1337", "archive", true).unwrap(),
        ];

        for (no, cmd_body) in cmds.into_iter().enumerate() {
            println!("Test: {}, {:?}", no, cmd_body);

            let _ = cmd_body.tag(format!("A{}", no)).unwrap();
        }
    }

    #[test]
    fn test_command_body_name() {
        let tests = [
            (CommandBody::Capability, "CAPABILITY"),
            (CommandBody::Noop, "NOOP"),
            (CommandBody::Logout, "LOGOUT"),
            #[cfg(feature = "starttls")]
            (CommandBody::StartTLS, "STARTTLS"),
            (
                CommandBody::Authenticate {
                    mechanism: AuthMechanism::Plain,
                    initial_response: None,
                },
                "AUTHENTICATE",
            ),
            (
                CommandBody::Login {
                    username: AString::try_from("user").unwrap(),
                    password: Secret::new(AString::try_from("pass").unwrap()),
                },
                "LOGIN",
            ),
            (
                CommandBody::Select {
                    mailbox: Mailbox::Inbox,
                },
                "SELECT",
            ),
            (CommandBody::Unselect, "UNSELECT"),
            (
                CommandBody::Examine {
                    mailbox: Mailbox::Inbox,
                },
                "EXAMINE",
            ),
            (
                CommandBody::Create {
                    mailbox: Mailbox::Inbox,
                },
                "CREATE",
            ),
            (
                CommandBody::Delete {
                    mailbox: Mailbox::Inbox,
                },
                "DELETE",
            ),
            (
                CommandBody::Rename {
                    from: Mailbox::Inbox,
                    to: Mailbox::Inbox,
                },
                "RENAME",
            ),
            (
                CommandBody::Subscribe {
                    mailbox: Mailbox::Inbox,
                },
                "SUBSCRIBE",
            ),
            (
                CommandBody::Unsubscribe {
                    mailbox: Mailbox::Inbox,
                },
                "UNSUBSCRIBE",
            ),
            (
                CommandBody::List {
                    reference: Mailbox::Inbox,
                    mailbox_wildcard: ListMailbox::try_from("").unwrap(),
                },
                "LIST",
            ),
            (
                CommandBody::Lsub {
                    reference: Mailbox::Inbox,
                    mailbox_wildcard: ListMailbox::try_from("").unwrap(),
                },
                "LSUB",
            ),
            (
                CommandBody::Status {
                    mailbox: Mailbox::Inbox,
                    item_names: vec![].into(),
                },
                "STATUS",
            ),
            (
                CommandBody::Append {
                    mailbox: Mailbox::Inbox,
                    flags: vec![],
                    date: None,
                    message: LiteralOrLiteral8::Literal(Literal::try_from("").unwrap()),
                },
                "APPEND",
            ),
            (
                CommandBody::Append {
                    mailbox: Mailbox::Inbox,
                    flags: vec![],
                    date: None,
                    message: LiteralOrLiteral8::Literal8(Literal8 {
                        data: b"Hello\x00World\x00".as_ref().into(),
                        mode: LiteralMode::NonSync,
                    }),
                },
                "APPEND",
            ),
            (CommandBody::Check, "CHECK"),
            (CommandBody::Close, "CLOSE"),
            (CommandBody::Expunge, "EXPUNGE"),
            (
                CommandBody::Search {
                    charset: None,
                    criteria: Vec1::from(SearchKey::Recent),
                    uid: true,
                },
                "SEARCH",
            ),
            (
                CommandBody::Fetch {
                    sequence_set: SequenceSet::try_from(1u32).unwrap(),
                    macro_or_item_names: MacroOrMessageDataItemNames::Macro(Macro::Full),
                    uid: true,
                },
                "FETCH",
            ),
            (
                CommandBody::Store {
                    sequence_set: SequenceSet::try_from(1).unwrap(),
                    flags: vec![],
                    response: StoreResponse::Silent,
                    kind: StoreType::Add,
                    uid: true,
                },
                "STORE",
            ),
            (
                CommandBody::Copy {
                    sequence_set: SequenceSet::try_from(1).unwrap(),
                    mailbox: Mailbox::Inbox,
                    uid: true,
                },
                "COPY",
            ),
            (CommandBody::Idle, "IDLE"),
            (
                CommandBody::Enable {
                    capabilities: Vec1::from(CapabilityEnable::Utf8(Utf8Kind::Only)),
                },
                "ENABLE",
            ),
            (
                CommandBody::Compress {
                    algorithm: CompressionAlgorithm::Deflate,
                },
                "COMPRESS",
            ),
            (
                CommandBody::GetQuota {
                    root: AString::try_from("root").unwrap(),
                },
                "GETQUOTA",
            ),
            (
                CommandBody::GetQuotaRoot {
                    mailbox: Mailbox::Inbox,
                },
                "GETQUOTAROOT",
            ),
            (
                CommandBody::SetQuota {
                    root: AString::try_from("root").unwrap(),
                    quotas: vec![],
                },
                "SETQUOTA",
            ),
            (
                CommandBody::Move {
                    sequence_set: SequenceSet::try_from(1).unwrap(),
                    mailbox: Mailbox::Inbox,
                    uid: true,
                },
                "MOVE",
            ),
        ];

        for (test, expected) in tests {
            assert_eq!(test.name(), expected);
        }
    }
}
