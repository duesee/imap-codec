//! IMAP - MOVE Extension

use std::convert::TryInto;

use thiserror::Error;

use crate::{
    command::{CommandBody, SequenceSet},
    message::Mailbox,
};

impl<'a> CommandBody<'a> {
    pub fn r#move<S, M>(
        sequence_set: S,
        mailbox: M,
        uid: bool,
    ) -> Result<Self, MoveError<S::Error, M::Error>>
    where
        S: TryInto<SequenceSet>,
        M: TryInto<Mailbox<'a>>,
    {
        Ok(CommandBody::Move {
            sequence_set: sequence_set.try_into().map_err(MoveError::Sequence)?,
            mailbox: mailbox.try_into().map_err(MoveError::Mailbox)?,
            uid,
        })
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum MoveError<S, M> {
    #[error("Invalid sequence: {0:?}")]
    Sequence(S),
    #[error("Invalid mailbox: {0:?}")]
    Mailbox(M),
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::*;
    use crate::codec::Encode;

    #[test]
    fn test_encode_command_body_move() {
        let tests = [
            (
                CommandBody::r#move("1", "inBox", false).unwrap(),
                CommandBody::Move {
                    sequence_set: SequenceSet::try_from(1).unwrap(),
                    mailbox: Mailbox::Inbox,
                    uid: false,
                },
                b"MOVE 1 INBOX".as_ref(),
            ),
            (
                CommandBody::r#move("1", "inBox", true).unwrap(),
                CommandBody::Move {
                    sequence_set: SequenceSet::try_from(1).unwrap(),
                    mailbox: Mailbox::Inbox,
                    uid: true,
                },
                b"UID MOVE 1 INBOX".as_ref(),
            ),
        ];

        for (test_1, test_2, expected) in tests {
            assert_eq!(test_1, test_2);

            let got = test_1.encode_detached().unwrap();
            assert_eq!(expected, got);
        }
    }
}
