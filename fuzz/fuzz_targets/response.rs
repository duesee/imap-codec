#![no_main]
use libfuzzer_sys::fuzz_target;

use imap_proto_server::parse::response::response;

fuzz_target!(|data: &[u8]| {
    if let Ok((_rem, parsed)) = response(data) {
        println!("{:?}", parsed);
    }
});
