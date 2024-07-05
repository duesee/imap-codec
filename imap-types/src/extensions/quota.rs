//! The IMAP QUOTA Extension
//!
//! This extends ...
//!
//! * [`Capability`](crate::response::Capability) with new variants:
//!
//!     - [`Capability::Quota`](crate::response::Capability::Quota)
//!     - [`Capability::QuotaRes`](crate::response::Capability::QuotaRes)
//!     - [`Capability::QuotaSet`](crate::response::Capability::QuotaSet)
//!
//! * [`CommandBod`y](crate::command::CommandBody) with new variants:
//!
//!     - [`Command::GetQuota`](crate::command::CommandBody::GetQuota)
//!     - [`Command::GetQuotaRoot`](crate::command::CommandBody::GetQuotaRoot)
//!     - [`Command::SetQuota`](crate::command::CommandBody::SetQuota)
//!
//! * [`Data`] with new variants:
//!
//!     - [`Data::Quota`]
//!     - [`Data::QuotaRoot`]
//!
//! * [`Code`](crate::response::Code) with a new variant:
//!
//!     - [`Code::OverQuota`](crate::response::Code::OverQuota)
//!
//! * [`StatusDataItemName`](crate::status::StatusDataItemName) with new variants:
//!
//!     - [`StatusDataItemName::Deleted`](crate::status::StatusDataItemName::Deleted)
//!     - [`StatusDataItemName::DeletedStorage`](crate::status::StatusDataItemName::DeletedStorage)
//!
//! * [`StatusDataItem`](crate::status::StatusDataItem) with new variants:
//!
//!     - [`StatusDataItem::Deleted`](crate::status::StatusDataItem::Deleted)
//!     - [`StatusDataItem::DeletedStorage`](crate::status::StatusDataItem::DeletedStorage)

use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    command::CommandBody,
    core::{impl_try_from, AString, Atom, Vec1},
    extensions::quota::error::{QuotaError, QuotaRootError, SetQuotaError},
    mailbox::Mailbox,
    response::Data,
};

impl<'a> CommandBody<'a> {
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the QUOTA capability.
    /// </div>
    pub fn get_quota<A>(root: A) -> Result<Self, A::Error>
    where
        A: TryInto<AString<'a>>,
    {
        Ok(CommandBody::GetQuota {
            root: root.try_into()?,
        })
    }

    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the QUOTA capability.
    /// </div>
    pub fn get_quota_root<M>(mailbox: M) -> Result<Self, M::Error>
    where
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::GetQuotaRoot {
            mailbox: mailbox.try_into()?,
        })
    }

    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the QUOTA capability.
    /// </div>
    pub fn set_quota<R, S>(root: R, quotas: S) -> Result<Self, SetQuotaError<R::Error, S::Error>>
    where
        R: TryInto<AString<'a>>,
        S: TryInto<Vec<QuotaSet<'a>>>,
    {
        Ok(CommandBody::SetQuota {
            root: root.try_into().map_err(SetQuotaError::Root)?,
            quotas: quotas.try_into().map_err(SetQuotaError::QuotaSet)?,
        })
    }
}

impl<'a> Data<'a> {
    pub fn quota<R, Q>(root: R, quotas: Q) -> Result<Self, QuotaError<R::Error, Q::Error>>
    where
        R: TryInto<AString<'a>>,
        Q: TryInto<Vec1<QuotaGet<'a>>>,
    {
        Ok(Self::Quota {
            root: root.try_into().map_err(QuotaError::Root)?,
            quotas: quotas.try_into().map_err(QuotaError::Quotas)?,
        })
    }

    pub fn quota_root<M, R>(
        mailbox: M,
        roots: R,
    ) -> Result<Self, QuotaRootError<M::Error, R::Error>>
    where
        M: TryInto<Mailbox<'a>>,
        R: TryInto<Vec<AString<'a>>>,
    {
        Ok(Self::QuotaRoot {
            mailbox: mailbox.try_into().map_err(QuotaRootError::Mailbox)?,
            roots: roots.try_into().map_err(QuotaRootError::Roots)?,
        })
    }
}

