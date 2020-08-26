#![no_main]
use libfuzzer_sys::fuzz_target;

use imap_proto_server::parse::response::response;

fuzz_target!(|data: &[u8]| {
    if let Ok((_, parsed)) = response(data) {
        //println!("# {}", String::from_utf8_lossy(&parsed.serialize()).trim()); TODO: NIY
        println!("{:?}\n\n", parsed);
    }
});
