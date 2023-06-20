use std::{
    io::{stdin, stdout, Read, Write},
    net::TcpStream,
    num::NonZeroU32,
};

use anyhow::{Context, Error};
use imap_codec::{
    auth::{AuthMechanism, AuthenticateData},
    command::{Command, CommandBody},
    core::{NonEmptyVec, Tag},
    fetch::{MessageDataItem, MessageDataItemName},
    mailbox::Mailbox,
    response::{Capability, Code, Continue, Data, Greeting, GreetingKind, Response, Status},
    secret::Secret,
    stream::sync::client::Client,
};
use native_tls::TlsConnector;
use rpassword::read_password;
use tracing::{info, warn};

const USAGE: &str = "USAGE: client <host> <port> [--insecure]";

/// `Stream` abstracts over `Read + Write`.
trait Stream: Read + Write {}

impl<T> Stream for T where T: Read + Write {}

fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();

    // Create a TCP or TLS stream (depending on `--insecure` flag).
    let stream = {
        let mut args = std::env::args().skip(1);
        let host = args.next().context(USAGE)?;
        let port = args
            .next()
            .context(USAGE)?
            .parse::<u16>()
            .context("Could not parse port")?;

        let insecure = {
            if let Some(flag) = args.next() {
                if flag.to_lowercase() == "--insecure" {
                    true
                } else {
                    return Err(Error::msg(format!(
                        "Last parameter can be `--insecure`, got `{}`",
                        flag
                    )));
                }
            } else {
                false
            }
        };

        let stream = TcpStream::connect((host.as_ref(), port))
            .context(format!("Could not connect to {}:{}", &host, port))?;
        let stream: Box<dyn Stream> = if insecure {
            Box::new(stream)
        } else {
            let connector = TlsConnector::new().unwrap();
            Box::new(connector.connect(&host, stream).unwrap())
        };

        stream
    };

    let mut client = Client::new(stream);

    // Receive greeting from the server ...
    let Greeting {
        kind,
        code,
        text: _,
    } = client.recv::<Greeting>().unwrap();

    // ... and check if we are happy with it.
    match kind {
        GreetingKind::Ok => {}
        GreetingKind::PreAuth => return Err(Error::msg("Unexpectedly pre-authenticated")),
        GreetingKind::Bye => return Err(Error::msg("Server rejected connection")),
    }

    let _capabilities = match code {
        Some(Code::Capability(capabilities)) => {
            info!(?capabilities, "Yay, already got capabilities in greeting");
            capabilities
        }
        _ => {
            info!("We need to ask the server for it's capabilities");
            capabilities(&mut client)?
        }
    };

    let username = prompt("Username", false);
    let password = prompt("Password", true);

    let auth_method = prompt(
        "Authentication method\na) [LOGIN]\nb) AUTH=PLAIN\nc) AUTH=LOGIN\nd) AUTH=XOAUTH2\n\nNote: The capabilities tell what the server supports.",
        false,
    );

    match auth_method.to_lowercase().as_str() {
        "a" | "" => login(&mut client, &username, &password)?,
        "b" => auth_plain(&mut client, &username, &password)?,
        "c" => auth_login(&mut client, &username, &password)?,
        "d" => auth_xoauth2(&mut client, &username, &password)?,
        _ => {
            return Err(Error::msg("Invalid authentication method"));
        }
    }

    let folders = list_folders(&mut client)?;

    println!("Available folders:");
    for (i, mailbox) in folders.iter().enumerate() {
        match mailbox {
            Mailbox::Inbox => println!("{i}\tINBOX"),
            Mailbox::Other(other) => println!(
                "{i})\t{}",
                std::str::from_utf8(other.as_ref()).context("Non-UTF-8 folder")?
            ),
        }
    }

    loop {
        let mailbox = {
            let folder_index: usize = prompt("Choose one", false).parse()?;

            folders.get(folder_index).context("No such mailbox")?
        };

        // The EXAMINE command is identical to SELECT and returns the same
        // output; however, the selected mailbox is identified as read-only.
        examine(&mut client, mailbox)?;

        let subjects = fetch_subjects(&mut client)?;

        for (uid, subject) in subjects.into_iter() {
            println!("UID: {uid}, Subject: {subject}");
        }
    }
}

