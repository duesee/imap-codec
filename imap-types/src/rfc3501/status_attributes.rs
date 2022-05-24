use std::num::NonZeroU32;

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "serdex")]
use serde::{Deserialize, Serialize};

/// The currently defined status data items that can be requested.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
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
}

/// The currently defined status data items.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
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
}
