use futures::{SinkExt, StreamExt};
use imap_codec::{
    command::{Command, CommandBody},
    message::Tag,
    response::{Response, Status},
    tokio_compat::client::{ImapClientCodec, OutcomeClient},
};
use tokio::{self, net::TcpStream};
use tokio_util::codec::Decoder;

#[tokio::main]
async fn main() {
    let client = async {
        let mut framed = {
            let stream = TcpStream::connect("127.0.0.1:14300").await.unwrap();
            let kib1 = 1024;

            ImapClientCodec::new(kib1).framed(stream)
        };

        // Read greeting
        let greeting = framed.next().await.unwrap();
        println!("S: {:?}", greeting);

        // Send LOGIN
        let tag_login = Tag::try_from("A1").unwrap();
        let cmd = Command::new(
            tag_login.clone(),
            CommandBody::login("alice", "password").unwrap(),
        )
        .unwrap();
        framed.send(&cmd).await.unwrap();
        println!("C: {:?}", cmd);

        loop {
            match framed.next().await.unwrap() {
                Ok(rsp) => {
                    println!("S: {:?}", rsp);

                    match rsp {
                        OutcomeClient::Respone(Response::Data(_)) => {
                            println!("[!] got data");
                        }
                        OutcomeClient::Respone(Response::Status(Status::Ok {
                            tag: Some(ref tag),
                            ..
                        })) if *tag == tag_login => {
                            println!("[!] login successful");
                            break;
                        }
                        OutcomeClient::Respone(Response::Status(Status::No {
                            tag: Some(ref tag),
                            ..
                        })) if *tag == tag_login => {
                            println!("[!] login failed");
                            break;
                        }
                        _ => println!("[!] unexpected response"),
                    }
                }
                Err(error) => {
                    println!("{:?}", error);
                }
            }
        }
    };

    client.await;
}
