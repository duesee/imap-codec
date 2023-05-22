//! # Serialization of messages
//!
//! All messages implement the `Encode` trait.
//! You can `use imap_codec::Encode` and call the `.encode(...)` (or `.encode_detached(...)`) method to serialize a message (into a writer).
//! Note that IMAP traces are not guaranteed to be UTF-8. Thus, be careful when using things like `std::str::from_utf8(...).unwrap()`.
//! It should generally be better not to think about IMAP as being UTF-8.
//! This is also why `Display` is not implemented.
//! All types implement `Debug`, though.
//!
//! ## Example
//!
//! ```
//! use imap_codec::{
//!     codec::Encode,
//!     command::{Command, CommandBody},
//! };
//!
//! // Create some command.
//! let cmd = Command::new("A123", CommandBody::login("alice", "password").unwrap()).unwrap();
//!
//! // Encode the `cmd` into `out`.
//! let out = cmd.encode_detached().unwrap();
//!
//! // Print the command.
//! // (Note that IMAP traces are not guaranteed to be valid UTF-8.)
//! println!("{}", std::str::from_utf8(&out).unwrap());
//! ```

mod decode;
mod encode;

pub use decode::{Decode, DecodeError};
pub use encode::Encode;
