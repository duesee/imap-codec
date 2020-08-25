#![no_main]
use libfuzzer_sys::fuzz_target;

use imap_proto_server::{
    //codec::Codec,
    parse::command::command,
};

fuzz_target!(|data: &[u8]| {
    if let Ok((_, parsed)) = command(data) {
        //println!("# {}", String::from_utf8_lossy(&parsed.serialize()));
        println!("{:?}", parsed);
    }
});
