#[cfg(test)]
mod tests {
    use crate::{
        command::{Command, CommandBody},
        testing::kat_inverse_command,
    };

    #[test]
    fn test_kat_inverse_command_unselect() {
        kat_inverse_command(&[(
            b"A UNSELECT\r\n".as_ref(),
            b"".as_ref(),
            Command::new("A", CommandBody::unselect()).unwrap(),
        )]);
    }
}