/// Ask for the server capabilities.
fn capabilities(client: &mut Client<Box<dyn Stream>>) -> Result<NonEmptyVec<Capability>, Error> {
    client
        .send(Command {
            tag: Tag::unvalidated("X"),
            body: CommandBody::Capability,
        })
        .unwrap();

    let mut capabilities_tmp = None;

    loop {
        match client.recv().unwrap() {
            Response::Data(Data::Capability(capabilities)) => {
                capabilities_tmp = Some(capabilities);
            }
            Response::Status(status) if status.tag() == Some(&Tag::unvalidated("X")) => {
                if !matches!(status, Status::Ok { .. }) {
                    return Err(Error::msg("Capability command failed"));
                }

                match capabilities_tmp {
                    Some(capabilities) => {
                        info!(?capabilities, "Got capabilities");
                        return Ok(capabilities);
                    }
                    None => return Err(Error::msg("Server doesn't tell it's capabilities")),
                }
            }
            unexpected => {
                warn!("Skipping unexpected response `{unexpected:?}`");
            }
        }
    }
}

/// Prompt for a string.
fn prompt(msg: &str, password: bool) -> String {
    print!("{}\n$ ", msg);
    stdout().flush().unwrap();

    let line = if password {
        read_password().unwrap()
    } else {
        let mut line = String::new();
        stdin().read_line(&mut line).unwrap();
        line.trim().to_owned()
    };
    println!();

    line
}

/// IMAP LOGIN
fn login(
    client: &mut Client<Box<dyn Stream>>,
    username: &str,
    password: &str,
) -> Result<(), Error> {
    client
        .send(Command {
            tag: Tag::unvalidated("A"),
            body: CommandBody::login(username, password)?,
        })
        .unwrap();

    loop {
        match client.recv().unwrap() {
            Response::Status(status) if status.tag() == Some(&Tag::unvalidated("A")) => {
                if matches!(status, Status::Ok { .. }) {
                    return Ok(());
                } else {
                    return Err(Error::msg(status.text().inner().to_owned()))
                        .context("Authentication failed");
                }
            }
            unexpected => {
                warn!("Skipping unexpected response `{unexpected:?}`");
            }
        }
    }
}

/// AUTHENTICATE PLAIN
fn auth_plain(
    client: &mut Client<Box<dyn Stream>>,
    username: &str,
    password: &str,
) -> Result<(), Error> {
    client
        .send(Command {
            tag: Tag::unvalidated("B"),
            body: CommandBody::authenticate(AuthMechanism::PLAIN),
        })
        .unwrap();

    loop {
        match client.recv().unwrap() {
            Response::Continue(_) => break,
            Response::Status(status) if status.tag() == Some(&Tag::unvalidated("B")) => {
                return Err(Error::msg(status.text().inner().to_owned()))
                    .context("Unexpected status, authentication failed");
            }
            unexpected => {
                warn!("Skipping unexpected response `{unexpected:?}`");
            }
        }
    }

    client
        .send(AuthenticateData(Secret::from(
            format!("\x00{}\x00{}", username, password).into_bytes(),
        )))
        .unwrap();

    loop {
        match client.recv().unwrap() {
            Response::Status(status) if status.tag() == Some(&Tag::unvalidated("B")) => {
                if matches!(status, Status::Ok { .. }) {
                    return Ok(());
                } else {
                    return Err(Error::msg(status.text().inner().to_owned()))
                        .context("Authentication failed");
                }
            }
            unexpected => {
                warn!("Skipping unexpected response `{unexpected:?}`");
            }
        }
    }
}

