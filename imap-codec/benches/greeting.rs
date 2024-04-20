use criterion::{black_box, criterion_group, criterion_main, Criterion};
use imap_codec::{decode::Decoder, encode::Encoder, imap_types::response::Greeting, GreetingCodec};
use imap_types::{
    auth::AuthMechanism,
    core::{Text, Vec1},
    response::{Capability, Code, GreetingKind},
};

fn criterion_benchmark(c: &mut Criterion) {
    // # Setup
    let codec = GreetingCodec::new();
    let instances = [("simple", create_simple()), ("complex", create_complex())];

    for (instance, object) in instances {
        c.bench_function(
            format!("bench_greeting_serialize_{instance}").as_str(),
            |b| b.iter(|| serialize(&codec, &object)),
        );

        let input = serialize(&codec, &object);
        c.bench_function(format!("bench_greeting_parse_{instance}").as_str(), |b| {
            b.iter(|| parse(&codec, black_box(&input[..])))
        });
    }
}

fn create_simple() -> Greeting<'static> {
    Greeting {
        kind: GreetingKind::Ok,
        code: None,
        text: Text::unvalidated("."),
    }
}

fn create_complex() -> Greeting<'static> {
    Greeting {
        kind: GreetingKind::PreAuth,
        code: Some(Code::Capability(
            Vec1::try_from(vec![
                Capability::Imap4Rev1,
                Capability::Auth(AuthMechanism::Login),
                Capability::Auth(AuthMechanism::Plain),
            ])
            .unwrap(),
        )),
        text: Text::unvalidated("."),
    }
}

#[inline]
fn serialize(codec: &GreetingCodec, object: &Greeting) -> Vec<u8> {
    codec.encode(object).dump()
}

#[inline]
fn parse<'a>(codec: &GreetingCodec, input: &'a [u8]) -> Greeting<'a> {
    let (_, cmd) = codec.decode(input).unwrap();

    cmd
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
