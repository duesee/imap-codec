//! Fuzzes the message attributes of a FETCH response, where the Gmail
//! (`X-GM-*`) and OBJECTID (`EMAILID`, `THREADID`) parsers live. The input is
//! wrapped as `* 1 FETCH (<input>)\r\n` so mutations stay inside the
//! attribute list instead of breaking the response framing.
//!
//! Property: anything that decodes must encode to bytes that decode to the
//! same value.
//!
//! Run with `cargo +nightly fuzz run fetch_msg_att --features ext_gmail,ext_objectid
//! fuzz/seeds/fetch_msg_att -- -dict=fuzz/fetch_msg_att.dict` from `imap-codec/`.
#![no_main]

use imap_codec::{ResponseCodec, decode::Decoder, encode::Encoder};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|attributes: &[u8]| {
    let mut input = b"* 1 FETCH (".to_vec();
    input.extend_from_slice(attributes);
    input.extend_from_slice(b")\r\n");

    if let Ok((_, parsed1)) = ResponseCodec::default().decode(&input) {
        let output = ResponseCodec::default().encode(&parsed1).dump();
        let (rem, parsed2) = ResponseCodec::default()
            .decode(&output)
            .expect("re-encoded response must decode");
        assert!(rem.is_empty());
        assert_eq!(parsed1, parsed2);
    }
});
