//! The IMAP UNSELECT command

use crate::command::CommandBody;

impl CommandBody<'_> {
    pub fn unselect() -> Self {
        CommandBody::Unselect
    }
}
