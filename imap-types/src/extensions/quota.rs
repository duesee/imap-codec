//! The IMAP QUOTA Extension
//!
//! This extends ...
//!
//! * [Capability](crate::response::data::Capability) with new variants:
//!
//!     - [Capability::Quota](crate::response::data::Capability::Quota)
//!     - [Capability::QuotaRes](crate::response::data::Capability::QuotaRes)
//!     - [Capability::QuotaSet](crate::response::data::Capability::QuotaSet)
//!
//! * [CommandBody](crate::command::CommandBody) with new variants:
//!
//!     - [Command::GetQuota](crate::command::CommandBody::GetQuota)
//!     - [Command::GetQuotaRoot](crate::command::CommandBody::GetQuotaRoot)
//!     - [Command::SetQuota](crate::command::CommandBody::SetQuota)
//!
//! * [Data](crate::response::Data) with new variants:
//!
//!     - [Data::Quota](crate::response::Data::Quota)
//!     - [Data::QuotaRoot](crate::response::Data::QuotaRoot)
//!
//! * [Code](crate::response::Code) with a new variant:
//!
//!     - [Code::OverQuota](crate::response::Code::OverQuota)
//!
//! * [StatusAttribute](crate::command::status::StatusAttribute) with new variants:
//!
//!     - [StatusAttribute::Deleted](crate::command::status::StatusAttribute::Deleted)
//!     - [StatusAttribute::DeletedStorage](crate::command::status::StatusAttribute::DeletedStorage)
//!
//! * [StatusAttributeValue](crate::response::data::StatusAttributeValue) with new variants:
//!
//!     - [StatusAttributeValue::Deleted](crate::response::data::StatusAttributeValue::Deleted)
//!     - [StatusAttributeValue::DeletedStorage](crate::response::data::StatusAttributeValue::DeletedStorage)

use std::{
    borrow::Cow,
    convert::{TryFrom, TryInto},
    io::Write,
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    codec::Encode,
    command::CommandBody,
    core::{AString, NonEmptyVec},
    imap4rev1::core::{impl_try_from, Atom, AtomError},
    message::Mailbox,
    response::Data,
};

impl<'a> CommandBody<'a> {
    pub fn get_quota<A>(root: A) -> Result<Self, A::Error>
    where
        A: TryInto<AString<'a>>,
    {
        Ok(CommandBody::GetQuota {
            root: root.try_into()?,
        })
    }

    pub fn get_quota_root<M>(mailbox: M) -> Result<Self, M::Error>
    where
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::GetQuotaRoot {
            mailbox: mailbox.try_into()?,
        })
    }

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
        Q: TryInto<NonEmptyVec<QuotaGet<'a>>>,
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
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    /// The maximum size of all annotations [RFC5257], in units of 1024 octets, associated with all
    /// messages in the mailboxes governed by the quota root.
    ///
    /// Support for this resource MUST be indicated by the server by advertising the
    /// "QUOTA=RES-ANNOTATION-STORAGE" capability.
    AnnotationStorage,
    /// Other.
    ///
    /// Note: The value must be semantically different from the supported variants.
    Other(ResourceOther<'a>),
}

impl_try_from!(Atom<'a>, 'a, &'a [u8], Resource<'a>);
impl_try_from!(Atom<'a>, 'a, Vec<u8>, Resource<'a>);
impl_try_from!(Atom<'a>, 'a, &'a str, Resource<'a>);
impl_try_from!(Atom<'a>, 'a, String, Resource<'a>);
impl_try_from!(Atom<'a>, 'a, Cow<'a, str>, Resource<'a>);

impl<'a> From<Atom<'a>> for Resource<'a> {
    fn from(value: Atom<'a>) -> Self {
        match value.inner().to_ascii_lowercase().as_ref() {
            "storage" => Resource::Storage,
            "message" => Resource::Message,
            "mailbox" => Resource::Mailbox,
            "annotation-storage" => Resource::AnnotationStorage,
            _ => Resource::Other(ResourceOther(value)),
        }
    }
}

impl<'a> Encode for Resource<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Resource::Storage => writer.write_all(b"STORAGE"),
            Resource::Message => writer.write_all(b"MESSAGE"),
            Resource::Mailbox => writer.write_all(b"MAILBOX"),
            Resource::AnnotationStorage => writer.write_all(b"ANNOTATION-STORAGE"),
            Resource::Other(atom) => atom.encode(writer),
        }
    }
}

/// A resource type (name) for use in IMAP's QUOTA extension that is not supported by imap-types.
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceOther<'a>(Atom<'a>);

impl<'a> ResourceOther<'a> {
    pub fn verify(value: impl AsRef<[u8]>) -> Result<(), ResourceOtherError> {
        if matches!(
            value.as_ref().to_ascii_lowercase().as_slice(),
            b"storage" | b"message" | b"mailbox" | b"annotation-storage",
        ) {
            return Err(ResourceOtherError::Reserved);
        }

        Ok(())
    }

    #[cfg(feature = "unchecked")]
    pub fn new_unchecked(atom: Atom<'a>) -> Self {
        Self(atom)
    }
}

