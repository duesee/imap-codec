use imap_proto_server::parse::command::command;
use std::io::Write;

fn main() -> std::io::Result<()> {
    loop {
        let line = {
            print!("Enter IMAP4REV1 command (or \"exit\"): ");
            std::io::stdout().flush().unwrap();

            let mut line = String::new();
            std::io::stdin().read_line(&mut line)?;
            line
        };

        if line.trim() == "exit" {
            break;
        }

        match command(line.as_bytes()) {
            Ok((remaining, command)) => {
                println!("{:#?}", command);

                if !remaining.is_empty() {
                    println!("Remaining data in buffer: {:?}", remaining);
                }
            }
            Err(error) => {
                println!("Error parsing the command. Is it correct? ({:?})", error);
            }
        }
    }

    Ok(())
}
