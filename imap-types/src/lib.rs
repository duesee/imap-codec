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
//!     core::{IString, NString, Vec1},
//!     fetch::MessageDataItem,
//!     response::{Data, Response},
//! };
//!
//! let fetch = {
//!     let data = Data::Fetch {
//!         seq: NonZeroU32::new(42).unwrap(),
//!         items: Vec1::try_from(vec![
//!             MessageDataItem::Rfc822Size(1337),
//!             MessageDataItem::Body(BodyStructure::Single {
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
//!
//! # Supported IMAP extensions
//!
//! | Description                                                                                             |
//! |---------------------------------------------------------------------------------------------------------|
//! | IMAP4 non-synchronizing literals ([RFC 2088], [RFC 7888])                                               |
//! | Internet Message Access Protocol (IMAP) - MOVE Extension ([RFC 6851])                                   |
//! | Internet Message Access Protocol (IMAP) UNSELECT command ([RFC 3691])                                   |
//! | IMAP Extension for Simple Authentication and Security Layer (SASL) Initial Client Response ([RFC 4959]) |
//! | The IMAP COMPRESS Extension ([RFC 4978])                                                                |
//! | The IMAP ENABLE Extension ([RFC 5161])                                                                  |
//! | IMAP4 IDLE command ([RFC 2177])                                                                         |
//! | IMAP QUOTA Extension ([RFC 9208])                                                                       |
//! | IMAP4 UIDPLUS extension ([RFC 2359], [RFC 4315])                                                        |
//! | IMAP4 Binary Content Extension ([RFC 3516])                                                             |
//! | Internet Message Access Protocol - SORT and THREAD Extensions ([RFC 5256], [RFC 5957])                  |
//!
//! # Features
//!
//! This crate uses the following features to enable experimental IMAP extensions:
//!
//! | Feature               | Description                                                                                                                  | Status     |
//! |-----------------------|------------------------------------------------------------------------------------------------------------------------------|------------|
//! | starttls              | IMAP4rev1 ([RFC 3501]; section 6.2.1)                                                                                        |            |
//! | ext_condstore_qresync | IMAP Extensions: Quick Flag Changes Resynchronization (CONDSTORE) and Quick Mailbox Resynchronization (QRESYNC) ([RFC 7162]) | Unfinished |
//! | ext_id                | IMAP4 ID extension ([RFC 2971])                                                                                              | Unfinished |
//! | ext_login_referrals   | IMAP4 Login Referrals ([RFC 2221])                                                                                           | Unfinished |
//! | ext_mailbox_referrals | IMAP4 Mailbox Referrals ([RFC 2193])                                                                                         | Unfinished |
//! | ext_metadata          | The IMAP METADATA Extension ([RFC 5464])                                                                                     | Unfinished |
//!
//! STARTTLS is not an IMAP extension but feature-gated because it [should be avoided](https://nostarttls.secvuln.info/).
//! For better performance and security, use "implicit TLS", i.e., IMAP-over-TLS on port 993, and don't use STARTTLS at all.
//!
//! Furthermore, imap-types uses the following features:
//!
//! | Feature          | Description                                                   | Enabled by default |
//! |------------------|---------------------------------------------------------------|--------------------|
//! | arbitrary        | Derive `Arbitrary` implementations                            | No                 |
//! | serde            | Derive `serde`s `Serialize` and `Deserialize` implementations | No                 |
//! | tag_generator    | Provide a generator for randomized `Tag`s                     | No                 |
//!
//! When using `arbitrary`, all types defined in imap-types implement the [Arbitrary] trait to ease testing.
//! This is used, for example, to generate instances during fuzz-testing.
//! (See, e.g., `imap-types/fuzz/fuzz_targets/to_static.rs`)
//! When the `serde` feature is used, all types implement [Serde](https://serde.rs/)'s [Serialize](https://docs.serde.rs/serde/trait.Serialize.html) and
//! [Deserialize](https://docs.serde.rs/serde/trait.Deserialize.html) traits. (Try running `cargo run --example serde_json`.)
//! Using `tag_generator` unlocks a `TagGenerator` to generate random tags.
//! This may help to prevent attacks that depend on the knowledge of the next tag.
//!
//! [Arbitrary]: https://docs.rs/arbitrary/1.0.1/arbitrary/trait.Arbitrary.html
//! [parse_command]: https://github.com/duesee/imap-codec/blob/main/imap-codec/examples/parse_command.rs
//! [RFC 2088]: https://datatracker.ietf.org/doc/html/rfc2088
//! [RFC 2177]: https://datatracker.ietf.org/doc/html/rfc2177
//! [RFC 2193]: https://datatracker.ietf.org/doc/html/rfc2193
//! [RFC 2221]: https://datatracker.ietf.org/doc/html/rfc2221
//! [RFC 2359]: https://datatracker.ietf.org/doc/html/rfc2359
//! [RFC 2971]: https://datatracker.ietf.org/doc/html/rfc2971
//! [RFC 3501]: https://datatracker.ietf.org/doc/html/rfc3501
//! [RFC 3516]: https://datatracker.ietf.org/doc/html/rfc3516
//! [RFC 3691]: https://datatracker.ietf.org/doc/html/rfc3691
//! [RFC 4315]: https://datatracker.ietf.org/doc/html/rfc4315
//! [RFC 4959]: https://datatracker.ietf.org/doc/html/rfc4959
//! [RFC 4978]: https://datatracker.ietf.org/doc/html/rfc4978
//! [RFC 5161]: https://datatracker.ietf.org/doc/html/rfc5161
//! [RFC 5256]: https://datatracker.ietf.org/doc/html/rfc5256
//! [RFC 5464]: https://datatracker.ietf.org/doc/html/rfc5464
//! [RFC 5957]: https://datatracker.ietf.org/doc/html/rfc5957
//! [RFC 6851]: https://datatracker.ietf.org/doc/html/rfc6851
//! [RFC 7162]: https://datatracker.ietf.org/doc/html/rfc7162
//! [RFC 7888]: https://datatracker.ietf.org/doc/html/rfc7888
//! [RFC 9208]: https://datatracker.ietf.org/doc/html/rfc9208

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]
// TODO(#313)
// #![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use bounded_static::{IntoBoundedStatic, ToBoundedStatic};

// Test examples from imap-types' README.
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

#[cfg(feature = "arbitrary")]
mod arbitrary;
pub mod auth;
pub mod body;
pub mod command;
pub mod core;
pub mod datetime;
pub mod envelope;
pub mod error;
pub mod extensions;
pub mod fetch;
pub mod flag;
pub mod mailbox;
pub mod response;
pub mod search;
pub mod secret;
pub mod sequence;
pub mod state;
pub mod status;
pub mod utils;

/// Create owned variant of object.
///
/// Useful, e.g., if you want to pass the object to another thread or executor.
pub trait ToStatic {
    type Static: 'static;

    fn to_static(&self) -> Self::Static;
}

impl<T> ToStatic for T
where
    T: ToBoundedStatic,
{
    type Static = <T as ToBoundedStatic>::Static;

    fn to_static(&self) -> Self::Static {
        ToBoundedStatic::to_static(self)
    }
}

/// Create owned variant of object (consuming it).
///
/// Useful, e.g., if you want to pass the object to another thread or executor.
pub trait IntoStatic {
    type Static: 'static;

    fn into_static(self) -> Self::Static;
}

impl<T> IntoStatic for T
where
    T: IntoBoundedStatic,
{
    type Static = <T as IntoBoundedStatic>::Static;

    fn into_static(self) -> Self::Static {
        IntoBoundedStatic::into_static(self)
    }
}
