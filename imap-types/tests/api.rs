use std::fmt::{Debug, Display};

use imap_types::{
    command::{Command, CommandBody, LoginError},
    core::{Atom, AtomExt},
    response::Data,
};

#[test]
fn test_construction_of_atom() {
    // `inner` is a private field
    // let atm = Atom {
    //     inner: Cow::Borrowed(" x "),
    // };

    // let mut atm = Atom::try_from("valid").unwrap();

    // `inner` is a private field
    // atm.inner = Cow::Borrowed(" x x x ");

    // Panics
    // let mut atm = Atom::try_from(" x ").unwrap();

    // #[cfg(feature = "unchecked")]
    // let atm = Atom::unchecked(" x ");
}

#[test]
fn test_construction_of_command() {
    trait DisplayDebug: Display + Debug {}

    impl<T> DisplayDebug for T where T: Display + Debug {}

    match CommandBody::login("\x00", "") {
        Err(LoginError::Username(e)) => println!("Oops, bad username: {}", e),
        Err(LoginError::Password(e)) => println!("Oops, bad password: {:?}", e),
        _ => {}
    }

    let tests: Vec<Box<dyn DisplayDebug>> = vec![
        Box::new(Command::new(b"".as_ref(), CommandBody::Noop).unwrap_err()),
        Box::new(Command::new(b"A ".as_ref(), CommandBody::Noop).unwrap_err()),
        Box::new(Command::new(b"\xff".as_ref(), CommandBody::Noop).unwrap_err()),
        Box::new("---"),
        Box::new(Command::new("", CommandBody::Noop).unwrap_err()),
        Box::new(Command::new("A ", CommandBody::Noop).unwrap_err()),
        Box::new("---"),
        Box::new(Command::new(String::from(""), CommandBody::Noop).unwrap_err()),
        Box::new(Command::new(String::from("A "), CommandBody::Noop).unwrap_err()),
        Box::new("---"),
        Box::new(Command::new(Vec::from(b"".as_ref()), CommandBody::Noop).unwrap_err()),
        Box::new(Command::new(Vec::from(b"\xff".as_ref()), CommandBody::Noop).unwrap_err()),
        Box::new("---"),
        Box::new(Atom::try_from("").unwrap_err()),
        Box::new(Atom::try_from("²").unwrap_err()),
        Box::new("---"),
        Box::new(AtomExt::try_from("").unwrap_err()),
        Box::new(AtomExt::try_from("²").unwrap_err()),
        Box::new("---"),
        Box::new(CommandBody::login("\x00", "").unwrap_err()),
        Box::new(CommandBody::login("", b"\x00".as_ref()).unwrap_err()),
        Box::new("---"),
        Box::new(Data::capability(vec![]).unwrap_err()),
    ];

    for test in tests.into_iter() {
        println!("{test:?} // {test}");
    }
}
