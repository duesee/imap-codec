#![no_main]

#[cfg(feature = "debug")]
use imap_codec::utils::escape_byte_string;
use imap_codec::{
    codec::{Context, Decode, Encode},
    command::Command,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    #[cfg(feature = "debug")]
    println!("[!] Input: {}", escape_byte_string(data));

    if let Ok((_rem, parsed1)) = Command::decode(data) {
        #[cfg(feature = "debug")]
        {
            let input = &data[..data.len() - _rem.len()];
            println!("[!] Consumed: {}", escape_byte_string(input));
            println!("[!] Parsed1: {parsed1:?}");
        }

        let output = parsed1.encode_detached(&Context::default()).unwrap();
        #[cfg(feature = "debug")]
        println!("[!] Serialized: {}", escape_byte_string(&output));

        let (rem, parsed2) = Command::decode(&output).unwrap();
        #[cfg(feature = "debug")]
        println!("[!] Parsed2: {parsed2:?}");
        assert!(rem.is_empty());

        assert_eq!(parsed1, parsed2);
    } else {
        #[cfg(feature = "debug")]
        println!("[!] <invalid>");
    }

    #[cfg(feature = "debug")]
    println!("\n\n\n");
});
