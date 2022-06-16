#![deny(missing_debug_implementations)]

#[cfg(feature = "arbitrary")]
mod arbitrary;
#[cfg(any(feature = "ext_idle", feature = "ext_enable", feature = "ext_compress"))]
pub mod extensions;
mod rfc3501;

pub use rfc3501::*;

// -- API -----------------------------------------------------------------------------------

pub mod codec;
pub mod state;
pub mod utils;

// -- Re-exports -----------------------------------------------------------------------------------

#[cfg(feature = "bounded-static")]
pub use bounded_static;
