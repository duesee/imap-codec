#![no_main]

use imap_types::{
    bounded_static::IntoBoundedStatic,
    command::Command,
    response::{Greeting, Response},
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|tuple: (Greeting, Command, Response)| {
    let (grt, cmd, rsp) = tuple;

    let got = grt.clone().into_static();
    assert_eq!(grt, got);

    let got = cmd.clone().into_static();
    assert_eq!(cmd, got);

    let got = rsp.clone().into_static();
    assert_eq!(rsp, got);
});
