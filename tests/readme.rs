use imap_codec::{
    codec::{Decode, Encode},
    command::Command,
};

#[test]
fn test_from_readme() {
    let input = b"ABCD UID FETCH 1,2:* (BODY.PEEK[1.2.3.4.MIME]<42.1337>)\r\n";

    let (_remainder, parsed) = Command::decode(input).unwrap();
    println!("Parsed:\n{:#?}\n", parsed);

    let mut buffer = Vec::new();
    parsed.encode(&mut buffer).unwrap(); // This could be send over the network.

    // Note: Not every IMAP message is valid UTF-8.
    //       We ignore that here to print the message.
    println!("Serialized:\n{}", String::from_utf8(buffer).unwrap());
}
