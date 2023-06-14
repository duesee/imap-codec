#![no_main]

use imap_codec::auth::AuthenticateData;
use imap_codec_fuzz::impl_to_bytes_and_back;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: AuthenticateData| {
    impl_to_bytes_and_back!(input, AuthenticateData);
});