/// AUTHENTICATE LOGIN
///
/// AUTH=LOGIN is slow, non-standardized, and has no advantages to AUTH=PLAIN.
/// This is only here for demonstration purposes.
fn auth_login(
    client: &mut Client<Box<dyn Stream>>,
    username: &str,
    password: &str,
) -> Result<(), Error> {
    client
        .send(Command {
            tag: Tag::unvalidated("C"),
            body: CommandBody::authenticate(AuthMechanism::LOGIN),
        })
        .unwrap();

    loop {
        match client.recv().unwrap() {
            Response::Continue(_) => break,
            Response::Status(status) if status.tag() == Some(&Tag::unvalidated("C")) => {
                return Err(Error::msg(status.text().inner().to_owned()))
                    .context("Unexpected status, authentication failed");
            }
            unexpected => {
                warn!("Skipping unexpected response `{unexpected:?}`");
            }
        }
    }

    client
        .send(AuthenticateData(Secret::from(username.as_bytes().to_vec())))
        .unwrap();

    loop {
        match client.recv().unwrap() {
            Response::Continue(_) => break,
            Response::Status(status) if status.tag() == Some(&Tag::unvalidated("C")) => {
                return Err(Error::msg(status.text().inner().to_owned()))
                    .context("Unexpected status, authentication failed");
            }
            unexpected => {
                warn!("Skipping unexpected response `{unexpected:?}`");
            }
        }
    }

    client
        .send(AuthenticateData(Secret::from(password.as_bytes().to_vec())))
        .unwrap();

    loop {
        match client.recv().unwrap() {
            Response::Status(status) if status.tag() == Some(&Tag::unvalidated("C")) => {
                if matches!(status, Status::Ok { .. }) {
                    return Ok(());
                } else {
                    return Err(Error::msg(status.text().inner().to_owned()))
                        .context("Authentication failed");
                }
            }
            unexpected => {
                warn!("Skipping unexpected response `{unexpected:?}`");
            }
        }
    }
}

/// AUTHENTICATE XOAUTH2
///
/// This is here for, e.g., Gmail.
/// Note: Setting up IMAP access in Gmail for unregistered applications became non-trivial.
fn auth_xoauth2(
    client: &mut Client<Box<dyn Stream>>,
    user: &str,
    token: &str,
) -> Result<(), Error> {
    client
        .send(Command {
            tag: Tag::unvalidated("B"),
            body: CommandBody::authenticate(AuthMechanism::XOAUTH2),
        })
        .unwrap();

    loop {
        match client.recv().unwrap() {
            Response::Continue(_) => break,
            Response::Status(status) if status.tag() == Some(&Tag::unvalidated("B")) => {
                return Err(Error::msg(status.text().inner().to_owned()))
                    .context("Unexpected status, authentication failed");
            }
            unexpected => {
                warn!("Skipping unexpected response `{unexpected:?}`");
            }
        }
    }

    client
        .send(AuthenticateData(Secret::from(
            format!("user={}\x01auth=Bearer {}\x01\x01", user, token).into_bytes(),
        )))
        .unwrap();

    loop {
        match client.recv().unwrap() {
            Response::Status(status) if status.tag() == Some(&Tag::unvalidated("B")) => {
                if matches!(status, Status::Ok { .. }) {
                    return Ok(());
                } else {
                    return Err(Error::msg(status.text().inner().to_owned()))
                        .context("Authentication failed");
                }
            }
            Response::Continue(cont) => {
                match cont {
                    Continue::Basic { .. } => {
                        warn!(?cont, "Got an error response");
                    }
                    Continue::Base64(data) => {
                        warn!(
                            json = std::str::from_utf8(&data).unwrap(),
                            "Got an error response"
                        );
                    }
                }
                client.send(Continue::base64(vec![])).unwrap();
            }
            unexpected => {
                warn!(?unexpected, "Skipping unexpected response");
            }
        }
    }
}

