#![no_main]

#[cfg(feature = "debug")]
use imap_codec::utils::escape_byte_string;
use imap_codec::{
    codec::{Decode, Encode},
    command::Command,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|test: Command| {
    #[cfg(feature = "debug")]
    println!("[!] Input: {test:?}");

    let buffer = test.encode_detached().unwrap();

    #[cfg(feature = "debug")]
    println!("[!] Serialized: {}", escape_byte_string(&buffer));

    match Command::decode(&buffer) {
        Ok((rem, parsed)) => {
            assert!(rem.is_empty());

            #[cfg(feature = "debug")]
            println!("[!] Parsed: {parsed:?}");

            assert_eq!(test, parsed)
        }
        Err(error) => {
            // TODO: Signal recursion limit?
            // Previously the nom code `nom::error::ErrorKind::TooLarge` signaled
            // an exceeded recursion limit. Should the API signal it, too?
            panic!("Could not parse produced object. Error: {:?}", error);
        }
    }

    #[cfg(feature = "debug")]
    println!("{}", str::repeat("-", 120));
});
