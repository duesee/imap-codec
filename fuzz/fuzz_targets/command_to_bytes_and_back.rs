#![no_main]

#[cfg(feature = "ext_enable")]
use imap_codec::message::CapabilityEnable;
#[cfg(feature = "debug")]
use imap_codec::utils::escape_byte_string;
use imap_codec::{
    codec::{Decode, Encode},
    command::{search::SearchKey, Command, CommandBody},
};
use libfuzzer_sys::fuzz_target;

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
    // TODO(#30): Skip certain generations for now as we know they need to be fixed.
    //            The goal is to not skip anything eventually.
    match test.body {
        CommandBody::Search { ref criteria, .. } if ignore_search_key_and(criteria) => {
            // FIXME(#30)
            return;
        }
        #[cfg(feature = "ext_enable")]
        CommandBody::Enable {
            ref capabilities, ..
        } if ignore_capabilities_enable(capabilities.as_ref()) => {
            // FIXME(#30)
            return;
        }
        _ => {}
    }

    #[cfg(feature = "debug")]
    println!("[!] Input: {test:?}");

    let buffer = test.encode_detached().unwrap();

    #[cfg(feature = "debug")]
    println!("[!] Serialized: {}", escape_byte_string(&buffer));

    match Command::decode(&buffer) {
        Ok((rem, parsed)) => {
            assert!(rem.is_empty());

            #[cfg(feature = "debug")]
            println!("[!] Parsed: {parsed:?}");

            assert_eq!(test, parsed)
        }
        Err(error) => {
            // TODO: Signal recursion limit?
            // Previously the nom code `nom::error::ErrorKind::TooLarge` signaled
            // an exceeded recursion limit. Should the API signal it, too?
            panic!("Could not parse produced object. Error: {:?}", error);
        }
    }

    #[cfg(feature = "debug")]
    println!("{}", str::repeat("-", 120));
});
