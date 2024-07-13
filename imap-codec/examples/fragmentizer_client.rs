use std::{io::Read, net::TcpStream};

use imap_codec::{fragmentizer::Fragmentizer, GreetingCodec, ResponseCodec};

enum State {
    Greeting,
    Response,
}

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:12345").unwrap();
    let mut fragmentizer = Fragmentizer::new(1024);

    let mut state = State::Greeting;

    loop {
        match fragmentizer.progress() {
            Some(fragment_info) => {
                dbg!(fragment_info);
                dbg!(fragmentizer.fragment_bytes(fragment_info));

                if fragmentizer.is_message_complete() {
                    match state {
                        State::Greeting => {
                            match fragmentizer.decode_message(&GreetingCodec::new()) {
                                Ok(greeting) => {
                                    dbg!(greeting);
                                    state = State::Response;
                                }
                                Err(error) => {
                                    dbg!(error);
                                }
                            }
                        }
                        State::Response => {
                            match fragmentizer.decode_message(&ResponseCodec::new()) {
                                Ok(response) => {
                                    dbg!(response);
                                }
                                Err(error) => {
                                    dbg!(error);
                                }
                            }
                        }
                    }
                }
            }
            None => {
                println!("Reading bytes...");
                let mut buffer = [0; 64];
                let count = dbg!(stream.read(&mut buffer).unwrap());
                if count == 0 {
                    println!("<Connection closed>");
                    break;
                }

                fragmentizer.enqueue_bytes(&buffer[..count]);
            }
        }
    }
}
