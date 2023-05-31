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
//! ## More complex messages.
//!
//! ### Example
//!
//! The following example is a server fetch response containing the size and MIME structure of message 42.
//!
//! ```
//! use std::{borrow::Cow, num::NonZeroU32};
//!
//! use imap_types::{
//!     body::{BasicFields, Body, BodyStructure, SinglePartExtensionData, SpecificFields},
//!     core::{IString, NString, NonEmptyVec},
//!     fetch::FetchAttributeValue,
//!     response::{Data, Response},
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
//!                 extension_data: None,
//!             }),
//!         ])
//!         .unwrap(),
//!     };
//!
//!     Response::Data(data)
//! };
//! ```
//!
//! # A Note on Types
//!
//! Due to the correctness guarantees, this library uses multiple "string types" like `Atom`, `Tag`, `NString`, and `IString`. See the [core](core) module.

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

#[cfg(feature = "arbitrary")]
mod arbitrary;
pub mod auth;
pub mod body;
pub mod command;
pub mod core;
pub mod datetime;
pub mod envelope;
#[cfg(any(
    feature = "ext_compress",
    feature = "ext_enable",
    feature = "ext_idle",
    feature = "ext_literal",
    feature = "ext_move",
    feature = "ext_quota",
    feature = "ext_unselect",
))]
pub mod extensions;
pub mod fetch;
pub mod flag;
pub mod mailbox;
pub mod response;
pub mod search;
pub mod secret;
pub mod section;
pub mod sequence;
pub mod state;
pub mod status;
pub mod utils;
