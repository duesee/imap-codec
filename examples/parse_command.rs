use std::io::Write;

use ansi_term::Colour::{Blue as ColorServer, Red as ColorClient};
use imap_codec::parse::command::command;

fn main() {
    welcome();

    let mut buffer = Vec::new();

    loop {
        // Try to parse the first command in `buffer`.
        match command(&buffer) {
            // Parser succeeded.
            Ok((remaining, command)) => {
                // Do something with the command ...
                println!("{:#?}", command);

                // ... and proceed with the remaining data.
                buffer = remaining.to_vec();
            }
            // Parser needs more data.
            Err(nom::Err::Incomplete(_needed)) => {
                // Read more data.
                read_more(&mut buffer);
            }
            // Parser needs more data, and a literal acknowledgement action is required.
            // This step is crucial for real clients. Otherwise a client won't send any more data.
            // TODO: `ErrorKind::Fix` is used (for now) to signal that a server acknowledgement is required.
            Err(nom::Err::Failure(failure)) if failure.code == nom::error::ErrorKind::Fix => {
                // Simulate literal acknowledgement ...
                println!("S: {}", ColorServer.paint("+ "));

                // ... and read more data.
                read_more(&mut buffer);
            }
            // Parser failed.
            Err(nom::Err::Error(error)) | Err(nom::Err::Failure(error)) => {
                println!("Error parsing command. Is it correct? ({:?})", error);
                println!("Clearing buffer.");

                // Clear the buffer and proceed with loop.
                buffer.clear();
            }
        }
    }
}

fn welcome() {
    let welcome = r#"
# Parsing of IMAP commands.

As a user, you are in the role of an IMAP client. Thus, you type IMAP commands. However, the example code shows typical server code, i.e., how to parse received commands.

"C:" denotes the client, "S:" denotes the server, and ".." denotes the continuation of an (incomplete) command, e.g., due to the use of IMAP literals.

Enter command (or "exit").
"#;

    println!("{}", welcome);
}

fn read_more(buffer: &mut Vec<u8>) {
    let prompt = if buffer.is_empty() { "C: " } else { ".. " };

    let line = read_line(prompt);

    if line.trim() == "exit" || line.trim() == "" {
        std::process::exit(0);
    }

    buffer.extend_from_slice(line.as_bytes());
}

pub fn read_line(prompt: &str) -> String {
    print!("{}{}", prompt, ColorClient.prefix());
    std::io::stdout().flush().unwrap();

    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();

    print!("{}", ColorClient.suffix());

    line
}
