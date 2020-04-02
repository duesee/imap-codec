//! # 3. State and Flow Diagram
//!
//! Once the connection between client and server is established, an
//! IMAP4rev1 connection is in one of four states.  The initial
//! state is identified in the server greeting.  Most commands are
//! only valid in certain states.  It is a protocol error for the
//! client to attempt a command while the connection is in an
//! inappropriate state, and the server will respond with a BAD or
//! NO (depending upon server implementation) command completion
//! result.
//!
//! ```text
//!           +----------------------+
//!           |connection established|
//!           +----------------------+
//!                      ||
//!                      \/
//!    +--------------------------------------+
//!    |          server greeting             |
//!    +--------------------------------------+
//!              || (1)       || (2)        || (3)
//!              \/           ||            ||
//!    +-----------------+    ||            ||
//!    |Not Authenticated|    ||            ||
//!    +-----------------+    ||            ||
//!     || (7)   || (4)       ||            ||
//!     ||       \/           \/            ||
//!     ||     +----------------+           ||
//!     ||     | Authenticated  |<=++       ||
//!     ||     +----------------+  ||       ||
//!     ||       || (7)   || (5)   || (6)   ||
//!     ||       ||       \/       ||       ||
//!     ||       ||    +--------+  ||       ||
//!     ||       ||    |Selected|==++       ||
//!     ||       ||    +--------+           ||
//!     ||       ||       || (7)            ||
//!     \/       \/       \/                \/
//!    +--------------------------------------+
//!    |               Logout                 |
//!    +--------------------------------------+
//!                      ||
//!                      \/
//!        +-------------------------------+
//!        |both sides close the connection|
//!        +-------------------------------+
//!
//! (1) connection without pre-authentication (OK greeting)
//! (2) pre-authenticated connection (PREAUTH greeting)
//! (3) rejected connection (BYE greeting)
//! (4) successful LOGIN or AUTHENTICATE command
//! (5) successful SELECT or EXAMINE command
//! (6) CLOSE command, or failed SELECT or EXAMINE command
//! (7) LOGOUT command, server shutdown, or connection closed
//! ```

use crate::types::mailbox::Mailbox;
use serde::Deserialize;

/// State of the IMAP4rev1 connection.
#[derive(Debug, Clone, Deserialize)]
pub enum State {
    /// ## 3.1. Not Authenticated State
    ///
    /// In the not authenticated state, the client MUST supply
    /// authentication credentials before most commands will be
    /// permitted.  This state is entered when a connection starts
    /// unless the connection has been pre-authenticated.
    NotAuthenticated,

    /// ## 3.2. Authenticated State
    ///
    /// In the authenticated state, the client is authenticated and MUST
    /// select a mailbox to access before commands that affect messages
    /// will be permitted.  This state is entered when a
    /// pre-authenticated connection starts, when acceptable
    /// authentication credentials have been provided, after an error in
    /// selecting a mailbox, or after a successful CLOSE command.
    Authenticated,

    /// ## 3.3. Selected State
    ///
    /// In a selected state, a mailbox has been selected to access.
    /// This state is entered when a mailbox has been successfully
    /// selected.
    Selected(Mailbox),

    /// ## 3.4. Logout State
    ///
    /// In the logout state, the connection is being terminated.  This
    /// state can be entered as a result of a client request (via the
    /// LOGOUT command) or by unilateral action on the part of either
    /// the client or server.
    ///
    /// If the client requests the logout state, the server MUST send an
    /// untagged BYE response and a tagged OK response to the LOGOUT
    /// command before the server closes the connection; and the client
    /// MUST read the tagged OK response to the LOGOUT command before
    /// the client closes the connection.
    ///
    /// A server MUST NOT unilaterally close the connection without
    /// sending an untagged BYE response that contains the reason for
    /// having done so.  A client SHOULD NOT unilaterally close the
    /// connection, and instead SHOULD issue a LOGOUT command.  If the
    /// server detects that the client has unilaterally closed the
    /// connection, the server MAY omit the untagged BYE response and
    /// simply close its connection.
    Logout,

    /// Extension IDLE
    IdleAuthenticated(String),

    /// Extension IDLE
    IdleSelected(String, Mailbox),
}
