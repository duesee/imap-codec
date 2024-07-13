use std::{io::Read, net::TcpListener};

use imap_codec::{fragmentizer::Fragmentizer, AuthenticateDataCodec, CommandCodec, IdleDoneCodec};
use imap_types::command::CommandBody;

enum State {
    Command,
    AuthenticateData,
    Idle,
}

fn main() {
    let mut stream = {
        let listener = TcpListener::bind("127.0.0.1:12345").unwrap();
        listener.accept().unwrap().0
    };

    let mut fragmentizer = Fragmentizer::new(1024);

    let mut state = State::Command;

    loop {
        match fragmentizer.progress() {
            Some(fragment_info) => {
                dbg!(fragment_info);
                dbg!(fragmentizer.fragment_bytes(fragment_info));

                if fragmentizer.is_message_complete() {
                    match state {
                        State::Command => match fragmentizer.decode_message(&CommandCodec::new()) {
                            Ok(command) => {
                                dbg!(&command);
                                state = match command.body {
                                    CommandBody::Authenticate { .. } => State::AuthenticateData,
                                    CommandBody::Idle => State::Idle,
                                    _ => State::Command,
                                };
                            }
                            Err(error) => {
                                dbg!(error);
                            }
                        },
                        State::AuthenticateData => {
                            match fragmentizer.decode_message(&AuthenticateDataCodec::new()) {
                                Ok(authenticate_data) => {
                                    dbg!(authenticate_data);
                                    // Pretend we are done after one SASL round.
                                    state = State::Command;
                                }
                                Err(error) => {
                                    dbg!(error);
                                }
                            }
                        }
                        State::Idle => match fragmentizer.decode_message(&IdleDoneCodec::new()) {
                            Ok(idle_done) => {
                                dbg!(idle_done);
                                state = State::Command;
                            }
                            Err(error) => {
                                dbg!(error);
                            }
                        },
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
