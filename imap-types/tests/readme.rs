use imap_types::{
    command::{Command, CommandBody},
    core::{AString, Atom, Literal, Tag},
    secret::Secret,
};

#[test]
fn test_readme() {
    Command::new("A1", CommandBody::login("alice", "password").unwrap()).unwrap();
    Command::new(
        "A1",
        CommandBody::login("alice\"", b"\xCA\xFE".as_ref()).unwrap(),
    )
    .unwrap();
    Command::new(
        "A1",
        CommandBody::login(Literal::try_from("alice").unwrap(), "password").unwrap(),
    )
    .unwrap();

    let tag = Tag::try_from("A1").unwrap();

    let _ = Command {
        tag,
        body: CommandBody::Login {
            username: AString::from(Atom::unvalidated("alice")),
            password: Secret::new(AString::from(Atom::unvalidated("password"))),
        },
    };
}

#[test]
#[should_panic]
fn test_readme_failing() {
    Command::new("A1", CommandBody::login("alice\x00", "password").unwrap()).unwrap();
}
