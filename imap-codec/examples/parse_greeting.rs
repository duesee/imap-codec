use imap_codec::{
    decode::{Decoder, GreetingDecodeError},
    GreetingCodec,
};

#[path = "common/common.rs"]
mod common;

use common::read_more;

use crate::common::Role;

const WELCOME: &str = r#"# Parsing of IMAP greetings

"S:" denotes the server.

Note: "\n" will be automatically replaced by "\r\n".

--------------------------------------------------------------------------------------------------

Enter IMAP greeting (or "exit").
"#;

fn main() {
    println!("{}", WELCOME);

    let mut buffer = Vec::new();

    loop {
        // Try to parse the first greeting in `buffer`.
        match GreetingCodec::default().decode(&buffer) {
            // Parser succeeded.
            Ok((remaining, greeting)) => {
                // Do something with the greeting ...
                println!("{:#?}", greeting);

                // ... and proceed with the remaining data.
                buffer = remaining.to_vec();
            }
            // Parser needs more data.
            Err(GreetingDecodeError::Incomplete) => {
                // Read more data.
                read_more(&mut buffer, Role::Server);
            }
            // Parser failed.
            Err(GreetingDecodeError::Failed) => {
                println!("Error parsing greeting.");
                println!("Clearing buffer.");

                // Clear the buffer and proceed with loop.
                buffer.clear();
            }
        }
    }
}
