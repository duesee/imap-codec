//! The IMAP UNSELECT command

use crate::command::CommandBody;

impl CommandBody<'_> {
    /// <div class="warning">
    /// This extension must only be used when the server advertised support for it sending the UNSELECT capability.
    /// </div>
    pub fn unselect() -> Self {
        CommandBody::Unselect
    }
}
