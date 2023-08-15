use abnf_core::streaming::sp;
use imap_types::status::{StatusDataItem, StatusDataItemName};
use nom::{
    branch::alt,
    bytes::streaming::tag_no_case,
    combinator::{map, value},
    multi::separated_list1,
    sequence::tuple,
};

use crate::{
    core::{number, number64, nz_number},
    decode::IMAPResult,
};

/// `status-att = "MESSAGES" /
///               "RECENT" /
///               "UIDNEXT" /
///               "UIDVALIDITY" /
///               "UNSEEN"`
pub(crate) fn status_att(input: &[u8]) -> IMAPResult<&[u8], StatusDataItemName> {
    alt((
        value(StatusDataItemName::Messages, tag_no_case(b"MESSAGES")),
        value(StatusDataItemName::Recent, tag_no_case(b"RECENT")),
        value(StatusDataItemName::UidNext, tag_no_case(b"UIDNEXT")),
        value(StatusDataItemName::UidValidity, tag_no_case(b"UIDVALIDITY")),
        value(StatusDataItemName::Unseen, tag_no_case(b"UNSEEN")),
        value(
            StatusDataItemName::DeletedStorage,
            tag_no_case(b"DELETED-STORAGE"),
        ),
        value(StatusDataItemName::Deleted, tag_no_case(b"DELETED")),
        #[cfg(feature = "ext_condstore_qresync")]
        value(
            StatusDataItemName::HighestModSeq,
            tag_no_case(b"HIGHESTMODSEQ"),
        ),
    ))(input)
}

/// `status-att-list = status-att-val *(SP status-att-val)`
///
/// Note: See errata id: 261
pub(crate) fn status_att_list(input: &[u8]) -> IMAPResult<&[u8], Vec<StatusDataItem>> {
    separated_list1(sp, status_att_val)(input)
}

/// `status-att-val  = ("MESSAGES" SP number) /
///                    ("RECENT" SP number) /
///                    ("UIDNEXT" SP nz-number) /
///                    ("UIDVALIDITY" SP nz-number) /
///                    ("UNSEEN" SP number)`
///
/// Note: See errata id: 261
fn status_att_val(input: &[u8]) -> IMAPResult<&[u8], StatusDataItem> {
    alt((
        map(
            tuple((tag_no_case(b"MESSAGES"), sp, number)),
            |(_, _, num)| StatusDataItem::Messages(num),
        ),
        map(
            tuple((tag_no_case(b"RECENT"), sp, number)),
            |(_, _, num)| StatusDataItem::Recent(num),
        ),
        map(
            tuple((tag_no_case(b"UIDNEXT"), sp, nz_number)),
            |(_, _, next)| StatusDataItem::UidNext(next),
        ),
        map(
            tuple((tag_no_case(b"UIDVALIDITY"), sp, nz_number)),
            |(_, _, val)| StatusDataItem::UidValidity(val),
        ),
        map(
            tuple((tag_no_case(b"UNSEEN"), sp, number)),
            |(_, _, num)| StatusDataItem::Unseen(num),
        ),
        map(
            tuple((tag_no_case(b"DELETED-STORAGE"), sp, number64)),
            |(_, _, num)| StatusDataItem::DeletedStorage(num),
        ),
        map(
            tuple((tag_no_case(b"DELETED"), sp, number)),
            |(_, _, num)| StatusDataItem::Deleted(num),
        ),
    ))(input)
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::*;
    use crate::testing::known_answer_test_encode;

    #[test]
    fn test_encode_status_data_item_name() {
        let tests = [
            (StatusDataItemName::Messages, b"MESSAGES".as_ref()),
            (StatusDataItemName::Recent, b"RECENT"),
            (StatusDataItemName::UidNext, b"UIDNEXT"),
            (StatusDataItemName::UidValidity, b"UIDVALIDITY"),
            (StatusDataItemName::Unseen, b"UNSEEN"),
            (StatusDataItemName::Deleted, b"DELETED"),
            (StatusDataItemName::DeletedStorage, b"DELETED-STORAGE"),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }

    #[test]
    fn test_encode_status_data_item() {
        let tests = [
            (StatusDataItem::Messages(0), b"MESSAGES 0".as_ref()),
            (StatusDataItem::Recent(u32::MAX), b"RECENT 4294967295"),
            (
                StatusDataItem::UidNext(NonZeroU32::new(1).unwrap()),
                b"UIDNEXT 1",
            ),
            (
                StatusDataItem::UidValidity(NonZeroU32::new(u32::MAX).unwrap()),
                b"UIDVALIDITY 4294967295",
            ),
            (StatusDataItem::Unseen(0), b"UNSEEN 0"),
            (StatusDataItem::Deleted(1), b"DELETED 1"),
            (
                StatusDataItem::DeletedStorage(u64::MAX),
                b"DELETED-STORAGE 18446744073709551615",
            ),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }
}
