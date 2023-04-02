//! # Misuse-resistant Types for the IMAP Protocol
//!
//! The main types in imap-types are [Greeting](response::Greeting), [Command](command::Command), and [Response](response::Response), and we use the term "message" to refer to either of them.
//!
//! ## Module structure
//!
//! The module structure reflects this terminology:
//! types that are specific to commands are in the [command](command) module;
//! types that are specific to responses (including the greeting) are in the [response](response) module;
//! types used in both are in the [message](message) module.
//! The [codec](codec) module contains the [Decode](codec::Decode) trait used to serialize messages.
//! The [core] module contains "string types" -- there should be no need to use them directly.
//!
//! ## Simple construction of messages.
//!
//! Messages can be created in different ways.
//! However, what all ways have in common is, that the API does not allow the creation of invalid ones.
//!
//! For example, all commands in IMAP (and many responses) are prefixed with a "tag".
//! Although IMAP tags are just strings, they have additional rules, such as that no whitespace is allowed.
//! Thus, imap-codec encapsulates tags in the [Tag](message::Tag) struct and ensures no invalid tag can be created.
//! This is why [Result](std::result::Result) is often used in associated functions or methods.
//!
//! Generally, imap-codec relies a lot on the [From](std::convert::From), [TryFrom](std::convert::TryFrom), [Into](std::convert::Into), and [TryInto](std::convert::TryInto) traits.
//! Make good use of them.
//! For types that are more cumbersome to create, there are helper methods available.
//!
//! ### Example
//!
//! ```
//! use std::convert::TryFrom;
//!
//! use imap_types::{
//!     command::{Command, CommandBody},
//!     message::Tag,
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
//! You can `use imap_types::Encode` and call the `.encode(...)` (or `.encode_detached(...)`) method to serialize a message (into a writer).
//! Note that IMAP traces are not guaranteed to be UTF-8. Thus, be careful when using things like `std::str::from_utf8(...).unwrap()`.
//! It should generally be better not to think about IMAP as being UTF-8.
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
//! // Encode the `cmd` into `out`.
//! let out = cmd.encode_detached().unwrap();
//!
//! // Print the command.
//! // (Note that IMAP traces are not guaranteed to be valid UTF-8.)
//! println!("{}", std::str::from_utf8(&out).unwrap());
//! ```
//!
//! ## More complex messages.
//!
//! ### Example
//!
//! The following example is a server fetch response containing the size and MIME structure of message 42.
//!
//! ```
//! use std::{borrow::Cow, convert::TryFrom, num::NonZeroU32};
//!
//! use imap_types::{
//!     codec::Encode,
//!     core::{IString, NString, NonEmptyVec},
//!     response::{
//!         data::{
//!             BasicFields, Body, BodyStructure, FetchAttributeValue, SinglePartExtensionData,
//!             SpecificFields,
//!         },
//!         Data, Response,
//!     },
//! };
//!
//! let fetch = {
//!     let data = Data::Fetch {
//!         seq_or_uid: NonZeroU32::new(42).unwrap(),
//!         attributes: NonEmptyVec::try_from(vec![
//!             FetchAttributeValue::Rfc822Size(1337),
//!             FetchAttributeValue::Body(BodyStructure::Single {
//!                 body: Body {
//!                     basic: BasicFields {
//!                         parameter_list: vec![],
//!                         id: NString(None),
//!                         description: NString(Some(
//!                             IString::try_from("Important message.").unwrap(),
//!                         )),
//!                         content_transfer_encoding: IString::try_from("base64").unwrap(),
//!                         size: 512,
//!                     },
//!                     specific: SpecificFields::Basic {
//!                         type_: IString::try_from("text").unwrap(),
//!                         subtype: IString::try_from("html").unwrap(),
//!                     },
//!                 },
//!                 extension: Some(SinglePartExtensionData {
//!                     md5: NString(None),
//!                     disposition: None,
//!                     language: None,
//!                     location: None,
//!                     extension: Cow::Borrowed(b""),
//!                 }),
//!             }),
//!         ])
//!         .unwrap(),
//!     };
//!
//!     Response::Data(data)
//! };
//!
//! let mut out = std::io::stdout();
//! fetch.encode(&mut out).unwrap();
//! ```
//!
//! # A Note on Types
//!
//! Due to the correctness guarantees, this library uses multiple "string types" like `Atom`, `Tag`, `NString`, and `IString`. See the [core](core) module.

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

