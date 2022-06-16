use futures::{SinkExt, StreamExt};
use imap_codec::{
    tokio_compat::{Action, ImapServerCodec, Outcome},
    types::api::response::{Continue, Response, Status},
};
use tokio::{self, net::TcpListener};
use tokio_util::codec::Decoder;

#[tokio::main]
async fn main() {
    //let stream = TcpStream::connect("127.0.0.1:14300").await.unwrap();

    let listener = TcpListener::bind("127.0.0.1:14300").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();

        let mut framed = ImapServerCodec::new(4).framed(stream);

        loop {
            match framed.next().await {
                Some(Ok(Outcome::Command(cmd))) => {
                    println!("Command: {:?}", cmd);
                }
                Some(Ok(Outcome::ActionRequired(Action::SendLiteralAck(_)))) => {
                    println!("Sending continuation request ...");
                    framed
                        .send(Response::Continue(Continue::basic(None, "...").unwrap()))
                        .await
                        .unwrap();
                    println!("... done.");
                }
                Some(Ok(Outcome::ActionRequired(Action::SendLiteralReject(_)))) => {
                    println!("Sending literal reject ...");
                    framed
                        .send(Response::Status(
                            Status::bad(None, None, "literal too large.").unwrap(),
                        ))
                        .await
                        .unwrap();
                    println!("... done.");
                }
                Some(Err(error)) => {
                    println!("Error: {:?}", error);
                }
                None => break,
            }
        }
    }
}
