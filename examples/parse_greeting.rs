use std::io::{Read, Result as IoResult, Write};

use imap_codec::{codec::Decode, types::response::Response};

pub fn read_file(path: &str) -> IoResult<Vec<u8>> {
    let mut file = std::fs::File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    Ok(data)
}

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();

    if let Some(path) = args.nth(1) {
        let data = read_file(&path).unwrap();

        // FIXME: Greeting != Response
        match Response::decode(&data) {
            Ok((remaining, greeting)) => {
                println!("{:#?}", greeting);

                if !remaining.is_empty() {
                    println!("Remaining data in buffer: {:?}", remaining);
                }
            }
            Err(error) => {
                println!("Error parsing the greeting. Is it correct? ({:?})", error);
            }
        }

        return Ok(());
    }

    loop {
        let line = {
            print!("Enter IMAP4REV1 greeting (or \"exit\"): ");
            std::io::stdout().flush().unwrap();

            let mut line = String::new();
            std::io::stdin().read_line(&mut line)?;
            line.replace("\n", "\r\n")
        };

        if line.trim() == "exit" {
            break;
        }

        // TODO: Greeting != Response
        match Response::decode(line.as_bytes()) {
            Ok((remaining, greeting)) => {
                println!("{:#?}", greeting);

                if !remaining.is_empty() {
                    println!("Remaining data in buffer: {:?}", remaining);
                }
            }
            Err(error) => {
                println!("Error parsing the greeting. Is it correct? ({:?})", error);
            }
        }
    }

    Ok(())
}
