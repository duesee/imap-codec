//! IMAP - MOVE Extension

use abnf_core::streaming::SP;
use nom::{bytes::streaming::tag_no_case, sequence::tuple, IResult};

use crate::{
    command::CommandBody,
    imap4rev1::{mailbox::mailbox, sequence::sequence_set},
};

/// ```abnf
/// move = "MOVE" SP sequence-set SP mailbox
/// ```
pub fn r#move(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"MOVE"), SP, sequence_set, SP, mailbox));

    let (remaining, (_, _, sequence_set, _, mailbox)) = parser(input)?;

    Ok((
        remaining,
        CommandBody::Move {
            sequence_set,
            mailbox,
            uid: false,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        command::{CommandBody, SequenceSet},
        message::Mailbox,
    };

    #[test]
    fn test_parse_command_body_move() {
        let tests = [
            (
                b"MoVe 1 test\r".as_ref(),
                CommandBody::Move {
                    sequence_set: SequenceSet::try_from(1).unwrap(),
                    mailbox: Mailbox::try_from("test").unwrap(),
                    uid: false,
                },
                b"\r".as_ref(),
            ),
            (
                b"MoVe 1 INBOX\r\n",
                CommandBody::Move {
                    sequence_set: SequenceSet::try_from(1).unwrap(),
                    mailbox: Mailbox::Inbox,
                    uid: false,
                },
                b"\r\n",
            ),
        ];

        for (test, expected_command, expected_remainder) in tests {
            let (got_remainder, got_command) = r#move(test).unwrap();
            assert_eq!(expected_command, got_command);
            assert_eq!(expected_remainder, got_remainder);
        }
    }
}
