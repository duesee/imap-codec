use std::fmt::{Debug, Display};

use imap_types::{
    command::{error::LoginError, Command, CommandBody},
    core::{AString, Atom, AtomExt, Charset, IString, Literal, NString, Quoted, Tag, Text},
    mailbox::{Mailbox, MailboxOther},
    response::Data,
    sequence::{SeqOrUid, Sequence, SequenceSet, MAX, MIN},
};

macro_rules! test_conversions {
    // Unvalidated
    (y, $try_from:tt, $from:tt, $as_ref:tt, $object:ty, $sample:expr) => {{
        let object = <$object>::unvalidated($sample);
        let _ = object.as_ref();

        test_conversions!($try_from, $from, $as_ref, $object, $sample);
    }};
    (n, $try_from:tt, $from:tt, $as_ref:tt, $object:ty, $sample:expr) => {{
        test_conversions!($try_from, $from, $as_ref, $object, $sample);
    }};

    // TryFrom
    (y, $from:tt, $as_ref:tt, $object:ty, $sample:expr) => {{
        let _ = <$object>::try_from($sample).unwrap();
        let _ = <$object>::try_from($sample.to_owned()).unwrap();
        let _ = <$object>::try_from($sample.as_bytes()).unwrap();
        let _ = <$object>::try_from($sample.as_bytes().to_vec()).unwrap();

        test_conversions!($from, $as_ref, $object, $sample);
    }};
    (n, $from:tt, $as_ref:tt, $object:ty, $sample:expr) => {{
        test_conversions!($from, $as_ref, $object, $sample);
    }};

    // From
    (y, $as_ref:tt, $object:ty, $sample:expr) => {{
        let _ = <$object>::from($sample);

        test_conversions!($as_ref, $object, $sample);
    }};
    (n, $as_ref:tt, $object:ty, $sample:expr) => {{
        test_conversions!($as_ref, $object, $sample);
    }};

    // AsRef
    (y, $object:ty, $sample:expr) => {{
        let object = <$object>::try_from($sample).unwrap();
        let _ = object.as_ref();

        // ...
    }};
    (n, $object:ty, $sample:expr) => {{
        // ...
    }};
}

