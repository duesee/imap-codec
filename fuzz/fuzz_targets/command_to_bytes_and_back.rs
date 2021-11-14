#![no_main]

use imap_codec::{
    codec::Encode,
    parse::command::command,
    types::{
        command::{Command, CommandBody},
        flag::Flag,
        response::Capability,
    },
};
use libfuzzer_sys::fuzz_target;

fn ignore_flags(flags: &Vec<Flag>) -> bool {
    flags.iter().any(|flag| {
        matches!(
            flag,
            Flag::Keyword(_)
                | Flag::Extension(_)
                | Flag::Permanent
                | Flag::Recent
                | Flag::NameAttribute(_)
        )
    })
}

fn ignore_capabilities(capabilities: &Vec<Capability>) -> bool {
    capabilities
        .iter()
        .any(|capability| matches!(capability, Capability::Other(_)))
}

fuzz_target!(|test: Command| {
    //println!("{:?}", test);

    let mut buffer = Vec::new();
    test.encode(&mut buffer).unwrap();

    //match std::str::from_utf8(&buffer) {
    //    Ok(str) => println!("{}", str),
    //    Err(_) => println!("{:?}", buffer),
    //}

    //println!("{:?}", std::str::from_utf8(&buffer));

    match &test.body {
        CommandBody::Store { flags, .. } if ignore_flags(flags) => {
            // TODO
        }
        CommandBody::Append { flags, .. } if ignore_flags(flags) => {
            // TODO
        }
        CommandBody::Enable { capabilities, .. } if ignore_capabilities(capabilities) => {
            // TODO
        }
        _ => {
            let (rem, parsed) = command(&buffer).unwrap();
            assert!(rem.is_empty());

            //println!("{:?}", parsed);

            assert_eq!(test, parsed)
        }
    }

    //println!("{}", str::repeat("-", 120));
});
