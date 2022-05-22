#![deny(missing_debug_implementations)]

#[cfg(any(feature = "ext_idle", feature = "ext_enable", feature = "ext_compress"))]
pub mod extensions;
pub mod rfc3501;
pub mod utils;

/// Raw nom parsers for the formal syntax of IMAP ([RFC3501](https://datatracker.ietf.org/doc/html/rfc3501#section-9)) and IMAP extensions.
#[cfg(feature = "nomx")]
pub mod internal;

/// This module is only available when the feature "nomx" was specified.
#[cfg(feature = "nomx")]
pub use nom;
pub use rfc3501::*;
