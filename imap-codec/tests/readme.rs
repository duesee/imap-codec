use imap_codec::{decode::Decoder, encode::Encoder, CommandCodec};

#[test]
fn test_from_readme() {
    let input = b"ABCD UID FETCH 1,2:* (BODY.PEEK[1.2.3.4.MIME]<42.1337>)\r\n";

    let (_remainder, parsed) = CommandCodec::decode(input).unwrap();
    println!("# Parsed\n\n{:#?}\n\n", parsed);

    let buffer = CommandCodec::default().encode(&parsed).dump();

    // Note: IMAP4rev1 may produce messages that are not valid UTF-8.
    println!("# Serialized\n\n{:?}", std::str::from_utf8(&buffer));
}
