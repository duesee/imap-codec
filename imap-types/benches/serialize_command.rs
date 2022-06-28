use std::{convert::TryFrom, num::NonZeroU32};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use imap_types::{
    codec::Encode,
    command::{
        fetch::{FetchAttribute, MacroOrFetchAttributes},
        Command, CommandBody,
    },
    message::Section,
};

fn serialize_command(cmd: &Command, out: &mut Vec<u8>) {
    cmd.encode(out).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    // # Setup
    //
    // Create a `Command` ...
    // TODO: What about other instances of `Command`?
    let cmd = Command::new(
        "C123",
        CommandBody::fetch(
            "1:*,2,3,4,5,6,7,8,9",
            MacroOrFetchAttributes::FetchAttributes(vec![
                FetchAttribute::Rfc822Size,
                FetchAttribute::BodyExt {
                    section: Some(Section::Text(None)),
                    peek: true,
                    partial: Some((1, NonZeroU32::try_from(100).unwrap())),
                },
                FetchAttribute::BodyStructure,
                FetchAttribute::Body,
                FetchAttribute::Envelope,
            ]),
            true,
        )
        .unwrap(),
    )
    .unwrap();

    // ... and preallocate some memory to serialize the `Command` into.
    let mut out = Vec::with_capacity(512);

    c.bench_function("serialize_command", |b| {
        b.iter(|| {
            serialize_command(black_box(&cmd), black_box(&mut out));

            // TODO: This should be a single instruction... should...
            out.clear();
        })
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
