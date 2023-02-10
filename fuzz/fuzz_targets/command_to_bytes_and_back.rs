#![no_main]

#[cfg(feature = "ext_enable")]
use imap_codec::message::CapabilityEnable;
use imap_codec::{
    codec::{Decode, Encode},
    command::{search::SearchKey, Command, CommandBody},
    message::Flag,
};
use libfuzzer_sys::fuzz_target;

fn ignore_flags(flags: &[Flag]) -> bool {
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
        .any(|capability| matches!(capability, CapabilityEnable::Other(_)))
}

fn ignore_search_key_and(sk: &SearchKey) -> bool {
    match sk {
        SearchKey::And(list) => match list.as_ref().len() {
            1 => true,
            _ => list.as_ref().iter().any(ignore_search_key_and),
        },
        SearchKey::Not(sk) => ignore_search_key_and(sk),
        SearchKey::Or(sk1, sk2) => ignore_search_key_and(sk1) || ignore_search_key_and(sk2),
        _ => false,
    }
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
        CommandBody::Search { criteria, .. } if ignore_search_key_and(criteria) => {
            // FIXME(#30)
        }
        #[cfg(feature = "ext_enable")]
        CommandBody::Enable { capabilities, .. }
            if ignore_capabilities_enable(capabilities.as_ref()) =>
        {
            // FIXME(#30)
        }
        _ => {
            match Command::decode(&buffer) {
                Ok((rem, parsed)) => {
                    assert!(rem.is_empty());

                    //println!("{:?}", parsed);

                    assert_eq!(test, parsed)
                }
                Err(error) => {
                    // TODO: Signal recursion limit?
                    // Previously the nom code `nom::error::ErrorKind::TooLarge` signaled
                    // an exceeded recursion limit. Should the API signal it, too?
                    panic!("Could not parse produced object. Error: {:?}", error);
                }
            }
        }
    }

    //println!("{}", str::repeat("-", 120));
});
