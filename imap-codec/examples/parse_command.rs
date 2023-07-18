use std::io::Write;

use imap_codec::{
    codec::{Decode, DecodeError},
    imap_types::command::Command,
};

const COLOR_SERVER: &str = "\x1b[34m";
const COLOR_CLIENT: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

fn main() {
    welcome();

    let mut buffer = Vec::new();

    loop {
        // Try to parse the first command in `buffer`.
        match Command::decode(&buffer) {
            // Parser succeeded.
            Ok((remaining, command)) => {
                // Do something with the command ...
                println!("{:#?}", command);

                // ... and proceed with the remaining data.
                buffer = remaining.to_vec();

                // Note: The `command` object is currently bounded to the `buffer`, i.e., we have
                // something like a `Command<'buffer>` here. We can use the `bounded-static` feature
                // and call `command.into_static()` to convert it into a more flexible `Command<'static>`.
            }
            // Parser needs more data.
            Err(DecodeError::Incomplete) => {
                // Read more data.
                read_more(&mut buffer);
            }
            // Parser needs more data, and a literal acknowledgement action is required.
            // This step is crucial for real clients. Otherwise a client won't send any more data.
            Err(DecodeError::LiteralFound { .. }) => {
                // Simulate literal acknowledgement ...
                println!("S: {COLOR_SERVER}+ {RESET}");

                // ... and read more data.
                read_more(&mut buffer);
            }
            // Parser failed.
            Err(DecodeError::Failed) => {
                println!("Error parsing command.");
                println!("Clearing buffer.");

                // Clear the buffer and proceed with loop.
                buffer.clear();
            }
        }
    }
}

fn welcome() {
    let welcome = r#"# Parsing of IMAP commands

"C:" denotes the client,
"S:" denotes the server, and
".." denotes the continuation of an (incomplete) command, e.g., due to the use of an IMAP literal.

Note: "\n" will be automatically replaced by "\r\n".

--------------------------------------------------------------------------------------------------

Enter IMAP command (or "exit").
"#;

    println!("{}", welcome);
}

fn read_more(buffer: &mut Vec<u8>) {
    let prompt = if buffer.is_empty() { "C: " } else { ".. " };

    let line = read_line(prompt);

    if line.trim() == "exit" {
        println!("Exiting.");
        std::process::exit(0);
    }

    buffer.extend_from_slice(line.as_bytes());
}

pub fn read_line(prompt: &str) -> String {
    print!("{}{COLOR_CLIENT}", prompt);
    std::io::stdout().flush().unwrap();

    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();

    print!("{RESET}");

    line.replace('\n', "\r\n")
}
