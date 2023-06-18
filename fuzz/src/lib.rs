#[macro_export]
macro_rules! impl_decode_target {
    ($object:ty) => {
        use libfuzzer_sys::fuzz_target;

        fuzz_target!(|input: &[u8]| {
            use imap_codec::codec::{Decode, Encode};
            #[cfg(feature = "debug")]
            use imap_codec::utils::escape_byte_string;

            #[cfg(feature = "debug")]
            println!("[!] Input: {}", escape_byte_string(input));

            if let Ok((_rem, parsed1)) = <$object>::decode(input) {
                #[cfg(feature = "debug")]
                {
                    let input = &input[..input.len() - _rem.len()];
                    println!("[!] Consumed: {}", escape_byte_string(input));
                    println!("[!] Parsed1: {parsed1:?}");
                }

                let output = parsed1.encode().dump();
                #[cfg(feature = "debug")]
                println!("[!] Serialized: {}", escape_byte_string(&output));

                let (rem, parsed2) = <$object>::decode(&output).unwrap();
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
    };
}

#[macro_export]
macro_rules! impl_to_bytes_and_back {
    ($object:tt) => {
        use libfuzzer_sys::fuzz_target;

        fuzz_target!(|input: $object| {
            use imap_codec::codec::{Decode, Encode};
            #[cfg(feature = "debug")]
            use imap_codec::utils::escape_byte_string;

            #[cfg(feature = "debug")]
            println!("[!] Input: {:?}", input);

            let buffer = input.encode().dump();

            #[cfg(feature = "debug")]
            println!("[!] Serialized: {}", escape_byte_string(&buffer));

            let (rem, parsed) = <$object>::decode(&buffer).unwrap();
            assert!(rem.is_empty());

            #[cfg(feature = "debug")]
            println!("[!] Parsed: {parsed:?}");

            assert_eq!(input, parsed);

            #[cfg(feature = "debug")]
            println!("{}", str::repeat("-", 120));
        });
    };
}