/// List all folders.
fn list_folders(client: &mut Client<Box<dyn Stream>>) -> Result<Vec<Mailbox<'static>>, Error> {
    // Note: We should do this differently, see https://datatracker.ietf.org/doc/html/rfc2683#section-3.2.1.1.
    client
        .send(Command {
            tag: Tag::unvalidated("L"),
            body: CommandBody::list("", "*")?,
        })
        .unwrap();

    let mut folders = vec![];

    loop {
        match client.recv().unwrap() {
            Response::Data(Data::List { mailbox, .. }) => {
                folders.push(mailbox);
            }
            Response::Status(status) if status.tag() == Some(&Tag::unvalidated("L")) => {
                break;
            }
            unexpected => {
                warn!("Skipping unexpected response `{unexpected:?}`");
            }
        }
    }

    Ok(folders)
}

/// EXAMINE a folder, i.e., SELECT read-only.
fn examine(client: &mut Client<Box<dyn Stream>>, mailbox: &Mailbox) -> Result<(), Error> {
    client
        .send(Command::new("S", CommandBody::examine(mailbox.clone())?)?)
        .unwrap();

    loop {
        match client.recv().unwrap() {
            Response::Status(status) if status.tag() == Some(&Tag::unvalidated("S")) => {
                return Ok(());
            }
            _ => {
                // Printing this would be too verbose ...
            }
        }
    }
}

/// Fetch message data items. Here, we fetch the UID and an envelope.
/// Then, we inspect the subject in the envelope.
fn fetch_subjects(
    client: &mut Client<Box<dyn Stream>>,
) -> Result<Vec<(NonZeroU32, String)>, Error> {
    client
        .send(Command::new(
            "F",
            CommandBody::fetch(
                ..,
                vec![
                    MessageDataItemName::Uid,
                    MessageDataItemName::Envelope,
                    // Uncomment for testing purposes:
                    /*
                    MessageDataItemName::Body,
                    MessageDataItemName::BodyExt {
                        section: None,
                        partial: None,
                        peek: true,
                    },
                    MessageDataItemName::BodyStructure,
                    MessageDataItemName::Envelope,
                    MessageDataItemName::Flags,
                    MessageDataItemName::InternalDate,
                    MessageDataItemName::Rfc822,
                    MessageDataItemName::Rfc822Header,
                    MessageDataItemName::Rfc822Size,
                    MessageDataItemName::Rfc822Text,
                    MessageDataItemName::Uid,
                    */
                ],
                false,
            )?,
        )?)
        .unwrap();

    let mut subjects = vec![];

    loop {
        match client.recv().unwrap() {
            Response::Data(Data::Fetch { items, .. }) => {
                let mut uid_tmp = None;
                let mut envelope_tmp = None;

                for item in items {
                    match item {
                        MessageDataItem::Uid(uid) => uid_tmp = Some(uid),
                        MessageDataItem::Envelope(envelope) => envelope_tmp = Some(envelope),
                        _ => {}
                    }
                }

                if let (Some(uid), Some(envelope)) = (uid_tmp, envelope_tmp) {
                    let subject = match envelope.subject.0 {
                        Some(subject) => match std::str::from_utf8(subject.as_ref()) {
                            Ok(subject) => subject.to_string(),
                            Err(error) => {
                                warn!(?error, "Non UTF-8 subject");
                                String::from("<non UTF-8 subject>")
                            }
                        },
                        None => {
                            warn!("Empty subject");
                            String::from("<empty subject>")
                        }
                    };

                    subjects.push((uid, subject));
                } else {
                    warn!("Requested `uid` and `envelope` but didn't get it");
                }
            }
            Response::Status(status) if status.tag() == Some(&Tag::unvalidated("F")) => {
                return Ok(subjects);
            }
            unexpected => {
                println!("Skipping unexpected response `{unexpected:?}`");
            }
        }
    }
}
