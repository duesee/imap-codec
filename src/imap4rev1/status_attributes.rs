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
