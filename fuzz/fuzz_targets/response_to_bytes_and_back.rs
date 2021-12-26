#![no_main]

use imap_codec::{
    codec::Encode,
    parse::response::response,
    types::response::{Status, Response, Data},
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|test: Response| {
    if matches!(test, Response::Continuation(_)) {
        // TODO
        return;
    }

    if matches!(test, Response::Status(Status::PreAuth { .. })) {
        // TODO
        return;
    }

    if matches!(test, Response::Data(Data::Flags(..))) {
        // TODO
        return;
    }

    if matches!(test, Response::Data(Data::List { .. })) {
        // TODO
        return;
    }

    if matches!(test, Response::Data(Data::Lsub { .. })) {
        // TODO
        return;
    }

    println!("{:?}", test);

    let mut buffer = Vec::new();
    test.encode(&mut buffer).unwrap();

    //match std::str::from_utf8(&buffer) {
    //    Ok(str) => println!("{}", str),
    //    Err(_) => println!("{:?}", buffer),
    //}

    println!("{:?}", std::str::from_utf8(&buffer));

    let (rem, parsed) = response(&buffer).unwrap();
    assert!(rem.is_empty());

    println!("{:?}", parsed);

    assert_eq!(test, parsed);

    println!("{}", str::repeat("-", 120));
});
