//! The IMAP OBJECTID extension (RFC 8474): `EMAILID` and `THREADID` fetch
//! attributes.

use core::{fmt, str::from_utf8};

use imap_types::{
    extensions::objectid::ObjectId,
    fetch::{MessageDataItem, MessageDataItemName},
};
use nom::{
    branch::alt,
    bytes::streaming::{tag_no_case, take_while_m_n},
    character::streaming::char,
    combinator::{map, value},
    sequence::{delimited, preceded},
};

use crate::{
    decode::IMAPResult,
    encode::{EncodeContext, EncodeIntoContext},
};

/// ```abnf
/// objectid = 1*255(ALPHA / DIGIT / "_" / "-")
/// ```
pub(crate) fn objectid(input: &[u8]) -> IMAPResult<&[u8], ObjectId> {
    map(
        take_while_m_n(1, 255, is_objectid_char),
        |bytes: &[u8]| {
            // Safety: `unwrap` can't panic because `is_objectid_char` admits ASCII only.
            ObjectId::unvalidated(from_utf8(bytes).unwrap())
        },
    )(input)
}

fn is_objectid_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

/// ```abnf
/// fetch-att =/ "EMAILID" / "THREADID"
/// ```
pub(crate) fn fetch_att_objectid(input: &[u8]) -> IMAPResult<&[u8], MessageDataItemName> {
    alt((
        value(MessageDataItemName::EmailId, tag_no_case(b"EMAILID")),
        value(MessageDataItemName::ThreadId, tag_no_case(b"THREADID")),
    ))(input)
}

/// ```abnf
/// msg-att-static =/ fetch-emailid-resp / fetch-threadid-resp
///
/// fetch-emailid-resp  = "EMAILID" SP "(" objectid ")"
/// fetch-threadid-resp = "THREADID" SP ( "(" objectid ")" / nil )
/// ```
pub(crate) fn msg_att_objectid(input: &[u8]) -> IMAPResult<&[u8], MessageDataItem> {
    alt((
        map(
            preceded(
                tag_no_case(b"EMAILID "),
                delimited(char('('), objectid, char(')')),
            ),
            MessageDataItem::EmailId,
        ),
        map(
            preceded(
                tag_no_case(b"THREADID "),
                alt((
                    map(delimited(char('('), objectid, char(')')), Some),
                    value(None, tag_no_case(b"NIL")),
                )),
            ),
            MessageDataItem::ThreadId,
        ),
    ))(input)
}

impl EncodeIntoContext for ObjectId<'_> {
    fn encode_ctx(&self, ctx: &mut EncodeContext) -> fmt::Result {
        ctx.write_all(self.inner().as_bytes())
    }
}

pub(crate) fn encode_email_id(id: &ObjectId, ctx: &mut EncodeContext) -> fmt::Result {
    ctx.write_all(b"EMAILID (")?;
    id.encode_ctx(ctx)?;
    ctx.write_all(b")")
}

pub(crate) fn encode_thread_id(id: &Option<ObjectId>, ctx: &mut EncodeContext) -> fmt::Result {
    match id {
        Some(id) => {
            ctx.write_all(b"THREADID (")?;
            id.encode_ctx(ctx)?;
            ctx.write_all(b")")
        }
        None => ctx.write_all(b"THREADID NIL"),
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::testing::{kat_inverse_response, known_answer_test_encode};

    #[test]
    fn test_encode_message_data_item_name() {
        let tests = [
            (MessageDataItemName::EmailId, b"EMAILID".as_ref()),
            (MessageDataItemName::ThreadId, b"THREADID"),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }

    #[test]
    fn test_parse_msg_att_objectid() {
        let (_, item) = msg_att_objectid(b"EMAILID (M6d99ac3275bb4e) ").unwrap();
        assert_eq!(
            item,
            MessageDataItem::EmailId(ObjectId::try_from("M6d99ac3275bb4e").unwrap())
        );

        let (_, item) = msg_att_objectid(b"THREADID (T64b478a75b7ea9de))").unwrap();
        assert_eq!(
            item,
            MessageDataItem::ThreadId(Some(ObjectId::try_from("T64b478a75b7ea9de").unwrap()))
        );

        let (_, item) = msg_att_objectid(b"THREADID NIL)").unwrap();
        assert_eq!(item, MessageDataItem::ThreadId(None));

        assert!(msg_att_objectid(b"EMAILID (a.b)").is_err());
        assert!(msg_att_objectid(b"EMAILID ()").is_err());
    }

    #[test]
    fn test_kat_inverse_fetch_response() {
        kat_inverse_response(&[(
            b"* 3 FETCH (UID 7 EMAILID (M6d99ac3275bb4e) THREADID NIL)\r\n".as_ref(),
            b"".as_ref(),
            imap_types::response::Response::Data(imap_types::response::Data::Fetch {
                seq: core::num::NonZeroU32::new(3).unwrap(),
                items: imap_types::core::Vec1::try_from(vec![
                    MessageDataItem::Uid(core::num::NonZeroU32::new(7).unwrap()),
                    MessageDataItem::EmailId(ObjectId::try_from("M6d99ac3275bb4e").unwrap()),
                    MessageDataItem::ThreadId(None),
                ])
                .unwrap(),
            }),
        )]);
    }
}
