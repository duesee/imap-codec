//! Gmail's IMAP extensions (`X-GM-EXT-1`): `X-GM-MSGID`, `X-GM-THRID` and
//! `X-GM-LABELS` fetch attributes.

use core::fmt;

use abnf_core::streaming::sp;
use imap_types::{
    extensions::gmail::GmailLabel,
    fetch::{MessageDataItem, MessageDataItemName},
};
use nom::{
    branch::alt,
    bytes::streaming::tag_no_case,
    character::streaming::char,
    combinator::{map, opt, value},
    multi::separated_list1,
    sequence::{delimited, preceded},
};

use crate::{
    core::{astring, atom, number64},
    decode::IMAPResult,
    encode::{EncodeContext, EncodeIntoContext, utils::join_serializable},
};

/// ```abnf
/// fetch-att =/ "X-GM-MSGID" / "X-GM-THRID" / "X-GM-LABELS"
/// ```
pub(crate) fn fetch_att_gmail(input: &[u8]) -> IMAPResult<&[u8], MessageDataItemName> {
    alt((
        value(
            MessageDataItemName::GmailMessageId,
            tag_no_case(b"X-GM-MSGID"),
        ),
        value(
            MessageDataItemName::GmailThreadId,
            tag_no_case(b"X-GM-THRID"),
        ),
        value(
            MessageDataItemName::GmailLabels,
            tag_no_case(b"X-GM-LABELS"),
        ),
    ))(input)
}

/// ```abnf
/// msg-att-gmail = "X-GM-MSGID" SP number64 /
///                 "X-GM-THRID" SP number64 /
///                 "X-GM-LABELS" SP "(" [x-gm-label *(SP x-gm-label)] ")"
/// ```
pub(crate) fn msg_att_gmail(input: &[u8]) -> IMAPResult<&[u8], MessageDataItem> {
    alt((
        map(
            preceded(tag_no_case(b"X-GM-MSGID "), number64),
            MessageDataItem::GmailMessageId,
        ),
        map(
            preceded(tag_no_case(b"X-GM-THRID "), number64),
            MessageDataItem::GmailThreadId,
        ),
        map(
            preceded(
                tag_no_case(b"X-GM-LABELS "),
                delimited(char('('), opt(separated_list1(sp, gmail_label)), char(')')),
            ),
            |labels| MessageDataItem::GmailLabels(labels.unwrap_or_default()),
        ),
    ))(input)
}

/// ```abnf
/// x-gm-label = "\" atom / astring
/// ```
pub(crate) fn gmail_label(input: &[u8]) -> IMAPResult<&[u8], GmailLabel> {
    alt((
        map(preceded(char('\\'), atom), GmailLabel::System),
        map(astring, GmailLabel::Name),
    ))(input)
}

impl EncodeIntoContext for GmailLabel<'_> {
    fn encode_ctx(&self, ctx: &mut EncodeContext) -> fmt::Result {
        match self {
            Self::System(atom) => {
                ctx.write_all(b"\\")?;
                atom.encode_ctx(ctx)
            }
            Self::Name(name) => name.encode_ctx(ctx),
        }
    }
}

pub(crate) fn encode_gmail_labels(labels: &[GmailLabel], ctx: &mut EncodeContext) -> fmt::Result {
    ctx.write_all(b"X-GM-LABELS (")?;
    join_serializable(labels, b" ", ctx)?;
    ctx.write_all(b")")
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use imap_types::{
        core::{AString, Atom},
        fetch::MessageDataItem,
    };

    use super::*;
    use crate::testing::{kat_inverse_response, known_answer_test_encode};

    #[test]
    fn test_encode_message_data_item_name() {
        let tests = [
            (MessageDataItemName::GmailMessageId, b"X-GM-MSGID".as_ref()),
            (MessageDataItemName::GmailThreadId, b"X-GM-THRID"),
            (MessageDataItemName::GmailLabels, b"X-GM-LABELS"),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }

    #[test]
    fn test_parse_msg_att_gmail() {
        let (rem, item) = msg_att_gmail(b"X-GM-MSGID 1278455344230334865 ").unwrap();
        assert_eq!(rem, b" ");
        assert_eq!(item, MessageDataItem::GmailMessageId(1278455344230334865));

        let (_, item) = msg_att_gmail(b"X-GM-THRID 1266894439832287888)").unwrap();
        assert_eq!(item, MessageDataItem::GmailThreadId(1266894439832287888));

        let (_, item) =
            msg_att_gmail(b"X-GM-LABELS (\\Inbox \"\\\\Important\" Work \"Muy Importante\"))")
                .unwrap();
        assert_eq!(
            item,
            MessageDataItem::GmailLabels(vec![
                GmailLabel::System(Atom::try_from("Inbox").unwrap()),
                GmailLabel::Name(AString::try_from("\\Important").unwrap()),
                GmailLabel::Name(AString::try_from("Work").unwrap()),
                GmailLabel::Name(AString::try_from("Muy Importante").unwrap()),
            ])
        );

        let (_, item) = msg_att_gmail(b"X-GM-LABELS ())").unwrap();
        assert_eq!(item, MessageDataItem::GmailLabels(vec![]));
    }

    #[test]
    fn test_kat_inverse_fetch_response() {
        kat_inverse_response(&[(
            b"* 1 FETCH (UID 4 X-GM-MSGID 1278455344230334865 X-GM-THRID 1266894439832287888 X-GM-LABELS (\\Inbox Work \"Muy Importante\"))\r\n".as_ref(),
            b"".as_ref(),
            imap_types::response::Response::Data(
                imap_types::response::Data::Fetch {
                    seq: core::num::NonZeroU32::new(1).unwrap(),
                    items: imap_types::core::Vec1::try_from(vec![
                        MessageDataItem::Uid(core::num::NonZeroU32::new(4).unwrap()),
                        MessageDataItem::GmailMessageId(1278455344230334865),
                        MessageDataItem::GmailThreadId(1266894439832287888),
                        MessageDataItem::GmailLabels(vec![
                            GmailLabel::System(Atom::try_from("Inbox").unwrap()),
                            GmailLabel::Name(AString::try_from("Work").unwrap()),
                            GmailLabel::Name(AString::try_from("Muy Importante").unwrap()),
                        ]),
                    ])
                    .unwrap(),
                },
            ),
        )]);
    }
}
