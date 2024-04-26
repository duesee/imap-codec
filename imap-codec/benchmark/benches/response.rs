use std::num::NonZeroU32;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use imap_codec::{decode::Decoder, encode::Encoder, imap_types::response::Response, ResponseCodec};
use imap_proto::Response as ImapProtoResponse;
use imap_types::{core::Vec1, fetch::MessageDataItem, response::Data};

fn criterion_benchmark(c: &mut Criterion) {
    // # Setup
    let codec = ResponseCodec::new();
    let instances = [("simple", create_simple()), ("complex", create_complex())];

    for (instance, object) in instances {
        c.bench_function(
            format!("bench_response_serialize_{instance}").as_str(),
            |b| b.iter(|| serialize(&codec, &object)),
        );

        let input = serialize(&codec, &object);
        c.bench_function(format!("bench_response_parse_{instance}").as_str(), |b| {
            b.iter(|| parse(&codec, black_box(&input[..])))
        });

        let input = serialize(&codec, &object);
        c.bench_function(
            format!("bench_response_parse_{instance}_imap_proto").as_str(),
            |b| b.iter(|| ImapProtoResponse::from_bytes(black_box(&input[..]))),
        );
    }
}

fn create_simple() -> Response<'static> {
    Response::Data(Data::Exists(0))
}

fn create_complex() -> Response<'static> {
    Response::Data(Data::Fetch {
        seq: NonZeroU32::try_from(u32::MAX).unwrap(),
        items: Vec1::try_from(vec![MessageDataItem::Rfc822Size(0)]).unwrap(),
    })
}

#[inline]
fn serialize(codec: &ResponseCodec, object: &Response) -> Vec<u8> {
    codec.encode(object).dump()
}

#[inline]
fn parse<'a>(codec: &ResponseCodec, input: &'a [u8]) -> Response<'a> {
    let (_, cmd) = codec.decode(input).unwrap();

    cmd
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
