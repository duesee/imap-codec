//! The IMAP NAMESPACE Extension
//!
//! This extends ...
//!
//! * [`Capability`](crate::response::Capability) with a new variant:
//!
//!     - [`Capability::Namespace`](crate::response::Capability::Namespace)
//!
//! * [`CommandBody`] with a new variant:
//!
//!     - [`CommandBody::Namespace`]
//!
//! * [`Data`] with a new variant:
//!
//!     - [`Data::Namespace`]

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    command::CommandBody,
    core::{IString, QuotedChar, Vec1},
    extensions::namespace::error::NamespaceError,
    response::Data,
};

impl<'a> CommandBody<'a> {
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it by sending the NAMESPACE capability.
    /// </div>
    pub fn namespace() -> Self {
        CommandBody::Namespace
    }
}

impl<'a> Data<'a> {
    #[allow(clippy::type_complexity)]
    pub fn namespace<P, O, S>(
        personal: P,
        other: O,
        shared: S,
    ) -> Result<Self, NamespaceError<P::Error, O::Error, S::Error>>
    where
        P: TryInto<Namespaces<'a>>,
        O: TryInto<Namespaces<'a>>,
        S: TryInto<Namespaces<'a>>,
    {
        Ok(Self::Namespace {
            personal: personal.try_into().map_err(NamespaceError::Personal)?,
            other: other.try_into().map_err(NamespaceError::Other)?,
            shared: shared.try_into().map_err(NamespaceError::Shared)?,
        })
    }
}

/// A list of `Namespace` definitions.
///
/// Corresponds to the `Namespace` rule in the ABNF, which is either `NIL`
/// or a parenthesized list of namespace descriptions. An empty `Vec` is treated as `NIL`.
pub type Namespaces<'a> = Vec<Namespace<'a>>;

/// A single namespace's description, containing a prefix, delimiter, and optional extensions.
///
/// Corresponds to the `( string SP (<"> QUOTED_CHAR <"> / nil) *(Namespace_Response_Extension) )`
/// part of the ABNF.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct Namespace<'a> {
    pub prefix: IString<'a>,
    pub delimiter: Option<QuotedChar>,
    /// Optional extension data for this namespace.
    pub extensions: Vec<NamespaceResponseExtension<'a>>,
}

impl<'a> Namespace<'a> {
    pub fn new(prefix: IString<'a>, delimiter: Option<QuotedChar>) -> Self {
        Self {
            prefix,
            delimiter,
            extensions: Vec::new(),
        }
    }
}

/// Extension data for a namespace response.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct NamespaceResponseExtension<'a> {
    pub key: IString<'a>,
    pub values: Vec1<IString<'a>>,
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
