//! Client Commands
//!
//! see <https://tools.ietf.org/html/rfc3501#section-6>

#[cfg(feature = "ext_sasl_ir")]
use std::borrow::Cow;
use std::convert::TryInto;

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "ext_compress")]
use crate::extensions::compress::CompressionAlgorithm;
#[cfg(feature = "ext_enable")]
use crate::extensions::enable::CapabilityEnable;
#[cfg(feature = "ext_quota")]
use crate::extensions::quota::{QuotaSet, SetQuotaError};
use crate::{
    command::{
        fetch::MacroOrFetchAttributes,
        status::StatusAttribute,
        store::{StoreResponse, StoreType},
        ListMailbox, SequenceSet,
    },
    core::{AString, Atom, Literal, NonEmptyVec},
    message::{AuthMechanism, Charset, DateTime, Flag, Mailbox, MyNaiveDate, Tag},
    security::Secret,
};

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Command<'a> {
    pub tag: Tag<'a>,
    pub body: CommandBody<'a>,
}

impl<'a> Command<'a> {
    pub fn new<T>(tag: T, body: CommandBody<'a>) -> Result<Self, T::Error>
    where
        T: TryInto<Tag<'a>>,
    {
        Ok(Self {
            tag: tag.try_into()?,
            body,
        })
    }

    pub fn name(&self) -> &'static str {
        self.body.name()
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    /// A [TLS] negotiation begins immediately after the CRLF at the end
    /// of the tagged OK response from the server.  Once a client issues a
    /// STARTTLS command, it MUST NOT issue further commands until a
    /// server response is seen and the [TLS] negotiation is complete.
    ///
    /// The server remains in the non-authenticated state, even if client
    /// credentials are supplied during the [TLS] negotiation.  This does
    /// not preclude an authentication mechanism such as EXTERNAL (defined
    /// in [SASL]) from using client identity determined by the [TLS]
    /// negotiation.
    ///
    /// Once [TLS] has been started, the client MUST discard cached
    /// information about server capabilities and SHOULD re-issue the
    /// CAPABILITY command.  This is necessary to protect against man-in-
    /// the-middle attacks which alter the capabilities list prior to
    /// STARTTLS.  The server MAY advertise different capabilities after
    /// STARTTLS.
    #[cfg(feature = "starttls")]
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
    /// The AUTHENTICATE command indicates a [SASL] authentication
    /// mechanism to the server.  If the server supports the requested
    /// authentication mechanism, it performs an authentication protocol
    /// exchange to authenticate and identify the client.  It MAY also
    /// negotiate an OPTIONAL security layer for subsequent protocol
    /// interactions.  If the requested authentication mechanism is not
    /// supported, the server SHOULD reject the AUTHENTICATE command by
    /// sending a tagged NO response.
    ///
    /// The AUTHENTICATE command does not support the optional "initial
    /// response" feature of [SASL].  Section 5.1 of [SASL] specifies how
    /// to handle an authentication mechanism which uses an initial
    /// response.
    ///
    /// The service name specified by this protocol's profile of [SASL] is
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
    /// If a security layer is negotiated through the [SASL]
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
    ///   additional [SASL] mechanisms that do not use plaintext
    ///   passwords, such the GSSAPI mechanism described in [SASL]
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
    /// protected by encryption/integrity checking.  [SASL] requires the
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
        mechanism: AuthMechanism<'a>,
        /// Already base64-decoded
        #[cfg(feature = "ext_sasl_ir")]
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
        username: AString<'a>,
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
    ///   <n> EXISTS  The number of messages in the mailbox.  See the
    ///               description of the EXISTS response for more detail.
    ///
    ///   <n> RECENT  The number of messages with the \Recent flag set.
    ///               See the description of the RECENT response for more
    ///               detail.
    ///
    ///   OK [UNSEEN <n>]
    ///               The message sequence number of the first unseen
    ///               message in the mailbox.  If this is missing, the
    ///               client can not make any assumptions about the first
    ///               unseen message in the mailbox, and needs to issue a
    ///               SEARCH command if it wants to find it.
    ///
    ///   OK [PERMANENTFLAGS (<list of flags>)]
    ///               A list of message flags that the client can change
    ///               permanently.  If this is missing, the client should
    ///               assume that all flags can be changed permanently.
    ///
    ///   OK [UIDNEXT <n>]
    ///               The next unique identifier value.  Refer to section
    ///               2.3.1.1 for more information.  If this is missing,
    ///               the client can not make any assumptions about the
    ///               next unique identifier value.
    ///
    ///   OK [UIDVALIDITY <n>]
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
    Select { mailbox: Mailbox<'a> },

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
    Examine { mailbox: Mailbox<'a> },

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
    Create { mailbox: Mailbox<'a> },

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
    Delete { mailbox: Mailbox<'a> },

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
    Rename { from: Mailbox<'a>, to: Mailbox<'a> },

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
    Subscribe { mailbox: Mailbox<'a> },

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
    Unsubscribe { mailbox: Mailbox<'a> },

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
        reference: Mailbox<'a>,
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
        reference: Mailbox<'a>,
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
        mailbox: Mailbox<'a>,
        attributes: Vec<StatusAttribute>,
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
    /// server MUST send the response code "[TRYCREATE]" as the prefix of
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
    ///   because it does not provide a mechanism to transfer [SMTP]
    ///   envelope information.
    Append {
        mailbox: Mailbox<'a>,
        flags: Vec<Flag<'a>>,
        date: Option<DateTime>,
        message: Literal<'a>,
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
    /// Arguments:  none
    /// Responses:  untagged responses: EXPUNGE
    /// Result:     OK - expunge completed
    ///             NO - expunge failure: can't expunge (e.g., permission
    ///                  denied)
    ///             BAD - command unknown or arguments invalid
    ///
    /// The EXPUNGE command permanently removes all messages that have the
    /// \Deleted flag set from the currently selected mailbox.  Before
    /// returning an OK to the client, an untagged EXPUNGE response is
    /// sent for each message that is removed.
    ///
    ///   Note: In this example, messages 3, 4, 7, and 11 had the
    ///   \Deleted flag set.  See the description of the EXPUNGE
    ///   response for further explanation.
    Expunge,

    /// ### 6.4.4.  SEARCH Command
    ///
    /// * Arguments:
    ///   * OPTIONAL [CHARSET] specification
    ///   * searching criteria (one or more)
    /// * Responses:  REQUIRED untagged response: SEARCH
    /// * Result:
    ///   * OK - search completed
    ///   * NO - search error: can't search that [CHARSET] or criteria
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
    /// The OPTIONAL [CHARSET] specification consists of the word
    /// "CHARSET" followed by a registered [CHARSET].  It indicates the
    /// [CHARSET] of the strings that appear in the search criteria.
    /// [MIME-IMB] content transfer encodings, and [MIME-HDRS] strings in
    /// [RFC-2822]/[MIME-IMB] headers, MUST be decoded before comparing
    /// text in a [CHARSET] other than US-ASCII.  US-ASCII MUST be
    /// supported; other [CHARSET]s MAY be supported.
    ///
    /// If the server does not support the specified [CHARSET], it MUST
    /// return a tagged NO response (not a BAD).  This response SHOULD
    /// contain the BADCHARSET response code, which MAY list the
    /// [CHARSET]s supported by the server.
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
        charset: Option<Charset<'a>>,
        criteria: SearchKey<'a>,
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
        sequence_set: SequenceSet,
        attributes: MacroOrFetchAttributes<'a>,
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
    /// FLAGS <flag list>
    ///    Replace the flags for the message (other than \Recent) with the
    ///    argument.  The new value of the flags is returned as if a FETCH
    ///    of those flags was done.
    ///
    /// FLAGS.SILENT <flag list>
    ///    Equivalent to FLAGS, but without returning a new value.
    ///
    /// +FLAGS <flag list>
    ///    Add the argument to the flags for the message.  The new value
    ///    of the flags is returned as if a FETCH of those flags was done.
    ///
    /// +FLAGS.SILENT <flag list>
    ///    Equivalent to +FLAGS, but without returning a new value.
    ///
    /// -FLAGS <flag list>
    ///    Remove the argument from the flags for the message.  The new
    ///    value of the flags is returned as if a FETCH of those flags was
    ///    done.
    ///
    /// -FLAGS.SILENT <flag list>
    ///    Equivalent to -FLAGS, but without returning a new value.
    Store {
        sequence_set: SequenceSet,
        kind: StoreType,
        response: StoreResponse,
        flags: Vec<Flag<'a>>, // FIXME(misuse): must not accept "\*" or "\Recent"
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
    /// server MUST send the response code "[TRYCREATE]" as the prefix of
    /// the text of the tagged NO response.  This gives a hint to the
    /// client that it can attempt a CREATE command and retry the COPY if
    /// the CREATE is successful.
    ///
    /// If the COPY command is unsuccessful for any reason, server
    /// implementations MUST restore the destination mailbox to its state
    /// before the COPY attempt.
    Copy {
        sequence_set: SequenceSet,
        mailbox: Mailbox<'a>,
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
    #[cfg(feature = "ext_idle")]
    Idle,

    #[cfg(feature = "ext_enable")]
    Enable {
        capabilities: NonEmptyVec<CapabilityEnable<'a>>,
    },

    #[cfg(feature = "ext_compress")]
    Compress { algorithm: CompressionAlgorithm },

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
    #[cfg(feature = "ext_quota")]
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
    #[cfg(feature = "ext_quota")]
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
    #[cfg(feature = "ext_quota")]
    SetQuota {
        /// Name of quota root.
        root: AString<'a>,
        /// List of resource limits.
        quotas: Vec<QuotaSet<'a>>,
    },
}

impl<'a> CommandBody<'a> {
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

    #[cfg(not(feature = "ext_sasl_ir"))]
    pub fn authenticate(mechanism: AuthMechanism<'a>) -> Self {
        CommandBody::Authenticate { mechanism }
    }

    #[cfg(feature = "ext_sasl_ir")]
    pub fn authenticate(mechanism: AuthMechanism<'a>, initial_response: Option<&'a [u8]>) -> Self {
        CommandBody::Authenticate {
            mechanism,
            initial_response: initial_response.map(|ir| Secret::new(Cow::Borrowed(ir))),
        }
    }

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

    pub fn select<M>(mailbox: M) -> Result<Self, M::Error>
    where
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::Select {
            mailbox: mailbox.try_into()?,
        })
    }

    pub fn examine<M>(mailbox: M) -> Result<Self, M::Error>
    where
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::Examine {
            mailbox: mailbox.try_into()?,
        })
    }

    pub fn create<M>(mailbox: M) -> Result<Self, M::Error>
    where
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::Create {
            mailbox: mailbox.try_into()?,
        })
    }

    pub fn delete<M>(mailbox: M) -> Result<Self, M::Error>
    where
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::Delete {
            mailbox: mailbox.try_into()?,
        })
    }

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

    pub fn subscribe<M>(mailbox: M) -> Result<Self, M::Error>
    where
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::Subscribe {
            mailbox: mailbox.try_into()?,
        })
    }

