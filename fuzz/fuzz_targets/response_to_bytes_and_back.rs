#![no_main]

#[cfg(feature = "debug")]
use std::str::from_utf8;

#[cfg(any(feature = "ext_login_referrals", feature = "ext_mailbox_referrals"))]
use imap_codec::response::Status;
use imap_codec::{
    codec::{Decode, Encode},
    response::{data::FetchAttributeValue, Code, Data, Response},
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|test: Response| {
    // TODO(#30): Skip certain generations for now as we know they need to be fixed.
    //            The goal is to not skip anything eventually.
    match test {
        Response::Data(ref data) => match data {
            Data::Fetch { ref attributes, .. } => {
                for attribute in attributes.as_ref().iter() {
                    match attribute {
                        FetchAttributeValue::Body(_) | FetchAttributeValue::BodyStructure(_) => {
                            // FIXME(#30): Body(Structure)
                            return;
                        }
                        FetchAttributeValue::Flags(_) => {
                            // FIXME(#30): Flag handling.
                            return;
                        }
                        _ => {}
                    }
                }
            }
            Data::List { items, .. } | Data::Lsub { items, .. } if !items.is_empty() => {
                // FIXME(#30): Flag handling.
                return;
            }
            Data::Flags(_) => {
                // FIXME(#30): Flag handling.
                return;
            }
            _ => {}
        },
        Response::Status(ref status) => {
            if let Some(ref code) = status.code() {
                match code {
                    Code::PermanentFlags(_) => {
                        // FIXME(#30): Flag handling.
                        return;
                    }
                    #[cfg(any(feature = "ext_login_referrals", feature = "ext_mailbox_referrals"))]
                    Code::Referral(_) => {
                        // FIXME(#30)
                        return;
                    }
                    Code::Other(_, _) => {
                        // FIXME(#30)
                        return;
                    }
                    _ => {}
                }
            }
        }
        Response::Continue(_) => {
            // FIXME(#30)
            return;
        }
    }

    #[cfg(feature = "debug")]
    println!("[!] Input: {test:?}");

    let mut buffer = Vec::new();
    test.encode(&mut buffer).unwrap();

    #[cfg(feature = "debug")]
    match from_utf8(&buffer) {
        Ok(str) => println!("[!] Serialized: {str}"),
        Err(_) => println!("[!] Serialized: {buffer:?}"),
    }

    let (rem, parsed) = Response::decode(&buffer).unwrap();
    assert!(rem.is_empty());

    #[cfg(feature = "debug")]
    println!("[!] Parsed: {parsed:?}");

    assert_eq!(test, parsed);

    #[cfg(feature = "debug")]
    println!("{}", str::repeat("-", 120));
});
