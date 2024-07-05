#![no_main]

use imap_types::{
    command::Command,
    response::{Greeting, Response},
    ToStatic,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|tuple: (Greeting, Command, Response)| {
    let (grt, cmd, rsp) = tuple;

    let got = grt.to_static();
    assert_eq!(grt, got);

    let got = cmd.to_static();
    assert_eq!(cmd, got);

    let got = rsp.to_static();
    assert_eq!(rsp, got);
});
