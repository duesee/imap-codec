//! # IMAP protocol library
//!
//! imap-codec provides complete and detailed parsing and construction of [IMAP4rev1] commands and responses.
//! It is based on [imap-types] and extends it with parsing support using [nom].
//!
//! ## Example
//!
//! ```rust
//! use imap_codec::{
//!     codec::{Decode, Encode},
//!     imap_types::command::Command,
//! };
//!
//! // We assume here that the message is already complete.
//! let input = b"ABCD UID FETCH 1,2:* (BODY.PEEK[1.2.3.4.MIME]<42.1337>)\r\n";
//!
//! let (_remainder, parsed) = Command::decode(input).unwrap();
//! println!("// Parsed:");
//! println!("{parsed:#?}");
//!
//! let serialized = parsed.encode().dump();
//!
//! // Not every IMAP message is valid UTF-8.
//! // We ignore that here, so that we can print the message.
//! let serialized = String::from_utf8(serialized).unwrap();
//! println!("// Serialized:");
//! println!("// {serialized}");
//! ```
//!
//! ## Decoding
//!
//! Parsing is implemented through the [`Decode`](crate::codec::Decode) trait.
//! The main entry points for parsing are
//! [`Greeting::decode(...)`](imap_types::response::Greeting#method.decode) (to parse the first message from a server),
//! [`Command::decode(...)`](imap_types::command::Command#method.decode) (to parse commands from a client), and
//! [`Response::decode(...)`](imap_types::response::Response#method.decode) (to parse responses or results from a server).
//! Note, however, that certain message flows require other parsers as well.
//! Every parser takes an input (`&[u8]`) and produces a remainder and a parsed value.
//!
//! ### Example
//!
//! Have a look at the [parse_command](https://github.com/duesee/imap-codec/blob/main/examples/parse_command.rs) example to see how a real-world application could decode IMAP.
//!
//! IMAP literals make separating the parsing logic from the application logic difficult.
//! When a server recognizes a literal (e.g. "{42}"), it first needs to agree to receive more data by sending a so-called "continuation request" (`+ ...`).
//! Without a continuation request, a client won't send more data, and the parser on the server would always return `Incomplete(42)`.
//! This makes real-world decoding of IMAP a bit more elaborate.
//!
//! ## Encoding
//!
//! The [`Encode::encode(...)`](codec::Encode::encode) method will return an instance of [`Encoded`](codec::Encoded)
//! that facilitates handling of literals. The idea is that the encoder not only "dumps" the final serialization of a message but can be iterated over.
//!
//! ### Example
//!
//! ```rust
//! #[cfg(feature = "ext_literal")]
//! use imap_codec::imap_types::core::LiteralMode;
//! use imap_codec::{
//!     codec::{Decode, Encode, Fragment},
//!     imap_types::command::{Command, CommandBody},
//! };
//!
//! let command = Command::new("A1", CommandBody::login("Alice", "Pa²²W0rD").unwrap()).unwrap();
//!
//! for fragment in command.encode() {
//!     match fragment {
//!         Fragment::Line { data } => {
//!             // A line that is ready to be send.
//!             println!("C: {}", String::from_utf8(data).unwrap());
//!         }
//!         #[cfg(not(feature = "ext_literal"))]
//!         Fragment::Literal { data } => {
//!             // Wait for a continuation request.
//!             println!("S: + ...")
//!         }
//!         #[cfg(feature = "ext_literal")]
//!         Fragment::Literal { data, mode } => match mode {
//!             LiteralMode::Sync => {
//!                 // Wait for a continuation request.
//!                 println!("S: + ...")
//!             }
//!             LiteralMode::NonSync => {
//!                 // We don't need to wait for a continuation request
//!                 // as the server will also not send it.
//!             }
//!         },
//!     }
//! }
//! ```
//!
//! Output of example:
//!
//! ```imap
//! C: A1 LOGIN alice {10}
//! S: + ...
//! C: Pa²²W0rD
//! ```
//!
//! # Features
//!
//! imap-codec forwards many features to imap-types. See [imap-types features] for a comprehensive list.
//!
//! In addition, imap-codec defines the following features:
//!
//! | Feature               | Description                    | Enabled by default |
//! |-----------------------|--------------------------------|--------------------|
//! | quirk_crlf_relaxed    | Make `\r` in `\r\n` optional.  | No                 |
//! | quirk_rectify_numbers | Rectify (invalid) numbers.     | No                 |
//! | quirk_missing_text    | Rectify missing `text` element.| No                 |
//! | tokio                 | Tokio support.                 | No                 |
//!
//! ## Quirks
//!
//! Features starting with `quirk_` are used to cope with existing interoperability issues.
//! Unfortunately, we already observed some standard violations, such as, negative numbers, and missing syntax elements.
//! Our policy is as follows: If we see an interoperability issue, we file an issue in the corresponding implementation.
//! If, for any reason, the issue cannot be fixed, *and* the implementation is "important enough", e.g.,  because a user of
//! imap-codec can't otherwise access their emails, we may add a `quirk_` feature to quickly resolve the problem.
//! Of course, imap-codec should never violate the IMAP standard itself. So, we need to do this carefully.
//!
//! ## Tokio support
//!
//! The `tokio` feature unlocks an implementation of [tokio_util::codec].
//! See the [tokio client] and [tokio server] demos.
//!
//! [imap-types]: https://docs.rs/imap-types/latest/imap_types
//! [imap-types features]: https://docs.rs/imap-types/latest/imap_types/#features
//! [IMAP4rev1]: https://tools.ietf.org/html/rfc3501
//! [parse_command]: https://github.com/duesee/imap-codec/blob/main/examples/parse_command.rs
//! [tokio_util::codec]: https://docs.rs/tokio-util/latest/tokio_util/codec/index.html
//! [tokio client]: https://github.com/duesee/imap-codec/tree/main/assets/demos/tokio-client
//! [tokio server]: https://github.com/duesee/imap-codec/tree/main/assets/demos/tokio-server

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod auth;
mod body;
mod command;
mod core;
mod datetime;
mod envelope;
#[cfg(any(
    feature = "ext_compress",
    feature = "ext_condstore_qresync",
    feature = "ext_enable",
    feature = "ext_idle",
    feature = "ext_literal",
    feature = "ext_move",
    feature = "ext_quota",
    feature = "ext_unselect",
))]
#[cfg_attr(docsrs, doc(cfg(feature = "ext_*")))]
mod extensions;
mod fetch;
mod flag;
mod mailbox;
mod response;
mod search;
mod sequence;
mod status;
#[cfg(test)]
mod testing;
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub mod tokio;

pub mod codec;

// Re-export.
pub use imap_types;
