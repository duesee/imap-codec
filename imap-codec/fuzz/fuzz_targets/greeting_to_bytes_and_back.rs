#![no_main]

use imap_codec::{imap_types::response::Greeting, GreetingCodec};
use imap_codec_fuzz::impl_to_bytes_and_back;

impl_to_bytes_and_back!(GreetingCodec, Greeting);
