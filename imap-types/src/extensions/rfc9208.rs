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
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    codec::Encode,
    rfc3501::core::{impl_try_from, impl_try_from_try_from, Atom},
};

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
    Other(ResourceOther<'a>),
}

impl<'a> Resource<'a> {
    /// Try to create a non-standard resource from a value.
    ///
    /// Note: The value must be semantically different from the supported variants.
    pub fn other<A>(other: A) -> Result<Self, ()>
    where
        A: TryInto<Atom<'a>>,
    {
        let atom = other.try_into().map_err(|_| ())?;

        Ok(Resource::Other(
            ResourceOther::try_from(atom).map_err(|_| ())?,
        ))
    }
}

impl_try_from!(Atom, 'a, &'a [u8], Resource<'a>);
impl_try_from!(Atom, 'a, Vec<u8>, Resource<'a>);
impl_try_from!(Atom, 'a, &'a str, Resource<'a>);
impl_try_from!(Atom, 'a, String, Resource<'a>);
impl_try_from!(Atom, 'a, Cow<'a, str>, Resource<'a>);

impl<'a> From<Atom<'a>> for Resource<'a> {
    fn from(value: Atom<'a>) -> Self {
        match value.inner().to_ascii_lowercase().as_ref() {
            "storage" => Resource::Storage,
            "message" => Resource::Message,
            "mailbox" => Resource::Mailbox,
            "annotation-storage" => Resource::AnnotationStorage,
            _ => Resource::Other(ResourceOther { inner: value }),
        }
    }
}

impl<'a> Encode for Resource<'a> {
    fn encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
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
pub struct ResourceOther<'a> {
    inner: Atom<'a>,
}

impl<'a> ResourceOther<'a> {
    #[cfg(feature = "unchecked")]
    pub fn new_unchecked(atom: Atom<'a>) -> Self {
        Self { inner: atom }
    }
}

impl_try_from_try_from!(Atom, 'a, &'a [u8], ResourceOther<'a>);
impl_try_from_try_from!(Atom, 'a, Vec<u8>, ResourceOther<'a>);
impl_try_from_try_from!(Atom, 'a, &'a str, ResourceOther<'a>);
impl_try_from_try_from!(Atom, 'a, String, ResourceOther<'a>);
impl_try_from_try_from!(Atom, 'a, Cow<'a, str>, ResourceOther<'a>);

impl<'a> TryFrom<Atom<'a>> for ResourceOther<'a> {
    type Error = ();

    fn try_from(atom: Atom<'a>) -> Result<Self, Self::Error> {
        match atom.inner.to_lowercase().as_ref() {
            "storage" | "message" | "mailbox" | "annotation-storage" => Err(()),
            _ => Ok(Self { inner: atom }),
        }
    }
}

impl<'a> Encode for ResourceOther<'a> {
    fn encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.inner.encode(writer)
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

impl<'a> Encode for QuotaGet<'a> {
    fn encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
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
    fn encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.resource.encode(writer)?;
        write!(writer, " {}", self.limit)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;
    use crate::{codec::Encode, response::Data, rfc3501::command::CommandBody};

    fn compare_output(items: Vec<(Result<impl Encode, ()>, &str)>) {
        for item in items {
            let out = item.0.unwrap().encode_detached().unwrap();
            assert_eq!(std::str::from_utf8(&out).unwrap(), item.1);
        }
    }

    #[test]
    fn test_command_output() {
        let commands = vec![
            (CommandBody::get_quota("INBOX"), "GETQUOTA INBOX"),
            (CommandBody::get_quota(""), "GETQUOTA \"\""),
            (
                CommandBody::get_quota_root("MAILBOX"),
                "GETQUOTAROOT MAILBOX",
            ),
            (CommandBody::set_quota("INBOX", vec![]), "SETQUOTA INBOX ()"),
            (
                CommandBody::set_quota(
                    "INBOX",
                    vec![QuotaSet {
                        resource: Resource::Storage,
                        limit: 256,
                    }],
                ),
                "SETQUOTA INBOX (STORAGE 256)",
            ),
            (
                CommandBody::set_quota(
                    "INBOX",
                    vec![
                        QuotaSet {
                            resource: Resource::Message,
                            limit: 256,
                        },
                        QuotaSet {
                            resource: Resource::Storage,
                            limit: 512,
                        },
                    ],
                ),
                "SETQUOTA INBOX (MESSAGE 256 STORAGE 512)",
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
                ),
                "* QUOTA INBOX (MESSAGE 1024 2048)\r\n",
            ),
            (Data::quota_root("INBOX", vec![]), "* QUOTAROOT INBOX\r\n"),
            (
                Data::quota_root(
                    "INBOX",
                    vec!["ROOT1".try_into().unwrap(), "ROOT2".try_into().unwrap()],
                ),
                "* QUOTAROOT INBOX ROOT1 ROOT2\r\n",
            ),
        ];

        compare_output(responses)
    }
}
