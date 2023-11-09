use imap_types::{
    command::{Command, CommandBody},
    core::{Tag, Text},
    response::{Response, Status, StatusBody, StatusKind, Tagged},
};

fn main() {
    let cmd = Command::new("A1", CommandBody::login("Alice", "Pa²²word").unwrap()).unwrap();
    println!("{:?}\n{}", cmd, serde_json::to_string_pretty(&cmd).unwrap());

    let rsp = Response::Status(Status::Tagged(Tagged {
        tag: Tag::try_from("A1").unwrap(),
        body: StatusBody {
            kind: StatusKind::Ok,
            code: None,
            text: Text::try_from("...").unwrap(),
        },
    }));

    println!("{:?}\n{}", rsp, serde_json::to_string_pretty(&rsp).unwrap());
}
