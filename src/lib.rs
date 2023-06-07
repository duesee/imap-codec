//! # IMAP Protocol Library
//!
//! imap-codec provides complete and detailed parsing and construction of [IMAP4rev1](https://tools.ietf.org/html/rfc3501) commands and responses.
//! It is based on [imap-types] and extends it with parsing support using [nom].
//!
//! ## Example
//!
//! ```rust
//! use imap_codec::{
//!     codec::{Decode, Encode},
//!     command::Command,
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
//! [`Greeting::decode(...)`](response::Greeting#method.decode) (to parse the first message from a server),
//! [`Command::decode(...)`](command::Command#method.decode) (to parse commands from a client), and
//! [`Response::decode(...)`](response::Response#method.decode) (to parse responses or results from a server).
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
//! that facilitates handling of literals (and other protocol flows). The idea is that the encoder not only "dumps"
//! the final serialization of a message but can be iterated over.
//!
//! ### Example
//!
//! ```rust
//! use imap_codec::{
//!     codec::{Decode, Encode, Fragment},
//!     command::Command,
//! };
//! use imap_types::command::CommandBody;
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
//!         Fragment::Literal { data, sync } => {
//!             if sync {
//!                 // Wait for a continuation request.
//!                 println!("S: + ...")
//!             } else {
//!                 // We don't need to wait for a continuation request
//!                 // as the server will also not send it.
//!             }
//!         }
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
//! This crate uses the following features to enable IMAP extensions:
//!
//! |Feature              |Description                                                 |Enabled by default |
//! |---------------------|------------------------------------------------------------|-------------------|
//! |ext_compress         |The IMAP COMPRESS Extension ([RFC 4978])                    |No (but may change)|
//! |ext_enable           |The IMAP ENABLE Extension ([RFC 5161])                      |No (but may change)|
//! |ext_idle             |IMAP4 IDLE command ([RFC 2177])                             |No (but may change)|
//! |ext_literal          |IMAP4 Non-synchronizing Literals ([RFC 2088], [RFC 7888])   |No (but may change)|
//! |ext_login_referrals  |IMAP4 Login Referrals ([RFC 2221])                          |No (but may change)|
//! |ext_mailbox_referrals|IMAP4 Mailbox Referrals ([RFC 2193])                        |No (but may change)|
//! |ext_move             |IMAP MOVE Extension ([RFC 6851])                            |No (but may change)|
//! |ext_quota            |IMAP QUOTA Extension ([RFC 9208])                           |No (but may change)|
//! |ext_sasl_ir          |IMAP Extension for SASL Initial Client Response ([RFC 4959])|No (but may change)|
//! |ext_unselect         |IMAP UNSELECT command ([RFC 3691])                          |No (but may change)|
//! |starttls             |IMAP4rev1 ([RFC 3501]; section 6.2.1)                       |No                 |
//!
//! Experimental (or unfinished) features:
//!
//! |Feature              |Description                                                                          |Enabled by default |
//! |---------------------|-------------------------------------------------------------------------------------|-------------------|
//! |ext_condstore_qresync|Quick Flag Changes Resynchronization and Quick Mailbox Resynchronization ([RFC 7162])|No (but may change)|
//!
//! Features prefixed with "ext_" are IMAP extensions and often require a more elaborate message flow.
//! STARTTLS is not considered an extension but feature-gated because it [should be avoided](https://nostarttls.secvuln.info/).
//! For better performance and security, use "implicit TLS", i.e., IMAP-over-TLS on port 993, and don't use STARTTLS at all.
//!
//! Furthermore, imap-codec uses the following features to facilitate interoperability:
//!
//! | Feature          | Description                                                    | Enabled by default |
//! |------------------|----------------------------------------------------------------|--------------------|
//! | arbitrary        | Derive `Arbitrary` implementations.                            | No                 |
//! | serde            | Derive `serdes` `Serialize` and `Deserialize` implementations. | No                 |
//! | tokio            | Provide `tokio` support.                                       | No                 |
//!
//! When using "arbitrary", all types defined in imap-codec implement the [Arbitrary](https://docs.rs/arbitrary/1.1.0/arbitrary/trait.Arbitrary.html)
//! trait to ease testing.
//! When the "serde" feature is used, all types implement [Serde](https://serde.rs/)'s [Serialize](https://docs.serde.rs/serde/trait.Serialize.html) and
//! [Deserialize](https://docs.serde.rs/serde/trait.Deserialize.html) traits.
//! The "tokio_util_compat" feature unlocks an implementation of [tokio_util::codec](https://docs.rs/tokio-util/latest/tokio_util/codec/index.html).
//! See the [tokio client] and [tokio server] demos.
//!
//! [tokio client]: https://github.com/duesee/imap-codec/tree/main/assets/demos/tokio_client
//! [tokio server]: https://github.com/duesee/imap-codec/tree/main/assets/demos/tokio_server
//! [tokio_util::codec]: https://docs.rs/tokio-util/latest/tokio_util/codec/index.html
//! [parse_command]: https://github.com/duesee/imap-codec/blob/main/examples/parse_command.rs
//! [RFC 2088]: https://datatracker.ietf.org/doc/html/rfc2088
//! [RFC 2177]: https://datatracker.ietf.org/doc/html/rfc2177
//! [RFC 2193]: https://datatracker.ietf.org/doc/html/rfc2193
//! [RFC 2221]: https://datatracker.ietf.org/doc/html/rfc2221
//! [RFC 3501]: https://datatracker.ietf.org/doc/html/rfc3501
//! [RFC 3691]: https://datatracker.ietf.org/doc/html/rfc3691
//! [RFC 4959]: https://datatracker.ietf.org/doc/html/rfc4959
//! [RFC 4978]: https://datatracker.ietf.org/doc/html/rfc4978
//! [RFC 5161]: https://datatracker.ietf.org/doc/html/rfc5161
//! [RFC 6851]: https://datatracker.ietf.org/doc/html/rfc6851
//! [RFC 7162]: https://datatracker.ietf.org/doc/html/rfc7162
//! [RFC 7888]: https://datatracker.ietf.org/doc/html/rfc7888
//! [RFC 9208]: https://datatracker.ietf.org/doc/html/rfc9208

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod auth;
pub mod body;
pub mod codec;
pub mod command;
pub mod core;
pub mod datetime;
pub mod envelope;
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
pub mod extensions;
pub mod fetch;
pub mod flag;
pub mod mailbox;
pub mod response;
pub mod search;
pub mod section;
pub mod sequence;
pub mod status;
#[cfg(test)]
mod testing;
#[cfg(any(feature = "tokio"))]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub mod tokio;
pub use imap_types::{secret, state, utils};
