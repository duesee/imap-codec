//! # IMAP Protocol Library
//!
//! imap-codec provides complete and detailed parsing and construction of [IMAP4rev1](https://tools.ietf.org/html/rfc3501) commands and responses.
//! It is based on [imap-types](imap_types) and extends it with parsing support via [nom](nom).
//!
//! ## Parsing
//!
//! Parsing is implemented through the [Decode](crate::codec::Decode) trait.
//! The main entry points for parsing are
//! [Greeting::decode(...)](response::Greeting#method.decode) (to parse the first message from a server)),
//! [Command::decode(...)](command::Command#method.decode) (to parse commands from a client), and
//! [Response::decode(...)](response::Response#method.decode) (to parse responses or results from a server).
//! Note, however, that certain message flows require other parsers as well.
//! Every parser takes an input (`&[u8]`) and produces a remainder and a parsed value.
//!
//! ## Serialization
//!
//! Serialization is implemented via the [Encode](crate::codec::Encode) trait.
//! See the [imap-types](imap_types) documentation for the module layout and how to construct messages.
//!
//! ## Example
//!
//! ```rust
//! use imap_codec::{
//!     codec::{Decode, Encode},
//!     command::Command,
//! };
//!
//! let input = b"ABCD UID FETCH 1,2:* (BODY.PEEK[1.2.3.4.MIME]<42.1337>)\r\n";
//!
//! let (_remainder, parsed) = Command::decode(input).unwrap();
//! println!("// Parsed:");
//! println!("{:#?}", parsed);
//!
//! let serialized = parsed.encode_detached().unwrap();
//!
//! let serialized = String::from_utf8(serialized).unwrap(); // Not every IMAP message is valid UTF-8.
//! println!("// Serialized:"); // We just ignore that, so that we can print the message.
//! println!("// {}", serialized);
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
//! |ext_quota            |IMAP QUOTA Extension ([RFC 9208])                           |No (but may change)|
//! |ext_sasl_ir          |IMAP Extension for SASL Initial Client Response ([RFC 4959])|No (but may change)|
//! |starttls             |IMAP4rev1 ([RFC 3501]; section 6.2.1)                       |No                 |
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
//! See the
//! [tokio client](https://github.com/duesee/imap-codec/tree/main/assets/demos/tokio_client) and
//! [tokio server](https://github.com/duesee/imap-codec/tree/main/assets/demos/tokio_server) demos.
//!
//! [RFC 2088]: https://datatracker.ietf.org/doc/html/rfc2088
//! [RFC 2177]: https://datatracker.ietf.org/doc/html/rfc2177
//! [RFC 2193]: https://datatracker.ietf.org/doc/html/rfc2193
//! [RFC 2221]: https://datatracker.ietf.org/doc/html/rfc2221
//! [RFC 3501]: https://datatracker.ietf.org/doc/html/rfc3501
//! [RFC 4959]: https://datatracker.ietf.org/doc/html/rfc4959
//! [RFC 4978]: https://datatracker.ietf.org/doc/html/rfc4978
//! [RFC 5161]: https://datatracker.ietf.org/doc/html/rfc5161
//! [RFC 7888]: https://datatracker.ietf.org/doc/html/rfc7888
//! [RFC 9208]: https://datatracker.ietf.org/doc/html/rfc9208

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

pub mod codec;
#[cfg(any(
    feature = "ext_idle",
    feature = "ext_enable",
    feature = "ext_compress",
    feature = "ext_quota"
))]
mod extensions;
mod imap4rev1;
pub mod utils;

pub use imap_types::{command, core, message, response, security, state};

// ----------- Compatibility modules -----------

#[cfg(any(feature = "tokio"))]
pub mod tokio;

// ----------- Re-exports -----------

pub use imap_types;
pub use nom;
#[cfg(any(feature = "tokio"))]
pub use tokio_util;
