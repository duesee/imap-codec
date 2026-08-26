use abnf_core::streaming::sp;
use imap_types::status::{StatusDataItem, StatusDataItemName};
use nom::{
    branch::alt,
    bytes::streaming::tag_no_case,
    combinator::{map, value},
    multi::separated_list1,
    sequence::preceded,
};

#[cfg(feature = "ext_condstore_qresync")]
use crate::extensions::condstore_qresync::mod_sequence_valzer;
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
        #[cfg(feature = "ext_status_size")]
        value(StatusDataItemName::Size, tag_no_case(b"SIZE")),
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

/// ```abnf
/// status-att-val  = "MESSAGES" SP number /
///                   "RECENT" SP number /
///                   "UIDNEXT" SP nz-number /
///                   "UIDVALIDITY" SP nz-number /
///                   "UNSEEN" SP number /
///                   "HIGHESTMODSEQ" SP mod-sequence-valzer
/// ```
///
/// Note: See errata id: 261
fn status_att_val(input: &[u8]) -> IMAPResult<&[u8], StatusDataItem> {
    alt((
        map(
            preceded(tag_no_case(b"MESSAGES "), number),
            StatusDataItem::Messages,
        ),
        map(
            preceded(tag_no_case(b"RECENT "), number),
            StatusDataItem::Recent,
        ),
        map(
            preceded(tag_no_case(b"UIDNEXT "), nz_number),
            StatusDataItem::UidNext,
        ),
        map(
            preceded(tag_no_case(b"UIDVALIDITY "), nz_number),
            StatusDataItem::UidValidity,
        ),
        map(
            preceded(tag_no_case(b"UNSEEN "), number),
            StatusDataItem::Unseen,
        ),
        map(
            preceded(tag_no_case(b"DELETED-STORAGE "), number64),
            StatusDataItem::DeletedStorage,
        ),
        map(
            preceded(tag_no_case(b"DELETED "), number),
            StatusDataItem::Deleted,
        ),
        #[cfg(feature = "ext_status_size")]
        map(
            preceded(tag_no_case(b"SIZE "), number64),
            StatusDataItem::Size,
        ),
        #[cfg(feature = "ext_condstore_qresync")]
        map(
            preceded(tag_no_case(b"HIGHESTMODSEQ "), mod_sequence_valzer),
            StatusDataItem::HighestModSeq,
        ),
    ))(input)
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    #[cfg(feature = "ext_status_size")]
    use imap_types::{
        command::CommandBody,
        mailbox::Mailbox,
        response::{Data, Response},
    };

    use super::*;
    use crate::testing::known_answer_test_encode;
    #[cfg(feature = "ext_status_size")]
    use crate::testing::{kat_inverse_command, kat_inverse_response};

    #[cfg(feature = "ext_status_size")]
    #[test]
    fn test_kat_inverse_status_size() {
        kat_inverse_command(&[(
            b"A01 STATUS frop (MESSAGES SIZE UIDNEXT)\r\n",
            b"",
            CommandBody::status(
                "frop",
                vec![
                    StatusDataItemName::Messages,
                    StatusDataItemName::Size,
                    StatusDataItemName::UidNext,
                ],
            )
            .unwrap()
            .tag("A01")
            .unwrap(),
        )]);

        kat_inverse_response(&[(
            b"* STATUS frop (MESSAGES 8 SIZE 44421 UIDNEXT 242344)\r\n",
            b"",
            Response::Data(Data::Status {
                mailbox: Mailbox::try_from("frop").unwrap(),
                items: vec![
                    StatusDataItem::Messages(8),
                    StatusDataItem::Size(44421),
                    StatusDataItem::UidNext(NonZeroU32::new(242344).unwrap()),
                ]
                .into(),
            }),
        )]);
    }

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
            #[cfg(feature = "ext_status_size")]
            (StatusDataItemName::Size, b"SIZE"),
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
            #[cfg(feature = "ext_status_size")]
            (StatusDataItem::Size(u64::MAX), b"SIZE 18446744073709551615"),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }
}
