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
