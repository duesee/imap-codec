use imap_codec::{
    decode::{Decoder, ResponseDecodeError},
    ResponseCodec,
};

#[path = "common/common.rs"]
mod common;

use common::read_more;

use crate::common::Role;

const WELCOME: &str = r#"# Parsing of IMAP responses

"S:" denotes the server, and
".." denotes the continuation of an (incomplete) response, e.g., due to the use of an IMAP literal.

Note: "\n" will be automatically replaced by "\r\n".

--------------------------------------------------------------------------------------------------

Enter IMAP response (or "exit").
"#;

fn main() {
    println!("{}", WELCOME);

    let mut buffer = Vec::new();

    loop {
        // Try to parse the first response in `buffer`.
        match ResponseCodec::default().decode(&buffer) {
            // Parser succeeded.
            Ok((remaining, response)) => {
                // Do something with the response ...
                println!("{:#?}", response);

                // ... and proceed with the remaining data.
                buffer = remaining.to_vec();
            }
            // Parser needs more data.
            Err(ResponseDecodeError::Incomplete) => {
                // Read more data.
                read_more(&mut buffer, Role::Server);
            }
            // Parser needs more data.
            //
            // A client MUST receive any literal and can't reject it. However, if the literal is too
            // large, the client would have the (semi-optimal) option to still *read it* but discard
            // the data chunk by chunk. It could also close the connection. This is why we have this
            // option.
            Err(ResponseDecodeError::LiteralFound { .. }) => {
                // Read more data.
                read_more(&mut buffer, Role::Server);
            }
            // Parser failed.
            Err(ResponseDecodeError::Failed) => {
                println!("Error parsing response.");
                println!("Clearing buffer.");

                // Clear the buffer and proceed with loop.
                buffer.clear();
            }
        }
    }
}
