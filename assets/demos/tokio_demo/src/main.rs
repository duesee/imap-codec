use futures::{stream::Stream, SinkExt, StreamExt};
use imap_codec::{
    imap_types::response::{Response, Status},
    tokio::{Action, ImapServerCodec, ImapServerCodecError, Outcome},
    types::response::Continue,
};
use tokio::{
    self,
    io::Sink,
    net::{TcpListener, TcpStream},
};
use tokio_util::codec::Decoder;

#[tokio::main]
async fn main() -> Result<(), Box<std::io::Error>> {
    //let stream = TcpStream::connect("127.0.0.1:14300").await?;

    let mut listener = TcpListener::bind("127.0.0.1:14300").await?;

    loop {
        let (stream, _) = listener.accept().await?;

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

    Ok(())
}
