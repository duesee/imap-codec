use std::{convert::TryInto, num::NonZeroU32};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use imap_types::{
    codec::Encode,
    response::{Code, Response, Status},
};

fn serialize_response(rsp: &Response, out: &mut Vec<u8>) {
    rsp.encode(out).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    // # Setup
    //
    // Create a `Response` ...
    // TODO: What about other instances of `Response`?
    let rsp = Response::Status(
        Status::ok(
            Some("ABC1234567".try_into().unwrap()),
            Some(Code::Unseen(NonZeroU32::new(12345).unwrap())),
            "xyz...",
        )
        .unwrap(),
    );

    // ... and preallocate some memory to serialize the `Command` into.
    let mut out = Vec::with_capacity(512);

    c.bench_function("serialize_response", |b| {
        b.iter(|| {
            serialize_response(black_box(&rsp), black_box(&mut out));

            // TODO: This should be a single instruction... should...
            out.clear();
        })
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
