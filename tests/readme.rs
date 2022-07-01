use imap_codec::{
    codec::{Decode, Encode},
    types::command::Command,
};

#[test]
fn test_from_readme() {
    let input = b"ABCD UID FETCH 1,2:* (BODY.PEEK[1.2.3.4.MIME]<42.1337>)\r\n";

    let (_remainder, parsed) = Command::decode(input).unwrap();
    println!("// Parsed:");
    println!("{:#?}", parsed);

    let mut serialized = Vec::new();
    parsed.encode(&mut serialized).unwrap(); // This could be send over the network.

    let serialized = String::from_utf8(serialized).unwrap(); // Not every IMAP message is valid UTF-8.
    println!("// Serialized:"); // We just ignore that, so that we can print the message.
    println!("// {}", serialized);
}
