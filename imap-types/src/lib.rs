//! # Misuse-resistant IMAP types
//!
//! The most prominent types in imap-types are [`Greeting`](response::Greeting), [`Command`](command::Command), and [`Response`](response::Response), and we use the term "message" to refer to either of them.
//! Messages can be created in different ways.
//! However, what all ways have in common is, that the API does not allow the creation of invalid ones.
//!
//! For example, all commands in IMAP are prefixed with a "tag".
//! Although IMAP's tags are just strings, they have additional rules, such as that no whitespace is allowed.
//! Thus, imap-types encapsulate them in [`Tag`](core::Tag) struct to ensure that invalid ones can't be created.
//!
//! ## Understanding and using the core types
//!
//! Similar to [`Tag`](core::Tag)s, there are more "core types" (or "string types"), such as, [`Atom`](core::Atom), [`Quoted`](core::Quoted), or [`Literal`](core::Literal).
//! Besides being used for correctness, these types play a crucial role in IMAP because they determine the IMAP protocol flow.
//! Sending a password as a literal requires a different protocol flow than sending the password as an atom or a quoted string.
//! So, even though imap-types can choose the most efficient representation for a datum automatically, it's good to become familiar with the [`core`] module at some point to master the IMAP protocol.
//!
//! ## Construction of messages
//!
//! imap-types relies a lot on the standard conversion traits, i.e., [`From`], [`TryFrom`], [`Into`], and [`TryInto`].
//! Make good use of them.
//! More convenient constructors are available for types that are more cumbersome to create.
//!
//! Note: When you are *sure* that the thing you want to create is valid, you can unlock various `unvalidated(...)` functions through the `unvalidated` feature.
//! This allows us to bypass certain checks in release builds.
//!
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
//! // (Note: `Command::new()` returns `Err(...)` when the tag is invalid.)
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
//! ## More complex messages
//!
//! ### Example
//!
//! The following example is a server fetch response containing the size and MIME structure of a message with the sequence number (or UID) 42.
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
//!         seq: NonZeroU32::new(42).unwrap(),
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
//!                         r#type: IString::try_from("text").unwrap(),
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

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]

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
#[cfg_attr(docsrs, doc(cfg(feature = "ext_*")))]
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
