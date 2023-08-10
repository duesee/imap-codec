use std::num::NonZeroU32;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use imap_codec::{
    encode::Encoder,
    imap_types::{
        command::{Command, CommandBody},
        fetch::{MacroOrMessageDataItemNames, MessageDataItemName, Section},
    },
    CommandCodec,
};

fn criterion_benchmark(c: &mut Criterion) {
    // # Setup
    //
    // Create a `Command` ...
    // TODO: What about other instances of `Command`?
    let cmd = Command::new(
        "C123",
        CommandBody::fetch(
            "1:*,2,3,4,5,6,7,8,9",
            MacroOrMessageDataItemNames::MessageDataItemNames(vec![
                MessageDataItemName::Rfc822Size,
                MessageDataItemName::BodyExt {
                    section: Some(Section::Text(None)),
                    peek: true,
                    partial: Some((1, NonZeroU32::try_from(100).unwrap())),
                },
                MessageDataItemName::BodyStructure,
                MessageDataItemName::Body,
                MessageDataItemName::Envelope,
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
            let tmp = CommandCodec::default().encode(&cmd).dump();
            out.extend_from_slice(black_box(&tmp));

            // TODO: This should be a single instruction... should...
            out.clear();
        })
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
