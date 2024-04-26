use criterion::{black_box, criterion_group, criterion_main, Criterion};
use imap_codec::{decode::Decoder, encode::Encoder, imap_types::command::Command, CommandCodec};
use imap_proto_stalwart::{
    receiver::Receiver as ImapProtoStalwartReceiver, Command as ImapProtoStalwartCommand,
};
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

        // Note: We don't get a fully serialized command with stalwart here. This hinders comparison ...
        //
        // ```rust
        // Request {
        //     tag: "A",
        //     command: Search(true),
        //     tokens: [
        //         Argument([67, 72, 65, 82, 83, 69, 84]), // CHARSET
        //         Argument([85, 84, 70, 45, 56]), // UTF-8
        //         Argument([49, 58, 52, 50, 44, 52, 50, 58, 49, 51, 51, 55, 44, 49, 51, 51, 55, 58, 42]), // 1:42,42:1337,1337:*
        //     ]
        // }
        // ```
        let input = serialize(&codec, &object);
        let mut receiver: ImapProtoStalwartReceiver<ImapProtoStalwartCommand> =
            ImapProtoStalwartReceiver::new();
        c.bench_function(
            format!("bench_command_parse_{instance}_imap_proto_stalwart").as_str(),
            |b| b.iter(|| receiver.parse(black_box(&mut input.iter())).unwrap()),
        );
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
