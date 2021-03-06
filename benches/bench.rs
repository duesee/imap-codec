#![feature(test)]
extern crate test;

use imap_codec::{
    codec::Encode,
    types::{
        command::Command,
        data_items::{DataItem, MacroOrDataItems, Section},
        response::{Code, Response, Status},
    },
};
use std::convert::TryInto;
use test::Bencher;

#[bench]
fn bench_command_serialize(b: &mut Bencher) {
    // Setup
    let cmd = Command::fetch(
        "1:*,2,3,4,5,6,7,8,9",
        MacroOrDataItems::DataItems(vec![
            DataItem::Rfc822Size,
            DataItem::BodyExt {
                section: Some(Section::Text(None)),
                peek: true,
                partial: Some((1, 100)),
            },
            DataItem::BodyStructure,
            DataItem::Body,
            DataItem::Envelope,
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
