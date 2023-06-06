use anyhow::{Context, Error};
use futures::{SinkExt, StreamExt};
use imap_codec::{
    command::{Command, CommandBody},
    core::Tag,
    response::{Response, Status},
    tokio::client::{Event, ImapClientCodec},
};
use tokio::{self, net::TcpStream};
use tokio_util::codec::Decoder;

// Poor human's terminal color support.
const BLUE: &str = "\x1b[34m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let addr = std::env::args()
        .nth(1)
        .context("USAGE: tokio_client <host>:<port>")?;

    let mut framed = {
        let stream = TcpStream::connect(&addr)
            .await
            .context(format!("Could not connect to `{addr}`"))?;
        // This is for demonstration purposes only, and we probably want a bigger number.
        let max_literal_size = 1024;

        ImapClientCodec::new(max_literal_size).framed(stream)
    };

    // First, we read the server greeting.
    let greeting = match framed
        .next()
        .await
        // We get an `Option<Result<...>>` here that denotes ...
        // 1) if we got something from the server, and
        // 2) if it was valid.
        .context("Connection closed unexpectedly")?
        .context("Failed to obtain next message")?
    {
        Event::Greeting(greeting) => greeting,
        Event::Response(response) => {
            return Err(Error::msg(format!("Expected greeting, got `{response:?}`")));
        }
    };
    println!("S: {BLUE}{greeting:#?}{RESET}");

    // Then, we send a login command to the server ...
    let tag_login = Tag::unchecked("A1");
    let cmd = Command {
        tag: tag_login.clone(),
        body: CommandBody::login("alice", "password").context("Could not create command")?,
    };
    framed.send(&cmd).await.context("Could not send command")?;
    println!("C: {RED}{cmd:#?}{RESET}");

    // ... and process the response(s). We must read zero or many data responses before we can
    // finally examine the status response that tells us whether the login succeeded.
    loop {
        match framed
            .next()
            .await
            .context("Connection closed unexpectedly")?
            .context("Failed to obtain next message")?
        {
            Event::Greeting(greeting) => {
                return Err(Error::msg(format!("Expected response, got `{greeting:?}`")));
            }
            Event::Response(response) => match response {
                Response::Data(_) => {
                    println!("[!] got data");
                }
                Response::Status(Status::Ok {
                    tag: Some(ref tag), ..
                }) if *tag == tag_login => {
                    println!("[!] login successful");
                    return Ok(());
                }
                Response::Status(Status::No {
                    tag: Some(ref tag), ..
                }) if *tag == tag_login => {
                    println!("[!] login failed");
                    return Ok(());
                }
                unexpected => {
                    return Err(Error::msg(format!("Unexpected response `{unexpected:?}`")));
                }
            },
        }
    }
}
