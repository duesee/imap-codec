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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::known_answer_test_encode;

    #[test]
    fn test_encode_status_attribute() {
        let tests = [
            (StatusAttribute::Messages, b"MESSAGES".as_ref()),
            (StatusAttribute::Recent, b"RECENT"),
            (StatusAttribute::UidNext, b"UIDNEXT"),
            (StatusAttribute::UidValidity, b"UIDVALIDITY"),
            (StatusAttribute::Unseen, b"UNSEEN"),
            #[cfg(feature = "ext_quota")]
            (StatusAttribute::Deleted, b"DELETED"),
            #[cfg(feature = "ext_quota")]
            (StatusAttribute::DeletedStorage, b"DELETED-STORAGE"),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }

    #[test]
    fn test_encode_status_attribute_value() {
        let tests = [
            (StatusAttributeValue::Messages(0), b"MESSAGES 0".as_ref()),
            (StatusAttributeValue::Recent(u32::MAX), b"RECENT 4294967295"),
            (
                StatusAttributeValue::UidNext(NonZeroU32::new(1).unwrap()),
                b"UIDNEXT 1",
            ),
            (
                StatusAttributeValue::UidValidity(NonZeroU32::new(u32::MAX).unwrap()),
                b"UIDVALIDITY 4294967295",
            ),
            (StatusAttributeValue::Unseen(0), b"UNSEEN 0"),
            #[cfg(feature = "ext_quota")]
            (StatusAttributeValue::Deleted(1), b"DELETED 1"),
            #[cfg(feature = "ext_quota")]
            (
                StatusAttributeValue::DeletedStorage(u64::MAX),
                b"DELETED-STORAGE 18446744073709551615",
            ),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }
}
