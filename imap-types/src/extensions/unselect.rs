//! The IMAP UNSELECT command

use crate::command::CommandBody;

impl CommandBody<'_> {
    pub fn unselect() -> Self {
        CommandBody::Unselect
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::Encode;

    #[test]
    fn test_encode_command_body_unselect() {
        let tests = [(
            CommandBody::unselect(),
            CommandBody::Unselect,
            b"UNSELECT".as_ref(),
        )];

        for (test_1, test_2, expected) in tests {
            assert_eq!(test_1, test_2);

            let got = test_1.encode_detached().unwrap();
            assert_eq!(expected, got);
        }
    }
}
