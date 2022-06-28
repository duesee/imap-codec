//! # Misuse-resistant Low-Level Types for the IMAP Protocol
//!
//! The two main types in imap-types are [Command](api::command::Command) and [Response](api::response::Response) and we use the term "message" to refer to either of them.
//!
//! ## Module structure
//!
//! The module structure reflects this terminology:
//! types that are specific to commands are in the [command](api::command) module;
//! types that are specific to responses are in the [response](api::response) module;
//! types that are used in both are in the [message](api::message) module.
//! The [codec](codec) module contains the [Decode](codec::Decode) trait used to serialize messages.
//! The [core] module contains "string types" -- there should be no need to use them directly.
//!
//! ## Simple construction of messages.
//!
//! Messages can be created in different ways.
//! However, what all ways have in common is, that the API does not allow to create invalid ones.
//!
//! For example, all command in IMAP (and many responses) are prefixed with a tag.
//! Although tags are basically strings, they have additional rules, such as, that no whitespace is allowed.
//! Thus, imap-codec encapsulates tags in the [Tag](api::message::Tag) struct and makes sure that no invalid tag can be created.
//! This is why [Result](std::result::Result) is often used in associated functions or methods.
//!
//! Generally, imap-codec relies a lot on the [From](std::convert::From), [TryFrom](std::convert::TryFrom), [Into](std::convert::Into), and [TryInto](std::convert::TryInto) traits.
//! Make good use of them.
//! However, some types are cumbersome to create.
//! Thus, there are helper methods, such as, [AuthMechanism::other(...)](api::message::AuthMechanism::other).
//!
//! ### Example
//!
//! ```
//! use std::convert::TryFrom;
//!
//! use imap_types::{
//!     command::{Command, CommandBody},
//!     core::Tag,
//! };
//!
//! // # Variant 1
//! // Create a `Command` with `tag` "A123" and `body` "NOOP".
//! // (Note: `Command::new()` returns `Result::Err(...)` when the tag is invalid.)
//! let cmd = Command::new("A123", CommandBody::Noop).unwrap();
//!
//! // # Variant 2
//! // Create a `CommandBody` first and finalize it into
//! // a `Command` by attaching a tag later.
//! let cmd = CommandBody::Noop.tag("A123").unwrap();
//!
//! // # Variant 3
//! // Create a `Command` directly.
//! let cmd = Command {
//!     tag: Tag::try_from("A123").unwrap(),
//!     body: CommandBody::Noop,
//! };
//! ```
//!
//! ## Serialization of messages.
//!
//! All messages implement the `Encode` trait.
//! You can `use imap_types::Encode` and call the `.encode(...)` method to serialize a message into a writer.
//! Note that IMAP traces are not guaranteed to be UTF-8. Thus, be careful when using things like `std::str::from_utf8(...).unwrap()`.
//! It should be generally better to not think about IMAP as being UTF-8.
//! This is also why `Display` is not implemented.
//! All types implement `Debug`, though.
//!
//! ### Example
//!
//! ```
//! use imap_types::{
//!     codec::Encode,
//!     command::{Command, CommandBody},
//! };
//!
//! // Create some command.
//! let cmd = Command::new("A123", CommandBody::login("alice", "password").unwrap()).unwrap();
//!
//! // Create something to encode the `cmd` into.
//! let mut out = Vec::new();
//!
//! // Encode the `cmd` into `out`.
//! cmd.encode(&mut out);
//!
//! // Print the command.
//! // (Note that IMAP traces are not guaranteed to be valid UTF-8.)
//! println!("{}", std::str::from_utf8(&out).unwrap());
//! ```
//!
//! ## More complex messages.
//!
//! ...
//!
//! ### Example
//!
//! ```
//! use std::convert::TryFrom;
//!
//! use imap_types::{
//!     command::{
//!         Command,
//!         CommandBody,
//!         SearchKey,
//!     },
//!     core::Charset,
//! };
//!
//! let search = {
//!     let mailbox = Charset::try_from("Archive").unwrap();
//!
//!     CommandBody::search(Some(mailbox), SearchKey::All, false)
//!         .tag("A123")
//!         .unwrap()
//! };
//!
//! println!("{:?}", search);
//! ```

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

// TODO: Temporarily. Use this in tests, examples, etc. and see if it feels right.
pub mod api {
    pub use crate::codec;

    pub mod core {
        pub use crate::rfc3501::core::{
            AString, Atom, AtomExt, IString, Literal, NString, NonEmptyVec, Quoted,
        };
    }

    pub mod message {
        #[cfg(feature = "ext_compress")]
        pub use crate::extensions::rfc4987::CompressionAlgorithm;
        #[cfg(feature = "ext_enable")]
        pub use crate::extensions::rfc5161::{CapabilityEnable, Utf8Kind};
        pub use crate::rfc3501::{
            core::{Charset, Tag},
            datetime::{MyDateTime, MyNaiveDate},
            flag::{Flag, FlagNameAttribute},
            mailbox::{Mailbox, MailboxOther},
            section::{Part, PartSpecifier, Section},
            AuthMechanism, AuthMechanismOther,
        };
    }

    pub mod command {
        pub use crate::rfc3501::{
            command::{Command, CommandBody},
            mailbox::{ListCharString, ListMailbox},
            sequence::{SeqNo, Sequence, SequenceSet, Strategy},
        };

        pub mod status {
            pub use crate::rfc3501::status_attributes::StatusAttribute;
        }

        pub mod search {
            pub use crate::rfc3501::command::SearchKey;
        }

        pub mod fetch {
            pub use crate::rfc3501::fetch_attributes::{
                FetchAttribute, Macro, MacroOrFetchAttributes,
            };
        }

        pub mod store {
            pub use crate::rfc3501::flag::{StoreResponse, StoreType};
        }
    }

    pub mod response {
        pub use crate::rfc3501::{
            core::Text,
            response::{Code, Continue, Data, Greeting, GreetingKind, Response, Status},
        };

        pub mod data {
            pub use crate::rfc3501::{
                address::Address,
                body::{
                    BasicFields, Body, BodyStructure, MultiPartExtensionData,
                    SinglePartExtensionData, SpecificFields,
                },
                core::QuotedChar,
                envelope::Envelope,
                fetch_attributes::FetchAttributeValue,
                flag::FlagNameAttribute,
                response::{Capability, CapabilityOther},
                status_attributes::StatusAttributeValue,
            };
        }
    }
}

// -- Re-exports -----------------------------------------------------------------------------------

#[cfg(feature = "bounded-static")]
pub use bounded_static;
