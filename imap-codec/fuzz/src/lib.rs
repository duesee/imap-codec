#[macro_export]
macro_rules! impl_decode_target {
    ($decoder:ident) => {
        use libfuzzer_sys::fuzz_target;

        fuzz_target!(|input: &[u8]| {
            use imap_codec::codec::{Decoder, Encode};
            #[cfg(feature = "debug")]
            use imap_codec::imap_types::utils::escape_byte_string;

            #[cfg(feature = "debug")]
            println!("[!] Input:      {}", escape_byte_string(input));

            if let Ok((_rem, parsed1)) = $decoder::decode(input) {
                #[cfg(feature = "debug")]
                {
                    let input = &input[..input.len() - _rem.len()];
                    println!("[!] Consumed:   {}", escape_byte_string(input));
                    println!("[!] Parsed1: {parsed1:?}");
                }

                let output = parsed1.encode().dump();
                #[cfg(feature = "debug")]
                println!("[!] Serialized: {}", escape_byte_string(&output));

                let (rem, parsed2) = $decoder::decode(&output).unwrap();
                #[cfg(feature = "debug")]
                println!("[!] Parsed2: {parsed2:?}");
                assert!(rem.is_empty());

                assert_eq!(parsed1, parsed2);

                /*
                #[cfg(feature = "split")]
                {
                    // Check splits ...
                    #[cfg(feature = "debug")]
                    println!("[!] Full: {}", escape_byte_string(&output));
                    #[cfg(feature = "debug")]
                    println!("[!] Full: {parsed2:?}");

                    for index in 0..=output.len() {
                        let partial = &output[..index];
                        #[cfg(feature = "debug")]
                        println!("[!] Split (..{index:>3}): {}", escape_byte_string(partial));
                        match <$decoder>::decode(partial) {
                            Ok((rem, out)) => {
                                assert!(rem.is_empty());
                                assert_eq!(index, output.len());
                                print!("\r{index}");
                            }
                            Err(error) => match error {
                                DecodeError::Incomplete => {
                                    assert!(index < output.len());
                                }
                                DecodeError::LiteralFound { .. } => {
                                    assert!(index < output.len());
                                }
                                DecodeError::Failed => {
                                    panic!("Expected `Ok` or `Incomplete`, got `Failed`");
                                }
                            },
                        }
                    }
                }
                */
            } else {
                #[cfg(feature = "debug")]
                println!("[!] <invalid>");
            }

            #[cfg(feature = "debug")]
            println!("{}", str::repeat("-", 120));
        });
    };
}

#[macro_export]
macro_rules! impl_to_bytes_and_back {
    ($decoder:tt, $object:tt) => {
        use libfuzzer_sys::fuzz_target;

        fuzz_target!(|input: $object| {
            use imap_codec::codec::{Decoder, Encode};
            #[cfg(feature = "debug")]
            use imap_codec::imap_types::utils::escape_byte_string;

            #[cfg(feature = "debug")]
            println!("[!] Input:  {:?}", input);

            let buffer = input.encode().dump();

            #[cfg(feature = "debug")]
            println!("[!] Serialized: {}", escape_byte_string(&buffer));

            let (rem, parsed) = <$decoder>::decode(&buffer).unwrap();
            assert!(rem.is_empty());

            #[cfg(feature = "debug")]
            println!("[!] Parsed: {parsed:?}");

            assert_eq!(input, parsed);

            #[cfg(feature = "debug")]
            println!("{}", str::repeat("-", 120));
        });
    };
}
