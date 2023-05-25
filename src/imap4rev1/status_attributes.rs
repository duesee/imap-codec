use abnf_core::streaming::SP;
use imap_types::{command::status::StatusAttribute, response::data::StatusAttributeValue};
use nom::{
    branch::alt,
    bytes::streaming::tag_no_case,
    combinator::{map, value},
    multi::separated_list1,
    sequence::tuple,
    IResult,
};

#[cfg(feature = "ext_quota")]
use crate::imap4rev1::core::number64;
use crate::imap4rev1::core::{number, nz_number};

/// `status-att = "MESSAGES" /
///               "RECENT" /
///               "UIDNEXT" /
///               "UIDVALIDITY" /
///               "UNSEEN"`
pub fn status_att(input: &[u8]) -> IResult<&[u8], StatusAttribute> {
    alt((
        value(StatusAttribute::Messages, tag_no_case(b"MESSAGES")),
        value(StatusAttribute::Recent, tag_no_case(b"RECENT")),
        value(StatusAttribute::UidNext, tag_no_case(b"UIDNEXT")),
        value(StatusAttribute::UidValidity, tag_no_case(b"UIDVALIDITY")),
        value(StatusAttribute::Unseen, tag_no_case(b"UNSEEN")),
        #[cfg(feature = "ext_quota")]
        value(
            StatusAttribute::DeletedStorage,
            tag_no_case(b"DELETED-STORAGE"),
        ),
        #[cfg(feature = "ext_quota")]
        value(StatusAttribute::Deleted, tag_no_case(b"DELETED")),
        #[cfg(feature = "ext_condstore_qresync")]
        value(
            StatusAttribute::HighestModSeq,
            tag_no_case(b"HIGHESTMODSEQ"),
        ),
    ))(input)
}

/// `status-att-list = status-att-val *(SP status-att-val)`
///
/// Note: See errata id: 261
pub fn status_att_list(input: &[u8]) -> IResult<&[u8], Vec<StatusAttributeValue>> {
    separated_list1(SP, status_att_val)(input)
}

/// `status-att-val  = ("MESSAGES" SP number) /
///                    ("RECENT" SP number) /
///                    ("UIDNEXT" SP nz-number) /
///                    ("UIDVALIDITY" SP nz-number) /
///                    ("UNSEEN" SP number)`
///
/// Note: See errata id: 261
fn status_att_val(input: &[u8]) -> IResult<&[u8], StatusAttributeValue> {
    alt((
        map(
            tuple((tag_no_case(b"MESSAGES"), SP, number)),
            |(_, _, num)| StatusAttributeValue::Messages(num),
        ),
        map(
            tuple((tag_no_case(b"RECENT"), SP, number)),
            |(_, _, num)| StatusAttributeValue::Recent(num),
        ),
        map(
            tuple((tag_no_case(b"UIDNEXT"), SP, nz_number)),
            |(_, _, next)| StatusAttributeValue::UidNext(next),
        ),
        map(
            tuple((tag_no_case(b"UIDVALIDITY"), SP, nz_number)),
            |(_, _, val)| StatusAttributeValue::UidValidity(val),
        ),
        map(
            tuple((tag_no_case(b"UNSEEN"), SP, number)),
            |(_, _, num)| StatusAttributeValue::Unseen(num),
        ),
        #[cfg(feature = "ext_quota")]
        map(
            tuple((tag_no_case(b"DELETED-STORAGE"), SP, number64)),
            |(_, _, num)| StatusAttributeValue::DeletedStorage(num),
        ),
        #[cfg(feature = "ext_quota")]
        map(
            tuple((tag_no_case(b"DELETED"), SP, number)),
            |(_, _, num)| StatusAttributeValue::Deleted(num),
        ),
    ))(input)
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

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
