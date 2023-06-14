#![no_main]

use imap_codec::response::Greeting;
use imap_codec_fuzz::impl_to_bytes_and_back;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: Greeting| {
    impl_to_bytes_and_back!(input, Greeting);
});
