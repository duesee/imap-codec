#![no_main]

use bounded_static::ToBoundedStatic;
use imap_types::{command::Command, response::Response};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|tuple: (Command, Response)| {
    let (cmd, rsp) = tuple;

    let got = cmd.to_static();
    assert_eq!(cmd, got);

    let got = rsp.to_static();
    assert_eq!(rsp, got);
});
