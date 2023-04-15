#![no_main]

#[cfg(any(feature = "ext_login_referrals", feature = "ext_mailbox_referrals"))]
use imap_codec::response::Code;
#[cfg(feature = "debug")]
use imap_codec::utils::escape_byte_string;
use imap_codec::{
    codec::{Decode, Encode},
    response::Greeting,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|test: Greeting| {
    // TODO(#30): Skip certain generations for now as we know they need to be fixed.
    //            The goal is to not skip anything eventually.
    #[cfg(any(feature = "ext_login_referrals", feature = "ext_mailbox_referrals"))]
    if let Some(Code::Referral(_)) = test.code {
        // FIXME(#30)
        return;
    }

    #[cfg(feature = "debug")]
    println!("[!] Input: {test:?}");

    let buffer = test.encode_detached().unwrap();

    #[cfg(feature = "debug")]
    println!("[!] Serialized: {}", escape_byte_string(&buffer));

    let (rem, parsed) = Greeting::decode(&buffer).unwrap();
    assert!(rem.is_empty());

    #[cfg(feature = "debug")]
    println!("[!] Parsed: {parsed:?}");

    assert_eq!(test, parsed);

    #[cfg(feature = "debug")]
    println!("{}", str::repeat("-", 120));
});
