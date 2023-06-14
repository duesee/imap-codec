#![no_main]

use imap_codec::command::Command;
use imap_codec_fuzz::impl_to_bytes_and_back;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: Command| {
    impl_to_bytes_and_back!(input, Command);
});
