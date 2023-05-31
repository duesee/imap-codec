//! IMAP4 IDLE command
//!
//! This extension adds a new method ...
//!
//! *  [`CommandBody::idle()`](crate::command::CommandBody#method.idle)
//!
//! ... adds a new type ...
//!
//! * [`IdleDone`](crate::extensions::idle::IdleDone)
//!
//! ... and extends ...
//!
//! * [`CommandBody`](crate::command::CommandBody) enum with a new variant [`CommandBody::Idle`](crate::command::CommandBody#variant.Idle), and
//! * [`Capability`](crate::response::Capability) enum with a new variant [`Capability::Idle`](crate::response::Capability#variant.Idle).

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Denotes the continuation data message "DONE\r\n" to end the IDLE command.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdleDone;
