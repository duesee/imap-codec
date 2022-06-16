#![deny(missing_debug_implementations)]

#[cfg(feature = "arbitrary")]
pub mod arbitrary;
#[cfg(any(feature = "ext_idle", feature = "ext_enable", feature = "ext_compress"))]
pub mod extensions;
pub mod rfc3501;
pub mod utils;

pub use rfc3501::*;

// -- API -----------------------------------------------------------------------------------

pub mod codec;
pub mod state;

// -- Re-exports -----------------------------------------------------------------------------------

#[cfg(feature = "bounded-static")]
pub use bounded_static;
