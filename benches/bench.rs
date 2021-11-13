#![feature(test)]
extern crate test;

use std::convert::TryInto;

use imap_codec::{
    codec::Encode,
    types::{
        command::Command,
        fetch_attributes::{FetchAttribute, MacroOrFetchAttributes, Section},
        response::{Code, Response, Status},
    },
};
use test::Bencher;

#[bench]
fn bench_command_serialize(b: &mut Bencher) {
    // Setup
    let cmd = Command::fetch(
        "1:*,2,3,4,5,6,7,8,9",
        MacroOrFetchAttributes::FetchAttributes(vec![
            FetchAttribute::Rfc822Size,
            FetchAttribute::BodyExt {
                section: Some(Section::Text(None)),
                peek: true,
                partial: Some((1, 100)),
            },
            FetchAttribute::BodyStructure,
            FetchAttribute::Body,
            FetchAttribute::Envelope,
        ]),
        true,
    )
    .unwrap();

    // Bench
    b.iter(|| {
        let mut out = Vec::with_capacity(512);
        cmd.encode(&mut out).unwrap();
        // Make sure that serialization step is not removed as dead code.
        // Not sure if needed...
        test::black_box(out);
    });
}

#[bench]
fn bench_response_serialize(b: &mut Bencher) {
    // Setup
    let tag = "ABC1234567".try_into().unwrap();

    let rsp = Response::Status(
        Status::ok(
            Some(tag),
            Some(Code::Other("XXXXX".try_into().unwrap(), None)),
            "xyz...",
        )
        .unwrap(),
    );

    // Bench
    b.iter(|| {
        let mut out = Vec::with_capacity(512);
        rsp.encode(&mut out).unwrap();
        // Make sure that serialization step is not removed as dead code.
        // Not sure if needed...
        test::black_box(out);
    });
}
