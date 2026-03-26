//! Regression: Mail.ru FETCH lines must parse (Himalaya envelope list).
use imap_codec::{ResponseCodec, decode::Decoder};

/// `include_bytes!` may yield LF-only after Git checkout; IMAP uses CRLF.
fn normalize_mailru_fixture_crlf(raw: &[u8]) -> Vec<u8> {
    let payload = raw
        .strip_suffix(b"\r\n")
        .or_else(|| raw.strip_suffix(b"\n"))
        .unwrap_or(raw);
    let mut v = payload.to_vec();
    v.extend_from_slice(b"\r\n");
    v
}

#[test]
fn mailru_minimal_fetch_empty_address_name_parses() {
    // Minimal repro: empty quoted display name in To (Mail.ru).
    let line = concat!(
        "* 1 FETCH (UID 1 FLAGS () ",
        "ENVELOPE (NIL NIL NIL NIL NIL ",
        "((\"\" NIL \"user\" \"mail.ru\")) ",
        "NIL NIL NIL NIL))\r\n"
    );
    let (rem, _) = ResponseCodec::default().decode(line.as_bytes()).unwrap();
    assert!(rem.is_empty());
}

#[test]
fn mailru_fetch_with_mixed_alternative_bodystructure_parses() {
    // Captured from a real Mail.ru IMAP `FETCH (UID FLAGS ENVELOPE BODYSTRUCTURE)` line.
    static RAW: &[u8] = include_bytes!("fixtures_mailru_fetch_2829.bin");
    let line = normalize_mailru_fixture_crlf(RAW);
    let decoded = ResponseCodec::default().decode(line.as_slice());
    if let Err(e) = &decoded {
        eprintln!("decode error: {e:?}");
    }
    let (rem, _resp) = decoded.expect("Mail.ru FETCH should parse");
    assert!(
        rem.is_empty(),
        "unexpected trailing bytes: {:?}",
        String::from_utf8_lossy(rem)
    );
}
