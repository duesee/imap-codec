use std::io::Write;

use imap_codec::{codec::Decode, response::Response};

fn main() -> std::io::Result<()> {
    loop {
        let line = {
            print!("Enter IMAP4REV1 response (or \"exit\"): ");
            std::io::stdout().flush().unwrap();

            let mut line = String::new();
            std::io::stdin().read_line(&mut line)?;
            line.replace("\n", "\r\n")
        };

        if line.trim() == "exit" {
            break;
        }

        match Response::decode(line.as_bytes()) {
            Ok((remaining, response)) => {
                println!("{:#?}", response);

                if !remaining.is_empty() {
                    println!("Remaining data in buffer: {:?}", remaining);
                }
            }
            Err(error) => {
                println!("Error parsing the response. Is it correct? ({:?})", error);
            }
        }
    }

    Ok(())
}
