#![no_main]

use base64::{engine::general_purpose::STANDARD as _base64, Engine};
#[cfg(feature = "debug")]
use imap_codec::utils::escape_byte_string;
use imap_codec::{
    codec::{Decode, Encode},
    response::{data::FetchAttributeValue, Continue, Data, Response},
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|test: Response| {
    // TODO(#30): Skip certain generations for now as we know they need to be fixed.
    //            The goal is to not skip anything eventually.
    match test {
        Response::Data(Data::Fetch { ref attributes, .. }) => {
            for attribute in attributes.as_ref().iter() {
                match attribute {
                    FetchAttributeValue::Body(_) | FetchAttributeValue::BodyStructure(_) => {
                        // FIXME(#30): Body(Structure)
                        return;
                    }
                    _ => {}
                }
            }
        }
        Response::Continue(Continue::Basic {
            code: None,
            ref text,
        }) => {
            // Oh, IMAP :-/
            if _base64.decode(text.inner()).is_ok() {
                // FIXME(#30)
                return;
            }
        }
        _ => {}
    }

    #[cfg(feature = "debug")]
    println!("[!] Input: {test:?}");

    let buffer = test.encode_detached().unwrap();

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
