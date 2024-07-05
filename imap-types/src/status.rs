use std::num::NonZeroU32;

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Status data item name used to request a status data item.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
#[doc(alias = "StatusAttribute")]
pub enum StatusDataItemName {
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
    Deleted,

    /// The amount of storage space that can be reclaimed by performing EXPUNGE on the mailbox.
    DeletedStorage,

    #[cfg(feature = "ext_condstore_qresync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ext_condstore_qresync")))]
    HighestModSeq,
}

/// Status data item.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
#[doc(alias = "StatusAttributeValue")]
pub enum StatusDataItem {
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
    Deleted(u32),

    /// The amount of storage space that can be reclaimed by performing EXPUNGE on the mailbox.
    DeletedStorage(u64),
}
