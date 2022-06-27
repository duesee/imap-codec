use futures::{SinkExt, StreamExt};
use imap_codec::{
    tokio_compat::{Action, ImapServerCodec, OutcomeServer},
    types::{
        api::response::{Continue, Response, Status},
        command::CommandBody,
        response::Greeting,
    },
};
use subtle::ConstantTimeEq;
use tokio::{self, net::TcpListener};
use tokio_util::codec::Decoder;

#[tokio::main]
async fn main() {
    let mut framed = {
        let stream = {
            // Bind listener ...
            let listener = TcpListener::bind("127.0.0.1:14300").await.unwrap();

            // ... and accept a single connection.
            let (stream, _) = listener.accept().await.unwrap();

            stream
        };

        // Accept 2 MiB ...
        let mib2 = 2 * 1024 * 1024;
        // ... literals.
        ImapServerCodec::new(mib2).framed(stream)
    };

    // Send OK greeting.
    let greeting = Response::Greeting(Greeting::ok(None, "Hello, World!").unwrap());
    framed.send(&greeting).await.unwrap();
    println!("S: {:?}", greeting);

    // Process client commands in a loop.
    while let Some(outcome) = framed.next().await {
        match outcome {
            Ok(OutcomeServer::Command(cmd)) => {
                println!("C: {:?}", cmd);

                match (cmd.tag, cmd.body) {
                    (tag, CommandBody::Login { username, password }) => {
                        // Convert `AString`s to `&[u8]` ...
                        let (username, password) = (username.as_ref(), password.as_ref());

                        // ... and compare them with something. (In this example we compare the
                        // hard-codec username and password in constant time by using the "subtle" crate.)
                        let rsp = if username.ct_eq(b"alice").unwrap_u8() == 1
                            && password.ct_eq(b"password").unwrap_u8() == 1
                        {
                            Response::Status(
                                Status::ok(Some(tag), None, "LOGIN succeeded").unwrap(),
                            )
                        } else {
                            Response::Status(Status::no(Some(tag), None, "LOGIN failed").unwrap())
                        };
                        framed.send(&rsp).await.unwrap();
                        println!("S: {:?}", rsp);
                    }
                    (tag, CommandBody::Logout) => {
                        let rsp = Response::Status(Status::bye(None, "...").unwrap());
                        framed.send(&rsp).await.unwrap();
                        println!("S: {:?}", rsp);

                        let rsp =
                            Response::Status(Status::ok(Some(tag), None, "LOGOUT done").unwrap());
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
            Ok(OutcomeServer::ActionRequired(Action::SendLiteralAck(_))) => {
                println!("[!] Send continuation request.");
                let rsp = Response::Continue(Continue::basic(None, "...").unwrap());
                framed.send(&rsp).await.unwrap();
                println!("S: {:?}", rsp);
            }
            Ok(OutcomeServer::ActionRequired(Action::SendLiteralReject(_))) => {
                println!("[!] Send literal reject.");
                let rsp = Response::Status(Status::bad(None, None, "literal too large.").unwrap());
                framed.send(&rsp).await.unwrap();
                println!("S: {:?}", rsp);
            }
            Err(error) => {
                println!("[!] Error: {:?}", error);
                let rsp =
                    Response::Status(Status::bad(None, None, "could not parse command").unwrap());
                framed.send(&rsp).await.unwrap();
                println!("S: {:?}", rsp);
            }
        }
    }
}