    pub fn unsubscribe<M>(mailbox: M) -> Result<Self, M::Error>
    where
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::Unsubscribe {
            mailbox: mailbox.try_into()?,
        })
    }

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

    pub fn status<M>(mailbox: M, attributes: Vec<StatusAttribute>) -> Result<Self, M::Error>
    where
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::Status {
            mailbox: mailbox.try_into()?,
            attributes,
        })
    }

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
            message: message.try_into().map_err(AppendError::Data)?,
        })
    }

    pub fn search(charset: Option<Charset<'a>>, criteria: SearchKey<'a>, uid: bool) -> Self {
        CommandBody::Search {
            charset,
            criteria,
            uid,
        }
    }

    pub fn fetch<S, I>(sequence_set: S, attributes: I, uid: bool) -> Result<Self, S::Error>
    where
        S: TryInto<SequenceSet>,
        I: Into<MacroOrFetchAttributes<'a>>,
    {
        let sequence_set = sequence_set.try_into()?;

        Ok(CommandBody::Fetch {
            sequence_set,
            attributes: attributes.into(),
            uid,
        })
    }

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

    #[cfg(feature = "ext_enable")]
    pub fn enable<C>(capabilities: C) -> Result<Self, C::Error>
    where
        C: TryInto<NonEmptyVec<CapabilityEnable<'a>>>,
    {
        Ok(CommandBody::Enable {
            capabilities: capabilities.try_into()?,
        })
    }

    #[cfg(feature = "ext_compress")]
    pub fn compress(algorithm: CompressionAlgorithm) -> Self {
        CommandBody::Compress { algorithm }
    }

    #[cfg(feature = "ext_quota")]
    pub fn get_quota<A>(root: A) -> Result<Self, A::Error>
    where
        A: TryInto<AString<'a>>,
    {
        Ok(CommandBody::GetQuota {
            root: root.try_into()?,
        })
    }

    #[cfg(feature = "ext_quota")]
    pub fn get_quota_root<M>(mailbox: M) -> Result<Self, M::Error>
    where
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::GetQuotaRoot {
            mailbox: mailbox.try_into()?,
        })
    }

    #[cfg(feature = "ext_quota")]
    pub fn set_quota<R, S>(root: R, quotas: S) -> Result<Self, SetQuotaError<R::Error, S::Error>>
    where
        R: TryInto<AString<'a>>,
        S: TryInto<Vec<QuotaSet<'a>>>,
    {
        Ok(CommandBody::SetQuota {
            root: root.try_into().map_err(SetQuotaError::Root)?,
            quotas: quotas.try_into().map_err(SetQuotaError::QuotaSet)?,
        })
    }

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
            Self::Search { .. } => "SEARCH",
            Self::Fetch { .. } => "FETCH",
            Self::Store { .. } => "STORE",
            Self::Copy { .. } => "COPY",
            #[cfg(feature = "ext_idle")]
            Self::Idle => "IDLE",
            #[cfg(feature = "ext_enable")]
            Self::Enable { .. } => "ENABLE",
            #[cfg(feature = "ext_compress")]
            Self::Compress { .. } => "COMPRESS",
            #[cfg(feature = "ext_quota")]
            Self::GetQuota { .. } => "GETQUOTA",
            #[cfg(feature = "ext_quota")]
            Self::GetQuotaRoot { .. } => "GETQUOTAROOT",
            #[cfg(feature = "ext_quota")]
            Self::SetQuota { .. } => "SETQUOTA",
        }
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum LoginError<U, P> {
    #[error("Invalid username: {0:?}")]
    Username(U),
    #[error("Invalid password: {0:?}")]
    Password(P),
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum RenameError<F, T> {
    #[error("Invalid mailbox (from): {0:?}")]
    From(F),
    #[error("Invalid mailbox (to): {0:?}")]
    To(T),
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum ListError<R, M> {
    #[error("Invalid reference: {0:?}")]
    Reference(R),
    #[error("Invalid mailbox: {0:?}")]
    Mailbox(M),
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum AppendError<M, D> {
    #[error("Invalid mailbox: {0:?}")]
    Mailbox(M),
    #[error("Invalid data: {0:?}")]
    Data(D),
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum CopyError<S, M> {
    #[error("Invalid sequence: {0:?}")]
    Sequence(S),
    #[error("Invalid mailbox: {0:?}")]
    Mailbox(M),
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthenticateData(pub Secret<Vec<u8>>);

/// The defined search keys.
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SearchKey<'a> {
    // <Not in RFC.>
    //
    // IMAP doesn't have a dedicated AND operator in its search syntax.
    // ANDing multiple search keys works by concatenating them with an ascii space.
    // Introducing this variant makes sense, because
    //   * it may help in understanding the RFC
    //   * and it can be used to distinguish between a single search key
    //     and multiple search keys.
    //
    // See also the corresponding `search` parser.
    And(NonEmptyVec<SearchKey<'a>>),

    /// Messages with message sequence numbers corresponding to the
    /// specified message sequence number set.
    SequenceSet(SequenceSet),

    /// All messages in the mailbox; the default initial key for ANDing.
    All,

    /// Messages with the \Answered flag set.
    Answered,

    /// Messages that contain the specified string in the envelope
    /// structure's BCC field.
    Bcc(AString<'a>),

    /// Messages whose internal date (disregarding time and timezone)
    /// is earlier than the specified date.
    Before(MyNaiveDate),

    /// Messages that contain the specified string in the body of the
    /// message.
    Body(AString<'a>),

    /// Messages that contain the specified string in the envelope
    /// structure's CC field.
    Cc(AString<'a>),

    /// Messages with the \Deleted flag set.
    Deleted,

    /// Messages with the \Draft flag set.
    Draft,

    /// Messages with the \Flagged flag set.
    Flagged,

    /// Messages that contain the specified string in the envelope
    /// structure's FROM field.
    From(AString<'a>),

    /// Messages that have a header with the specified field-name (as
    /// defined in [RFC-2822]) and that contains the specified string
    /// in the text of the header (what comes after the colon).  If the
    /// string to search is zero-length, this matches all messages that
    /// have a header line with the specified field-name regardless of
    /// the contents.
    Header(AString<'a>, AString<'a>),

    /// Messages with the specified keyword flag set.
    Keyword(Atom<'a>),

    /// Messages with an [RFC-2822] size larger than the specified
    /// number of octets.
    Larger(u32),

    /// Messages that have the \Recent flag set but not the \Seen flag.
    /// This is functionally equivalent to "(RECENT UNSEEN)".
    New,

    /// Messages that do not match the specified search key.
    Not(Box<SearchKey<'a>>), // TODO(misuse): is this a Vec or a single SearchKey?

    /// Messages that do not have the \Recent flag set.  This is
    /// functionally equivalent to "NOT RECENT" (as opposed to "NOT
    /// NEW").
    Old,

    /// Messages whose internal date (disregarding time and timezone)
    /// is within the specified date.
    On(MyNaiveDate),

    /// Messages that match either search key.
    Or(Box<SearchKey<'a>>, Box<SearchKey<'a>>), /* TODO(misuse): is this a Vec or a single SearchKey? */

    /// Messages that have the \Recent flag set.
    Recent,

    /// Messages that have the \Seen flag set.
    Seen,

    /// Messages whose [RFC-2822] Date: header (disregarding time and
    /// timezone) is earlier than the specified date.
    SentBefore(MyNaiveDate),

    /// Messages whose [RFC-2822] Date: header (disregarding time and
    /// timezone) is within the specified date.
    SentOn(MyNaiveDate),

    /// Messages whose [RFC-2822] Date: header (disregarding time and
    /// timezone) is within or later than the specified date.
    SentSince(MyNaiveDate),

    /// Messages whose internal date (disregarding time and timezone)
    /// is within or later than the specified date.
    Since(MyNaiveDate),

    /// Messages with an [RFC-2822] size smaller than the specified
    /// number of octets.
    Smaller(u32),

    /// Messages that contain the specified string in the envelope
    /// structure's SUBJECT field.
    Subject(AString<'a>),

    /// Messages that contain the specified string in the header or
    /// body of the message.
    Text(AString<'a>),

    /// Messages that contain the specified string in the envelope
    /// structure's TO field.
    To(AString<'a>),

    /// Messages with unique identifiers corresponding to the specified
    /// unique identifier set.  Sequence set ranges are permitted.
    Uid(SequenceSet),

    /// Messages that do not have the \Answered flag set.
    Unanswered,

    /// Messages that do not have the \Deleted flag set.
    Undeleted,

    /// Messages that do not have the \Draft flag set.
    Undraft,

    /// Messages that do not have the \Flagged flag set.
    Unflagged,

    /// Messages that do not have the specified keyword flag set.
    Unkeyword(Atom<'a>),

    /// Messages that do not have the \Seen flag set.
    Unseen,
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "ext_sasl_ir")]
    use std::borrow::Cow;
    #[cfg(feature = "ext_sasl_ir")]
    use std::convert::TryFrom;
    use std::convert::TryInto;

    use chrono::DateTime as ChronoDateTime;

    #[cfg(feature = "ext_compress")]
    use crate::extensions::compress::CompressionAlgorithm;
    #[cfg(feature = "ext_enable")]
    use crate::extensions::enable::{CapabilityEnable, Utf8Kind};
    #[cfg(feature = "ext_quota")]
    use crate::extensions::quota::{QuotaSet, Resource, ResourceOther};
    use crate::{
        codec::Encode,
        command::{
            fetch::{FetchAttribute, Macro},
            search::SearchKey,
            status::StatusAttribute,
            store::{StoreResponse, StoreType},
            CommandBody, ListMailbox,
        },
        core::{AString, IString},
        message::{AuthMechanism, DateTime, Flag, Part, Section},
    };
    #[cfg(feature = "ext_sasl_ir")]
    use crate::{command::Command, message::Tag, security::Secret};

    #[test]
    fn test_commandbody_new() {
        let cmds = vec![
            CommandBody::Capability,
            CommandBody::Noop,
            CommandBody::Logout,
            #[cfg(feature = "starttls")]
            CommandBody::StartTLS,
            CommandBody::authenticate(
                AuthMechanism::Plain,
                #[cfg(feature = "ext_sasl_ir")]
                None,
            ),
            CommandBody::authenticate(
                AuthMechanism::Login,
                #[cfg(feature = "ext_sasl_ir")]
                None,
            ),
            CommandBody::authenticate(
                AuthMechanism::Plain,
                #[cfg(feature = "ext_sasl_ir")]
                Some(b"XXXXXXXX"),
            ),
            CommandBody::authenticate(
                AuthMechanism::Login,
                #[cfg(feature = "ext_sasl_ir")]
                Some(b"YYYYYYYY"),
            ),
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
            CommandBody::status("inbox", vec![StatusAttribute::Messages]).unwrap(),
            CommandBody::append(
                "inbox",
                vec![],
                Some(DateTime(
                    ChronoDateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0200").unwrap(),
                )),
                vec![0xff, 0xff, 0xff],
            )
            .unwrap(),
            CommandBody::append(
                "inbox",
                vec![Flag::Keyword("test".try_into().unwrap())],
                Some(DateTime(
                    ChronoDateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0200").unwrap(),
                )),
                vec![0xff, 0xff, 0xff],
            )
            .unwrap(),
            CommandBody::Check,
            CommandBody::Close,
            CommandBody::Expunge,
            CommandBody::search(
                None,
                SearchKey::And(
                    vec![SearchKey::All, SearchKey::New, SearchKey::Unseen]
                        .try_into()
                        .unwrap(),
                ),
                false,
            ),
            CommandBody::search(
                None,
                SearchKey::And(
                    vec![SearchKey::All, SearchKey::New, SearchKey::Unseen]
                        .try_into()
                        .unwrap(),
                ),
                true,
            ),
            //Command::search(None, SearchKey::And(vec![SearchKey::SequenceSet(vec![Sequence::Single(SeqNo::Value(42))])]), true),
            CommandBody::search(None, SearchKey::SequenceSet("42".try_into().unwrap()), true),
            CommandBody::search(None, SearchKey::SequenceSet("*".try_into().unwrap()), true),
            CommandBody::search(
                None,
                SearchKey::Or(Box::new(SearchKey::Draft), Box::new(SearchKey::All)),
                true,
            ),
            CommandBody::fetch(
                "1",
                vec![FetchAttribute::BodyExt {
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
            #[cfg(feature = "ext_idle")]
            CommandBody::Idle,
            #[cfg(feature = "ext_enable")]
            CommandBody::enable(vec![CapabilityEnable::Utf8(Utf8Kind::Only)]).unwrap(),
            #[cfg(feature = "ext_enable")]
            CommandBody::enable(vec![CapabilityEnable::Utf8(Utf8Kind::Accept)]).unwrap(),
            #[cfg(feature = "ext_compress")]
            CommandBody::compress(CompressionAlgorithm::Deflate),
            #[cfg(feature = "ext_quota")]
            CommandBody::get_quota("").unwrap(),
            #[cfg(feature = "ext_quota")]
            CommandBody::get_quota_root("INBOX").unwrap(),
            #[cfg(feature = "ext_quota")]
            CommandBody::set_quota(
                "",
                vec![
                    QuotaSet::new(Resource::Message, 1337),
                    QuotaSet::new(Resource::Other(ResourceOther::try_from("spam").unwrap()), 0),
                ],
            )
            .unwrap(),
        ];

        for (no, cmd_body) in cmds.into_iter().enumerate() {
            println!("Test: {}, {:?}", no, cmd_body);

            let cmd = cmd_body.tag(format!("A{}", no)).unwrap();
            let serialized = cmd.encode_detached().unwrap();
            let printable = String::from_utf8_lossy(&serialized);
            print!("Serialized: {}", printable);
        }
    }

    #[cfg(feature = "ext_sasl_ir")]
    #[test]
    fn that_empty_ir_is_encoded_correctly() {
        let command = Command::new(
            Tag::try_from("A").unwrap(),
            CommandBody::Authenticate {
                mechanism: AuthMechanism::Plain,
                initial_response: Some(Secret::new(Cow::Borrowed(&b""[..]))),
            },
        )
        .unwrap();

        let buffer = command.encode_detached().unwrap();

        assert_eq!(buffer, b"A AUTHENTICATE PLAIN =\r\n")
    }
}
