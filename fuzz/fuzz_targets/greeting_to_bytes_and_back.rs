#![no_main]

// use std::str::from_utf8;

use imap_codec::{
    codec::{Decode, Encode},
    response::{Code, Greeting},
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|test: Greeting| {
    if matches!(&test, Greeting { code, .. } if matches!(code, Some(Code::Referral(_)))) {
        // FIXME(#30)
        return;
    }

    if matches!(&test, Greeting { code, .. } if matches!(code, Some(Code::PermanentFlags(_)| Code::Other(_, _))))
    {
        // FIXME(#30)
        return;
    }

    if let Some(first) = test.text.inner().chars().next() {
        if first == '[' {
            // FIXME(#30)
            return;
        }
    }

    // println!("{:?}", test);

    let mut buffer = Vec::new();
    test.encode(&mut buffer).unwrap();

    // match std::str::from_utf8(&buffer) {
    //     Ok(str) => println!("{}", str),
    //     Err(_) => println!("{:?}", buffer),
    // }

    // println!("{:?}", from_utf8(&buffer));

    let (rem, parsed) = Greeting::decode(&buffer).unwrap();
    assert!(rem.is_empty());

    // println!("{:?}", parsed);

    assert_eq!(test, parsed);

    // println!("{}", str::repeat("-", 120));
});
