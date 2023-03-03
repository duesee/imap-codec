#![no_main]

// use std::str::from_utf8;

use imap_codec::{
    codec::{Decode, Encode},
    response::{Code, Greeting},
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|test: Greeting| {
    // TODO(#30): Skip certain generations for now as we know they need to be fixed.
    //            The goal is to not skip anything eventually.
    match test.code {
        Some(ref code) => match code {
            Code::PermanentFlags(_) => {
                // FIXME(#30)
                return;
            }
            #[cfg(any(feature = "ext_login_referrals", feature = "ext_mailbox_referrals"))]
            Code::Referral(_) => {
                // FIXME(#30)
                return;
            }
            Code::Other(_, _) => {
                // FIXME(#30)
                return;
            }
            _ => {}
        },
        _ => {}
    }

    #[cfg(feature = "debug")]
    println!("[!] Input: {test:?}");

    let mut buffer = Vec::new();
    test.encode(&mut buffer).unwrap();

    #[cfg(feature = "debug")]
    match std::str::from_utf8(&buffer) {
        Ok(str) => println!("[!] Serialized: {str}"),
        Err(_) => println!("[!] Serialized: {buffer:?}"),
    }

    let (rem, parsed) = Greeting::decode(&buffer).unwrap();
    assert!(rem.is_empty());

    #[cfg(feature = "debug")]
    println!("[!] Parsed: {parsed:?}");

    assert_eq!(test, parsed);

    #[cfg(feature = "debug")]
    println!("{}", str::repeat("-", 120));
});
