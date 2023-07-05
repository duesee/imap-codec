use imap_types::{
    command::{Command, CommandBody},
    core::{Tag, Text},
    response::{Response, Status},
};
use serde_json;

fn main() {
    let cmd = Command::new("A1", CommandBody::login("Alice", "Pa²²word").unwrap()).unwrap();
    println!("{:?}\n{}", cmd, serde_json::to_string_pretty(&cmd).unwrap());

    let rsp = Response::Status(Status::Ok {
        tag: Some(Tag::try_from("A1").unwrap()),
        code: None,
        text: Text::try_from("...").unwrap(),
    });

    println!("{:?}\n{}", rsp, serde_json::to_string_pretty(&rsp).unwrap());
}
