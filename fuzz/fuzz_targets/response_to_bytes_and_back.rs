#![no_main]

use std::str::from_utf8;

use imap_codec::{
    codec::{Decode, Encode},
    response::{Code, Data, Response, Status},
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|test: Response| {
    if matches!(test, Response::Continue(_)) {
        // FIXME(#30)
        return;
    }

    if matches!(test, Response::Data(Data::Flags(..))) {
        // FIXME(#30)
        return;
    }

    if matches!(test, Response::Data(Data::List { .. })) {
        // FIXME(#30)
        return;
    }

    if matches!(test, Response::Data(Data::Lsub { .. })) {
        // FIXME(#30)
        return;
    }

    if matches!(test, Response::Status(
        Status::Ok { ref code, .. } |
        Status::No { ref code, .. } |
        Status::Bad { ref code, .. } |
        Status::Bye{ ref code, .. }) if matches!(code, Some(Code::Referral(_))))
    {
        // FIXME(#30)
        return;
    }

    println!("{:?}", test);

    let mut buffer = Vec::new();
    test.encode(&mut buffer).unwrap();

    //match std::str::from_utf8(&buffer) {
    //    Ok(str) => println!("{}", str),
    //    Err(_) => println!("{:?}", buffer),
    //}

    println!("{:?}", from_utf8(&buffer));

    let (rem, parsed) = Response::decode(&buffer).unwrap();
    assert!(rem.is_empty());

    println!("{:?}", parsed);

    assert_eq!(test, parsed);

    println!("{}", str::repeat("-", 120));
});
