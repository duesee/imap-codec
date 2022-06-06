#![no_main]

#[cfg(feature = "ext_enable")]
use imap_codec::extensions::rfc5161::CapabilityEnable;
use imap_codec::{
    command::command,
    types::{
        codec::Encode,
        command::{Command, CommandBody},
        flag::Flag,
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

#[cfg(feature = "ext_enable")]
fn ignore_capabilities_enable(capabilities: &[CapabilityEnable]) -> bool {
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
            // FIXME(#30)
        }
        CommandBody::Append { flags, .. } if ignore_flags(flags) => {
            // FIXME(#30)
        }
        #[cfg(feature = "ext_enable")]
        CommandBody::Enable { capabilities, .. } if ignore_capabilities(capabilities) => {
            // FIXME(#30)
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
