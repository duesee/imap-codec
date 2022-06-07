//! # Features
//!
//! This crate uses the following features to enable IMAP extensions:
//!
//! |Feature     |Description                                                                |Enabled by default |
//! |------------|---------------------------------------------------------------------------|-------------------|
//! |starttls    |See [STARTTLS](https://datatracker.ietf.org/doc/html/rfc3501#section-6.2.1)|No                 |
//! |ext_idle    |See [IDLE](https://datatracker.ietf.org/doc/html/rfc2177)                  |No (but may change)|
//! |ext_enable  |See [ENABLE](https://datatracker.ietf.org/doc/html/rfc5161)                |No (but may change)|
//! |ext_compress|See [COMPRESS](https://datatracker.ietf.org/doc/html/rfc4978)              |No (but may change)|
//!
//! Features prefixed with "ext_" are IMAP extensions and often require a more elaborate message flow.
//! STARTTLS is not considered an extension but feature-gated because it [should be avoided](https://nostarttls.secvuln.info/).
//! You should always use IMAPS, i.e., IMAP-over-TLS on port 993, instead of STARTTLS.
//!
//! Furthermore, imap-codec uses the following features to facilitate interoperability:
//!
//! |Feature     |Description                     |Enabled by default|
//! |------------|--------------------------------|------------------|
//! |arbitrary   |`derive(Arbitrary)`             |No                |
//! |nom         |`pub use internal;`             |No                |
//! |serde       |`derive(Serialize, Deserialize)`|No                |
//!
//! When using "arbitrary", all types defined in imap-codec implement the [Arbitrary](https://docs.rs/arbitrary/1.1.0/arbitrary/trait.Arbitrary.html)
//! trait to ease testing. Although [nom](https://docs.rs/nom/latest/nom/) is always used for parsing, imap-codec tries to hide nom from the public API.
//! Should you want to reuse a parser from imap-codec, use the "nom" feature to export all parsers. When the "serde" feature is used, all types implement
//! [Serde](https://serde.rs/)'s [Serialize](https://docs.serde.rs/serde/trait.Serialize.html) and [Deserialize](https://docs.serde.rs/serde/trait.Deserialize.html) traits.

#![deny(missing_debug_implementations)]

#[cfg(any(feature = "ext_idle", feature = "ext_enable", feature = "ext_compress"))]
pub mod extensions;
pub mod rfc3501;
pub mod utils;

/// Raw nom parsers for the formal syntax of IMAP ([RFC3501](https://datatracker.ietf.org/doc/html/rfc3501#section-9)) and IMAP extensions.
#[cfg(feature = "nom")]
pub mod internal;

pub use imap_types;
pub use imap_types as types;
/// This module is only available when the feature "nom" was specified.
#[cfg(feature = "nom")]
pub use nom;
pub use rfc3501::*;
