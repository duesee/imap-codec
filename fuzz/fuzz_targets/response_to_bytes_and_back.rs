#![no_main]

use base64::{engine::general_purpose::STANDARD as _base64, Engine};
#[cfg(feature = "debug")]
use imap_codec::utils::escape_byte_string;
use imap_codec::{
    codec::{Context, Decode, Encode},
    response::{data::FetchAttributeValue, Code, Continue, Data, Response},
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
                    _ => {}
                }
            }
        }
        Response::Continue(ref continue_request) => match continue_request {
            Continue::Basic {
                code: Some(code), ..
            } => match code {
                Code::PermanentFlags(_) => {
                    // FIXME(#30): Flag handling.
                    return;
                }
                #[cfg(any(feature = "ext_login_referrals", feature = "ext_mailbox_referrals"))]
                Code::Referral(_) => {
                    // FIXME(#30)
                    return;
                }
                _ => {}
            },
            // Oh, IMAP :-/
            Continue::Basic { code: None, text } => {
                if _base64.decode(text.inner()).is_ok() {
                    // FIXME(#30): Flag handling.
                    return;
                }
            }
            _ => {}
        },
    }

    #[cfg(feature = "debug")]
    println!("[!] Input: {test:?}");

    let buffer = test.encode_detached(&Context::default()).unwrap();

    #[cfg(feature = "debug")]
    println!("[!] Serialized: {}", escape_byte_string(&buffer));

    let (rem, parsed) = Response::decode(&buffer).unwrap();
    assert!(rem.is_empty());

    #[cfg(feature = "debug")]
    println!("[!] Parsed: {parsed:?}");

    assert_eq!(test, parsed);

    #[cfg(feature = "debug")]
    println!("{}", str::repeat("-", 120));
});