#[test]
fn test_constructions() {
    // Unvalidated | TryFrom | From | AsRef | Type | Sample
    test_conversions!(y, y, n, y, Tag, "tag");
    test_conversions!(y, y, n, y, Text, "text");
    // --------------------------------------------
    test_conversions!(n, y, n, y, AString, "astring");
    test_conversions!(y, y, n, y, Atom, "atom");
    test_conversions!(y, y, n, y, AtomExt, "atomext");
    test_conversions!(n, y, n, y, IString, "istring");
    test_conversions!(y, y, n, y, Quoted, "quoted");
    test_conversions!(n, y, n, y, Literal, "literal");
    test_conversions!(n, y, n, n, NString, "nstring");
    // --------------------------------------------
    test_conversions!(n, y, n, n, Mailbox, "mailbox");
    test_conversions!(n, y, n, y, MailboxOther, "mailbox");
    // --------------------------------------------
    test_conversions!(n, y, n, y, Charset, "charset");
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

#[test]
fn test_construction_of_sequence_etc() {
    // # From
    // ## SequenceSet
    let _ = SequenceSet::from(MIN);
    let _ = SequenceSet::from(MAX);
    let _ = SequenceSet::from(..);
    let _ = SequenceSet::from(MIN..);
    let _ = SequenceSet::try_from(MIN..MAX).unwrap();
    let _ = SequenceSet::from(MIN..=MAX);
    let _ = SequenceSet::try_from(..MAX).unwrap();
    let _ = SequenceSet::from(MIN..=MAX);
    // ## Sequence
    let _ = Sequence::from(MIN);
    let _ = Sequence::from(MAX);
    let _ = Sequence::from(..);
    let _ = Sequence::from(MIN..);
    let _ = Sequence::try_from(MIN..MAX).unwrap();
    let _ = Sequence::from(MIN..=MAX);
    let _ = Sequence::try_from(..MAX).unwrap();
    let _ = Sequence::from(MIN..=MAX);
    // ## SeqOrUid
    let _ = SeqOrUid::from(MIN);
    let _ = SeqOrUid::from(MAX);

    macro_rules! try_from {
        ($min:literal, $max:literal) => {
            let _ = SequenceSet::try_from($min).unwrap();
            let _ = SequenceSet::try_from($max).unwrap();
            let _ = SequenceSet::try_from(..).unwrap();
            let _ = SequenceSet::try_from($min..).unwrap();
            let _ = SequenceSet::try_from($min..$max).unwrap();
            let _ = SequenceSet::try_from(..$max).unwrap();
            let _ = SequenceSet::try_from($min..$max).unwrap();

            let _ = Sequence::try_from($min).unwrap();
            let _ = Sequence::try_from($max).unwrap();
            let _ = Sequence::try_from(..).unwrap();
            let _ = Sequence::try_from($min..).unwrap();
            let _ = Sequence::try_from($min..$max).unwrap();
            let _ = Sequence::try_from(..$max).unwrap();
            let _ = Sequence::try_from($min..$max).unwrap();

            let _ = SeqOrUid::try_from($min).unwrap();
            let _ = SeqOrUid::try_from($max).unwrap();
        };
    }

    try_from!(1i8, 127i8);
    try_from!(1i16, 32_767i16);
    try_from!(1i32, 2_147_483_647i32);
    try_from!(1i64, 2_147_483_647i64);
    try_from!(1isize, 2_147_483_647isize);
    try_from!(1u8, 255u8);
    try_from!(1u16, 65_535u16);
    try_from!(1u32, 4_294_967_295u32);
    try_from!(1u64, 4_294_967_295u64);
    try_from!(1usize, 4_294_967_295usize);

    macro_rules! try_from_fail_zero {
        ($min:literal, $max:literal) => {
            let _ = SequenceSet::try_from($min).unwrap_err();
            let _ = SequenceSet::try_from($min..).unwrap_err();
            let _ = SequenceSet::try_from($min..$max).unwrap_err();
            let _ = SequenceSet::try_from($min..$max).unwrap_err();

            let _ = Sequence::try_from($min).unwrap_err();
            let _ = Sequence::try_from($min..).unwrap_err();
            let _ = Sequence::try_from($min..$max).unwrap_err();
            let _ = Sequence::try_from($min..$max).unwrap_err();

            let _ = SeqOrUid::try_from($min).unwrap_err();
        };
    }

    try_from_fail_zero!(0i8, 127i8);
    try_from_fail_zero!(0i16, 32_767i16);
    try_from_fail_zero!(0i32, 2_147_483_647i32);
    try_from_fail_zero!(0i64, 2_147_483_647i64);
    try_from_fail_zero!(0isize, 2_147_483_647isize);
    try_from_fail_zero!(0u8, 255u8);
    try_from_fail_zero!(0u16, 65_535u16);
    try_from_fail_zero!(0u32, 4_294_967_295u32);
    try_from_fail_zero!(0u64, 4_294_967_295u64);
    try_from_fail_zero!(0usize, 4_294_967_295usize);

    macro_rules! try_from_fail_max {
        ($min:literal, $max:literal) => {
            let _ = SequenceSet::try_from($max).unwrap_err();
            let _ = SequenceSet::try_from($min..$max).unwrap_err();
            let _ = SequenceSet::try_from(..$max).unwrap_err();
            let _ = SequenceSet::try_from($min..$max).unwrap_err();

            let _ = Sequence::try_from($max).unwrap_err();
            let _ = Sequence::try_from($min..$max).unwrap_err();
            let _ = Sequence::try_from(..$max).unwrap_err();
            let _ = Sequence::try_from($min..$max).unwrap_err();

            let _ = SeqOrUid::try_from($max).unwrap_err();
        };
    }

    try_from_fail_max!(1i64, 9_223_372_036_854_775_807i64);
    try_from_fail_max!(1u64, 18_446_744_073_709_551_615u64);
}
