//! IMAP - MOVE Extension

use crate::{
    command::CommandBody, extensions::r#move::error::MoveError, mailbox::Mailbox,
    sequence::SequenceSet,
};

impl<'a> CommandBody<'a> {
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the MOVE capability.
    /// </div>
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

/// Error-related types.
pub mod error {
    use thiserror::Error;

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum MoveError<S, M> {
        #[error("Invalid sequence: {0}")]
        Sequence(S),
        #[error("Invalid mailbox: {0}")]
        Mailbox(M),
    }
}
