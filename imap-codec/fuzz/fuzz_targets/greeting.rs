#![no_main]

use imap_codec::response::Greeting;
use imap_codec_fuzz::impl_decode_target;

impl_decode_target!(Greeting);
