use futures::{SinkExt, StreamExt};
use imap_codec::{
    tokio_compat::{Action, ImapServerCodec, Outcome},
    types::{
        api::response::{Continue, Response, Status},
        command::CommandBody,
    },
};
use tokio::{self, net::TcpListener};
use tokio_util::codec::Decoder;

#[tokio::main]
async fn main() {
    let server = async {
        let stream = {
            // Bind listener.
            let listener = TcpListener::bind("127.0.0.1:14300").await.unwrap();

            // Accept a single connection.
            let (stream, _) = listener.accept().await.unwrap();

            stream
        };

        let mut framed = {
            // Accept 2 MiB for literals
            let mib2 = 2 * 1024 * 1024;

            ImapServerCodec::new(mib2).framed(stream)
        };

        // Send OK greeting.
        let greeting = Response::Status(Status::ok(None, None, "Hello, World!").unwrap());
        framed.send(&greeting).await.unwrap();
        println!("S: {:?}", greeting);

        loop {
            match framed.next().await {
                Some(Ok(Outcome::Command(cmd))) => {
                    println!("C: {:?}", cmd);

                    match (cmd.tag, cmd.body) {
                        (
                            tag,
                            CommandBody::Login {
                                username: _,
                                password: _,
                            },
                        ) => {
                            let rsp = Response::Status(
                                Status::ok(Some(tag), None, "LOGIN done").unwrap(),
                            );
                            framed.send(&rsp).await.unwrap();
                            println!("S: {:?}", rsp);
                        }
                        (tag, CommandBody::Logout) => {
                            let rsp = Response::Status(Status::bye(None, "...").unwrap());
                            framed.send(&rsp).await.unwrap();
                            println!("S: {:?}", rsp);

                            let rsp = Response::Status(
                                Status::ok(Some(tag), None, "LOGOUT done").unwrap(),
                            );
                            framed.send(&rsp).await.unwrap();
                            println!("S: {:?}", rsp);

                            break;
                        }
                        (tag, body) => {
                            let text = format!("{} not supported", body.name());

                            let rsp = Response::Status(Status::no(Some(tag), None, &text).unwrap());
                            framed.send(&rsp).await.unwrap();
                            println!("S: {:?}", rsp);
                        }
                    }
                }
                Some(Ok(Outcome::ActionRequired(Action::SendLiteralAck(_)))) => {
                    println!("[!] Sending continuation request ...");

                    let rsp = Response::Continue(Continue::basic(None, "...").unwrap());

                    framed.send(&rsp).await.unwrap();
                    println!("[!] ... done.");
                }
                Some(Ok(Outcome::ActionRequired(Action::SendLiteralReject(_)))) => {
                    println!("[!] Sending literal reject ...");
                    let rsp =
                        Response::Status(Status::bad(None, None, "literal too large.").unwrap());

                    framed.send(&rsp).await.unwrap();
                    println!("[!] ... done.");
                }
                Some(Err(error)) => {
                    println!("[!] Error: {:?}", error);

                    let rsp = Response::Status(
                        Status::bad(None, None, "could not parse command").unwrap(),
                    );
                    framed.send(&rsp).await.unwrap();
                    println!("S: {:?}", rsp);
                }
                None => break,
            }
        }
    };

    // TODO:
    // let client = async {
    //     let stream = TcpStream::connect("127.0.0.1:14300").await.unwrap();
    //
    //
    // };

    server.await;
}
