use imap_codec::{
    decode::{CommandDecodeError, Decoder},
    CommandCodec,
};

#[path = "common/common.rs"]
mod common;

use common::{read_more, COLOR_SERVER, RESET};

use crate::common::Role;

const WELCOME: &str = r#"# Parsing of IMAP commands

"C:" denotes the client,
"S:" denotes the server, and
".." denotes the continuation of an (incomplete) command, e.g., due to the use of an IMAP literal.

Note: "\n" will be automatically replaced by "\r\n".

--------------------------------------------------------------------------------------------------

Enter IMAP command (or "exit").
"#;

fn main() {
    println!("{}", WELCOME);

    let mut buffer = Vec::new();

    loop {
        // Try to parse the first command in `buffer`.
        match CommandCodec::default().decode(&buffer) {
            // Parser succeeded.
            Ok((remaining, command)) => {
                // Do something with the command ...
                println!("{:#?}", command);

                // ... and proceed with the remaining data.
                buffer = remaining.to_vec();
            }
            // Parser needs more data.
            Err(CommandDecodeError::Incomplete) => {
                // Read more data.
                read_more(&mut buffer, Role::Client);
            }
            // Parser needs more data, and a command continuation request is expected.
            Err(CommandDecodeError::LiteralFound { .. }) => {
                // Simulate literal acknowledgement ...
                println!("S: {COLOR_SERVER}+ {RESET}");

                // ... and read more data.
                read_more(&mut buffer, Role::Client);
            }
            // Parser failed.
            Err(CommandDecodeError::Failed) => {
                println!("Error parsing command.");
                println!("Clearing buffer.");

                // Clear the buffer and proceed with loop.
                buffer.clear();
            }
        }
    }
}
