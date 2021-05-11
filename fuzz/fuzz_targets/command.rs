#![no_main]
use libfuzzer_sys::fuzz_target;

use imap_codec::{codec::Encode, parse::command::command};

fuzz_target!(|data: &[u8]| {
    if let Ok((_rem, parsed1)) = command(data) {
        //let input = &data[..data.len() - rem.len()];

        //println!("libFuzzer:  {}", String::from_utf8_lossy(input).trim());
        //println!("parsed:     {:?}", parsed1);

        let mut input = Vec::with_capacity(data.len() * 2);
        parsed1.encode(&mut input).unwrap();
        //println!("serialized: {}", String::from_utf8_lossy(&input).trim());
        let (rem, parsed2) = command(&input).unwrap();
        //println!("parsed:     {:?}", parsed2);
        assert!(rem.is_empty());

        assert_eq!(parsed1, parsed2);

        //println!("\n\n\n");
    }
});
