//! The IMAP NAMESPACE Extension
//!
//! This extends ...
//!
//! * [`Capability`](crate::response::Capability) with a new variant:
//!
//!     - [`Capability::Namespace`](crate::response::Capability::Namespace)
//!
//! * [`CommandBody`](crate::command::CommandBody) with a new variant:
//!
//!     - [`CommandBody::Namespace`](crate::command::CommandBody::Namespace)
//!
//! * [`Data`] with a new variant:
//!
//!     - [`Data::Namespace`]

#[cfg(feature = "ext_namespace")]
use crate::core::{AString, QuotedChar};
use crate::{command::CommandBody, extensions::namespace::error::NamespaceError, response::Data};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use bounded_static_derive::ToStatic;

impl<'a> CommandBody<'a> {
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it by sending the NAMESPACE capability.
    /// </div>
    pub fn namespace() -> Self {
        CommandBody::Namespace
    }
}

impl<'a> Data<'a> {
    pub fn namespace<P, O, S>(
        personal: P,
        other: O,
        shared: S,
    ) -> Result<Self, NamespaceError<P::Error, O::Error, S::Error>>
    where
        P: TryInto<Vec<NamespaceDescription<'a>>>,
        O: TryInto<Vec<NamespaceDescription<'a>>>,
        S: TryInto<Vec<NamespaceDescription<'a>>>,
    {
        Ok(Self::Namespace {
            personal: personal.try_into().map_err(NamespaceError::Personal)?,
            other: other.try_into().map_err(NamespaceError::Other)?,
            shared: shared.try_into().map_err(NamespaceError::Shared)?,
        })
    }
}

/// A type that holds a prefix and a hierarchy delimiter.
/// Used in the response of the NAMESPACE command.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct NamespaceDescription<'a> {
    pub prefix: AString<'a>,
    pub delimiter: Option<QuotedChar>,
}

impl<'a> NamespaceDescription<'a> {
    pub fn new(prefix: AString<'a>, delimiter: Option<QuotedChar>) -> Self {
        Self { prefix, delimiter }
    }
}

/// Error-related types.
pub mod error {
    use thiserror::Error;

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum NamespaceError<P, O, S> {
        #[error("Invalid personal namespace: {0}")]
        Personal(P),
        #[error("Invalid other namespace: {0}")]
        Other(O),
        #[error("Invalid shared namespace: {0}")]
        Shared(S),
    }
}
