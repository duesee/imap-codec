#![no_main]

use imap_codec::auth::AuthenticateData;
use imap_codec_fuzz::impl_decode_target;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    impl_decode_target!(AuthenticateData, data);
});
