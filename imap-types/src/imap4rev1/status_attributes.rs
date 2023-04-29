use std::num::NonZeroU32;

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The currently defined status data items that can be requested.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StatusAttribute {
    /// The number of messages in the mailbox.
    Messages,

    /// The number of messages with the \Recent flag set.
    Recent,

    /// The next unique identifier value of the mailbox.
    UidNext,

    /// The unique identifier validity value of the mailbox.
    UidValidity,

    /// The number of messages which do not have the \Seen flag set.
    Unseen,

    /// The number of messages with the \Deleted flag set.
    #[cfg(feature = "ext_quota")]
    Deleted,

    /// The amount of storage space that can be reclaimed by performing EXPUNGE on the mailbox.
    #[cfg(feature = "ext_quota")]
    DeletedStorage,
}

/// The currently defined status data items.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StatusAttributeValue {
    /// The number of messages in the mailbox.
    Messages(u32),

    /// The number of messages with the \Recent flag set.
    Recent(u32),

    /// The next unique identifier value of the mailbox.  Refer to
    /// section 2.3.1.1 for more information.
    UidNext(NonZeroU32),

    /// The unique identifier validity value of the mailbox.  Refer to
    /// section 2.3.1.1 for more information.
    UidValidity(NonZeroU32),

    /// The number of messages which do not have the \Seen flag set.
    Unseen(u32),

    /// The number of messages with the \Deleted flag set.
    #[cfg(feature = "ext_quota")]
    Deleted(u32),

    /// The amount of storage space that can be reclaimed by performing EXPUNGE on the mailbox.
    #[cfg(feature = "ext_quota")]
    DeletedStorage(u64),
}
