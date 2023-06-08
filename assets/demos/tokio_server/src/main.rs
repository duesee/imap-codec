use anyhow::{Context, Error};
use futures::{SinkExt, StreamExt};
use imap_codec::{
    command::CommandBody,
    core::NonEmptyVec,
    response::{Capability, Continue, Data, Greeting, Response, Status},
    tokio::server::{Action, Event, ImapServerCodec},
};
use tokio::{self, net::TcpListener};
use tokio_util::codec::Decoder;

// Poor human's terminal color support.
const BLUE: &str = "\x1b[34m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let addr = std::env::args()
        .nth(1)
        .context("USAGE: tokio_server <host>:<port>")?;

    let mut framed = {
        let stream = {
            // Bind listener ...
            let listener = TcpListener::bind(&addr)
                .await
                .context(format!("Could not bind to `{addr}`"))?;

            // ... and accept a single connection.
            let (stream, _) = listener
                .accept()
                .await
                .context(format!("Could not accept connection"))?;

            stream
        };

        // Accept 2 MiB literals.
        let mib2 = 2 * 1024 * 1024;
        ImapServerCodec::new(mib2).framed(stream)
    };

    // Send a positive greeting ...
    let greeting = Greeting::ok(None, "Hello, World!").context("Could not create greeting")?;
    framed
        .send(&greeting)
        .await
        .context("Could not send greeting")?;
    println!("S: {BLUE}{greeting:#?}{RESET}");

    // ... and process the following commands in a loop.
    loop {
        match framed
            .next()
            .await
            .context("Connection closed unexpectedly")?
            .context("Failed to obtain next message")?
        {
            Event::Command(cmd) => {
                println!("C: {RED}{cmd:#?}{RESET}");

                match (cmd.tag, cmd.body) {
                    (tag, CommandBody::Capability) => {
                        let rsp = Response::Data(Data::Capability(NonEmptyVec::from(
                            Capability::Imap4Rev1,
                        )));
                        framed.send(&rsp).await.context("Could not send response")?;
                        println!("S: {BLUE}{rsp:#?}{RESET}");

                        let rsp = Response::Status(
                            Status::ok(Some(tag), None, "CAPABILITY done")
                                .context("Could not create `Status`")?,
                        );
                        framed.send(&rsp).await.context("Could not send response")?;
                        println!("S: {BLUE}{rsp:#?}{RESET}");
                    }
                    (tag, CommandBody::Login { username, password }) => {
                        let rsp =
                            if username.as_ref() == b"alice" && password.compare_with("password") {
                                Response::Status(
                                    Status::ok(Some(tag), None, "LOGIN succeeded")
                                        .context("Could not create `Status`")?,
                                )
                            } else {
                                Response::Status(
                                    Status::no(Some(tag), None, "LOGIN failed")
                                        .context("Could not create `Status`")?,
                                )
                            };
                        framed.send(&rsp).await.context("Could not send response")?;
                        println!("S: {BLUE}{rsp:#?}{RESET}");
                    }
                    (tag, CommandBody::Logout) => {
                        let rsp = Response::Status(
                            Status::bye(None, "...").expect("Could not create `Status`"),
                        );
                        framed.send(&rsp).await.context("Could not send response")?;
                        println!("S: {BLUE}{rsp:#?}{RESET}");

                        let rsp = Response::Status(
                            Status::ok(Some(tag), None, "LOGOUT done")
                                .expect("Could not create `Status`"),
                        );
                        framed.send(&rsp).await.context("Could not send response")?;
                        println!("S: {BLUE}{rsp:#?}{RESET}");

                        return Ok(());
                    }
                    (tag, body) => {
                        let text = format!("{} not supported", body.name());
                        let rsp = Response::Status(
                            Status::no(Some(tag), None, text)
                                .context("Could not create `Status`")?,
                        );
                        framed.send(&rsp).await.context("Could not send response")?;
                        println!("S: {BLUE}{rsp:#?}{RESET}");
                    }
                }
            }
            Event::ActionRequired(Action::SendLiteralAck(_)) => {
                println!("[!] Send continuation request.");
                let rsp = Response::Continue(
                    Continue::basic(None, "...").context("Could not create `Continue`")?,
                );
                framed.send(&rsp).await.context("Could not send response")?;
                println!("S: {BLUE}{rsp:#?}{RESET}");
            }
            Event::ActionRequired(Action::SendLiteralReject(_)) => {
                println!("[!] Send literal reject.");
                let rsp = Response::Status(
                    Status::bad(None, None, "literal too large.")
                        .context("Could not create `Status`")?,
                );
                framed.send(&rsp).await.context("Could not send response")?;
                println!("S: {BLUE}{rsp:#?}{RESET}");
            }
        }
    }
}