macro_rules! impl_try_from {
    ($from:ty) => {
        impl<'a> TryFrom<$from> for ResourceOther<'a> {
            type Error = ResourceOtherError;

            fn try_from(value: $from) -> Result<Self, Self::Error> {
                let atom = Atom::try_from(value)?;

                Self::verify(atom.as_ref())?;

                Ok(Self(atom))
            }
        }
    };
}

impl_try_from!(&'a [u8]);
impl_try_from!(Vec<u8>);
impl_try_from!(&'a str);
impl_try_from!(String);

impl<'a> TryFrom<Atom<'a>> for ResourceOther<'a> {
    type Error = ResourceOtherError;

    fn try_from(atom: Atom<'a>) -> Result<Self, Self::Error> {
        Self::verify(atom.as_ref())?;

        Ok(Self(atom))
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResourceOtherError {
    #[error(transparent)]
    Atom(#[from] AtomError),
    #[error("Reserved. Please use one of the typed variants.")]
    Reserved,
}

impl<'a> Encode for ResourceOther<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.0.encode(writer)
    }
}

/// A type that holds a resource name, usage, and limit.
/// Used in the response of the GETQUOTA command.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl<'a> Encode for QuotaGet<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.resource.encode(writer)?;
        write!(writer, " {} {}", self.usage, self.limit)
    }
}

/// A type that holds a resource name and limit.
/// Used in the SETQUOTA command.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuotaSet<'a> {
    pub resource: Resource<'a>,
    pub limit: u64,
}

impl<'a> QuotaSet<'a> {
    pub fn new(resource: Resource<'a>, limit: u64) -> Self {
        Self { resource, limit }
    }
}

impl<'a> Encode for QuotaSet<'a> {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.resource.encode(writer)?;
        write!(writer, " {}", self.limit)
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum QuotaError<R, Q> {
    #[error("Invalid root: {0:?}")]
    Root(R),
    #[error("Invalid quotas: {0:?}")]
    Quotas(Q),
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum QuotaRootError<M, R> {
    #[error("Invalid root: {0:?}")]
    Mailbox(M),
    #[error("Invalid quotas: {0:?}")]
    Roots(R),
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum SetQuotaError<R, S> {
    #[error("Invalid root: {0:?}")]
    Root(R),
    #[error("Invalid quota set: {0:?}")]
    QuotaSet(S),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_output() {
        let commands = vec![
            (CommandBody::get_quota("INBOX").unwrap(), b"GETQUOTA INBOX".as_ref()),
            (CommandBody::get_quota("").unwrap(), b"GETQUOTA \"\""),
            (
                CommandBody::get_quota_root("MAILBOX").unwrap(),
                b"GETQUOTAROOT MAILBOX",
            ),
            (
                CommandBody::set_quota("INBOX", vec![]).unwrap(),
                b"SETQUOTA INBOX ()",
            ),
            (
                CommandBody::set_quota(
                    "INBOX",
                    vec![QuotaSet {
                        resource: Resource::Storage,
                        limit: 256,
                    }],
                )
                .unwrap(),
                b"SETQUOTA INBOX (STORAGE 256)",
            ),
            (
                CommandBody::set_quota(
                    "INBOX",
                    vec![
                        QuotaSet {
                            resource: Resource::Storage,
                            limit: 0,
                        },
                        QuotaSet {
                            resource: Resource::Message,
                            limit: 512,
                        },
                        QuotaSet {
                            resource: Resource::Mailbox,
                            limit: 512,
                        },
                        QuotaSet {
                            resource: Resource::AnnotationStorage,
                            limit: 123,
                        },
                        QuotaSet {
                            resource: Resource::Other(ResourceOther::try_from("Foo").unwrap()),
                            limit: u64::MAX,
                        },
                    ],
                )
                .unwrap(),
                b"SETQUOTA INBOX (STORAGE 0 MESSAGE 512 MAILBOX 512 ANNOTATION-STORAGE 123 Foo 18446744073709551615)",
            ),
        ];

        compare_output(commands)
    }

    #[test]
    fn test_response_output() {
        let responses = vec![
            (
                Data::quota(
                    "INBOX",
                    vec![QuotaGet {
                        resource: Resource::Message,
                        usage: 1024,
                        limit: 2048,
                    }],
                )
                .unwrap(),
                b"* QUOTA INBOX (MESSAGE 1024 2048)\r\n".as_ref(),
            ),
            (
                Data::quota_root("INBOX", vec![]).unwrap(),
                b"* QUOTAROOT INBOX\r\n",
            ),
            (
                Data::quota_root(
                    "INBOX",
                    vec!["ROOT1".try_into().unwrap(), "ROOT2".try_into().unwrap()],
                )
                .unwrap(),
                b"* QUOTAROOT INBOX ROOT1 ROOT2\r\n",
            ),
        ];

        compare_output(responses)
    }

    fn compare_output(items: Vec<(impl Encode, &[u8])>) {
        for item in items {
            let out = item.0.encode_detached().unwrap();
            assert_eq!(out, item.1);
        }
    }
}
