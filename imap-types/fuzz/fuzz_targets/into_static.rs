#![no_main]

use bounded_static::IntoBoundedStatic;
use imap_types::{command::Command, response::Response};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|tuple: (Command, Response)| {
    let (cmd, rsp) = tuple;

    let got = cmd.clone().into_static();
    assert_eq!(cmd, got);

    let got = rsp.clone().into_static();
    assert_eq!(rsp, got);
});
