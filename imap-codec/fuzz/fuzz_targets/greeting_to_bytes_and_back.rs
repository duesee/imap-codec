#![no_main]

use imap_codec::{codec::GreetingCodec, imap_types::response::Greeting};
use imap_codec_fuzz::impl_to_bytes_and_back;

impl_to_bytes_and_back!(GreetingCodec, Greeting);