/// A resource type for use in IMAP's QUOTA extension.
///
/// Supported resource names MUST be advertised as a capability by prepending the resource name with "QUOTA=RES-".
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum Resource<'a> {
    /// The physical space estimate, in units of 1024 octets, of the mailboxes governed by the quota
    /// root.
    ///
    /// This MAY not be the same as the sum of the RFC822.SIZE of the messages. Some implementations
    /// MAY include metadata sizes for the messages and mailboxes, and other implementations MAY
    /// store messages in such a way that the physical space used is smaller, for example, due to
    /// use of compression. Additional messages might not increase the usage. Clients MUST NOT use
    /// the usage figure for anything other than informational purposes; for example, they MUST NOT
    /// refuse to APPEND a message if the limit less the usage is smaller than the RFC822.SIZE
    /// divided by 1024 octets of the message, but it MAY warn about such condition. The usage
    /// figure may change as a result of performing actions not associated with adding new messages
    /// to the mailbox, such as SEARCH, since this may increase the amount of metadata included in
    /// the calculations.
    ///
    /// When the server supports this resource type, it MUST also support the DELETED-STORAGE status
    /// data item.
    ///
    /// Support for this resource MUST be indicated by the server by advertising the
    /// "QUOTA=RES-STORAGE" capability.
    Storage,

    /// The number of messages stored within the mailboxes governed by the quota root.
    ///
    /// This MUST be an exact number; however, clients MUST NOT assume that a change in the usage
    /// indicates a change in the number of messages available, since the quota root may include
    /// mailboxes the client has no access to.
    ///
    /// When the server supports this resource type, it MUST also support the DELETED status data
    /// item.
    ///
    /// Support for this resource MUST be indicated by the server by advertising the
    /// "QUOTA=RES-MESSAGE" capability.
    Message,

    /// The number of mailboxes governed by the quota root.
    ///
    /// This MUST be an exact number; however, clients MUST NOT assume that a change in the usage
    /// indicates a change in the number of mailboxes, since the quota root may include mailboxes
    /// the client has no access to.
    ///
    /// Support for this resource MUST be indicated by the server by advertising the
    /// "QUOTA=RES-MAILBOX" capability.
    Mailbox,

    /// The maximum size of all annotations \[RFC5257\], in units of 1024 octets, associated with all
    /// messages in the mailboxes governed by the quota root.
    ///
    /// Support for this resource MUST be indicated by the server by advertising the
    /// "QUOTA=RES-ANNOTATION-STORAGE" capability.
    AnnotationStorage,

    /// An (unknown) resource.
    Other(ResourceOther<'a>),
}

/// An (unknown) resource.
///
/// It's guaranteed that this type can't represent any resource from [`Resource`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct ResourceOther<'a>(Atom<'a>);

impl_try_from!(Atom<'a>, 'a, &'a [u8], Resource<'a>);
impl_try_from!(Atom<'a>, 'a, Vec<u8>, Resource<'a>);
impl_try_from!(Atom<'a>, 'a, &'a str, Resource<'a>);
impl_try_from!(Atom<'a>, 'a, String, Resource<'a>);
impl_try_from!(Atom<'a>, 'a, Cow<'a, str>, Resource<'a>);

impl<'a> From<Atom<'a>> for Resource<'a> {
    fn from(atom: Atom<'a>) -> Self {
        match atom.inner().to_ascii_uppercase().as_ref() {
            "STORAGE" => Resource::Storage,
            "MESSAGE" => Resource::Message,
            "MAILBOX" => Resource::Mailbox,
            "ANNOTATION-STORAGE" => Resource::AnnotationStorage,
            _ => Resource::Other(ResourceOther(atom)),
        }
    }
}

impl<'a> Display for Resource<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Storage => "STORAGE",
            Self::Message => "MESSAGE",
            Self::Mailbox => "MAILBOX",
            Self::AnnotationStorage => "ANNOTATION-STORAGE",
            Self::Other(other) => other.0.as_ref(),
        })
    }
}

/// A type that holds a resource name, usage, and limit.
/// Used in the response of the GETQUOTA command.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct QuotaGet<'a> {
    pub resource: Resource<'a>,
    pub usage: u64,
    pub limit: u64,
}

impl<'a> QuotaGet<'a> {
    pub fn new(resource: Resource<'a>, usage: u64, limit: u64) -> Self {
        Self {
            resource,
            usage,
            limit,
        }
    }
}

/// A type that holds a resource name and limit.
/// Used in the SETQUOTA command.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(ToStatic, Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuotaSet<'a> {
    pub resource: Resource<'a>,
    pub limit: u64,
}

impl<'a> QuotaSet<'a> {
    pub fn new(resource: Resource<'a>, limit: u64) -> Self {
        Self { resource, limit }
    }
}

/// Error-related types.
pub mod error {
    use thiserror::Error;

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum QuotaError<R, Q> {
        #[error("Invalid root: {0}")]
        Root(R),
        #[error("Invalid quotas: {0}")]
        Quotas(Q),
    }

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum QuotaRootError<M, R> {
        #[error("Invalid mailbox: {0}")]
        Mailbox(M),
        #[error("Invalid roots: {0}")]
        Roots(R),
    }

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum SetQuotaError<R, S> {
        #[error("Invalid root: {0}")]
        Root(R),
        #[error("Invalid quota set: {0}")]
        QuotaSet(S),
    }
}
