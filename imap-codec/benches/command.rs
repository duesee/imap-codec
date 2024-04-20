use criterion::{black_box, criterion_group, criterion_main, Criterion};
use imap_codec::{decode::Decoder, encode::Encoder, imap_types::command::Command, CommandCodec};
use imap_types::{
    command::CommandBody,
    core::{Charset, Tag, Vec1},
    search::SearchKey,
    sequence::{Sequence, SequenceSet},
};

fn criterion_benchmark(c: &mut Criterion) {
    // # Setup
    let codec = CommandCodec::new();
    let instances = [("simple", create_simple()), ("complex", create_complex())];

    for (instance, object) in instances {
        c.bench_function(
            format!("bench_command_serialize_{instance}").as_str(),
            |b| b.iter(|| serialize(&codec, &object)),
        );

        let input = serialize(&codec, &object);
        c.bench_function(format!("bench_command_parse_{instance}").as_str(), |b| {
            b.iter(|| parse(&codec, black_box(&input[..])))
        });
    }
}

fn create_simple() -> Command<'static> {
    Command::new(Tag::unvalidated("A"), CommandBody::Noop).unwrap()
}

fn create_complex() -> Command<'static> {
    Command::new(
        Tag::unvalidated("A"),
        CommandBody::search(
            Some(Charset::try_from("UTF-8").unwrap()),
            Vec1::try_from(vec![SearchKey::SequenceSet(SequenceSet(
                Vec1::try_from(vec![
                    Sequence::try_from("1:42").unwrap(),
                    Sequence::try_from("42:1337").unwrap(),
                    Sequence::try_from("1337:*").unwrap(),
                ])
                .unwrap(),
            ))])
            .unwrap(),
            true,
        ),
    )
    .unwrap()
}

#[inline]
fn serialize(codec: &CommandCodec, object: &Command) -> Vec<u8> {
    codec.encode(object).dump()
}

#[inline]
fn parse<'a>(codec: &CommandCodec, input: &'a [u8]) -> Command<'a> {
    let (_, cmd) = codec.decode(input).unwrap();

    cmd
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
