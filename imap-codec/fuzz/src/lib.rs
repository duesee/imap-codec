#[macro_export]
macro_rules! impl_decode_target {
    ($codec:ident) => {
        use libfuzzer_sys::fuzz_target;

        fuzz_target!(|input: &[u8]| {
            #[cfg(feature = "debug")]
            use imap_codec::imap_types::utils::escape_byte_string;
            use imap_codec::{decode::Decoder, encode::Encoder};

            #[cfg(feature = "debug")]
            println!("[!] Input:      {}", escape_byte_string(input));

            if let Ok((_rem, parsed1)) = $codec::default().decode(input) {
                #[cfg(feature = "debug")]
                {
                    let input = &input[..input.len() - _rem.len()];
                    println!("[!] Consumed:   {}", escape_byte_string(input));
                    println!("[!] Parsed1: {parsed1:?}");
                }

                let output = $codec::default().encode(&parsed1).dump();
                #[cfg(feature = "debug")]
                println!("[!] Serialized: {}", escape_byte_string(&output));

                let (rem, parsed2) = $codec::default().decode(&output).unwrap();
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
    ($codec:tt, $object:tt) => {
        use libfuzzer_sys::fuzz_target;

        fuzz_target!(|input: $object| {
            #[cfg(feature = "debug")]
            use imap_codec::imap_types::utils::escape_byte_string;
            use imap_codec::{decode::Decoder, encode::Encoder};

            #[cfg(feature = "debug")]
            println!("[!] Input:  {:?}", input);

            let buffer = <$codec>::default().encode(&input).dump();

            #[cfg(feature = "debug")]
            println!("[!] Serialized: {}", escape_byte_string(&buffer));

            let (rem, parsed) = <$codec>::decode(&buffer).unwrap();
            assert!(rem.is_empty());

            #[cfg(feature = "debug")]
            println!("[!] Parsed: {parsed:?}");

            assert_eq!(input, parsed);

            #[cfg(feature = "debug")]
            println!("{}", str::repeat("-", 120));
        });
    };
}