#[cfg(feature = "arbitrary")]
mod arbitrary;
mod rfc3501;
pub mod security;
mod utils;

// -- API -----------------------------------------------------------------------------------

pub mod codec;
pub mod state;

pub mod core {
    //! # Core Data Types
    //!
    //! This module exposes IMAPs "core data types" (or "string types").
    //! It is loosely based on the IMAP standard.
    //! Some additional types are defined and some might be missing.
    //!
    //! "IMAP4rev1 uses textual commands and responses.
    //! Data in IMAP4rev1 can be in one of several forms: atom, number, string, parenthesized list, or NIL.
    //! Note that a particular data item may take more than one form; for example, a data item defined as using "astring" syntax may be either an atom or a string." ([RFC 3501](https://www.rfc-editor.org/rfc/rfc3501.html))
    //!
    //! ## (Incomplete) Summary
    //!
    //! ```text
    //!        ┌───────┐ ┌─────────────────┐
    //!        │AString│ │     NString     │
    //!        └──┬─┬──┘ │(Option<IString>)│
    //!           │ │    └─────┬───────────┘
    //!           │ └──────┐   │
    //!           │        │   │
    //! ┌────┐ ┌──▼────┐ ┌─▼───▼─┐
    //! │Atom│ │AtomExt│ │IString│
    //! └────┘ └───────┘ └┬─────┬┘
    //!                   │     │
    //!             ┌─────▼─┐ ┌─▼────┐
    //!             │Literal│ │Quoted│
    //!             └───────┘ └──────┘
    //! ```

    pub use crate::rfc3501::core::{
        AString, Atom, AtomExt, IString, Literal, NString, NonEmptyVec, Quoted,
    };
}

pub mod message {
    //! # Types used in commands and responses

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
    //! # Types used in commands

    pub use crate::rfc3501::{
        command::{
            AppendError, AuthenticateData, Command, CommandBody, CopyError, ListError, LoginError,
            RenameError,
        },
        mailbox::{ListCharString, ListMailbox},
        sequence::{SeqOrUid, Sequence, SequenceSet, Strategy},
    };

    pub mod status {
        //! # Types used in STATUS command

        pub use crate::rfc3501::status_attributes::StatusAttribute;
    }

    pub mod search {
        //! # Types used in SEARCH command

        pub use crate::rfc3501::command::SearchKey;
    }

    pub mod fetch {
        //! # Types used in FETCH command

        pub use crate::rfc3501::fetch_attributes::{FetchAttribute, Macro, MacroOrFetchAttributes};
    }

    pub mod store {
        //! # Types used in STORE command
        pub use crate::rfc3501::flag::{StoreResponse, StoreType};
    }

    #[cfg(feature = "ext_quota")]
    pub mod quota {
        pub use crate::extensions::rfc9208::SetQuotaError;
    }

    #[cfg(feature = "ext_idle")]
    pub mod idle {
        pub use crate::extensions::rfc2177::IdleDone;
    }
}

pub mod response {
    //! # Types used in responses

    pub use crate::rfc3501::{
        core::Text,
        response::{Code, CodeOther, Continue, Data, Greeting, GreetingKind, Response, Status},
    };

    pub mod data {
        pub use crate::rfc3501::{
            address::Address,
            body::{
                BasicFields, Body, BodyStructure, MultiPartExtensionData, SinglePartExtensionData,
                SpecificFields,
            },
            core::QuotedChar,
            envelope::Envelope,
            fetch_attributes::FetchAttributeValue,
            flag::FlagNameAttribute,
            response::{Capability, CapabilityOther},
            status_attributes::StatusAttributeValue,
        };
    }

    #[cfg(feature = "ext_quota")]
    pub mod quota {
        pub use crate::extensions::rfc9208::{QuotaError, QuotaRootError};
    }
}

#[cfg(any(
    feature = "ext_idle",
    feature = "ext_enable",
    feature = "ext_compress",
    feature = "ext_quota"
))]
pub mod extensions;

// -- Re-exports -----------------------------------------------------------------------------------

#[cfg(feature = "bounded-static")]
pub use bounded_static;
