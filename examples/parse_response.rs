use imap_proto_server::parse::response::response;
use std::io::Write;

fn main() -> std::io::Result<()> {
    loop {
        let line = {
            print!("Enter IMAP4REV1 response (or \"exit\"): ");
            std::io::stdout().flush().unwrap();

            let mut line = String::new();
            std::io::stdin().read_line(&mut line)?;
            line
        };

        if line.trim() == "exit" {
            break;
        }

        match response(line.as_bytes()) {
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
