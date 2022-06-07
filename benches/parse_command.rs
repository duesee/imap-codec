use criterion::{black_box, criterion_group, criterion_main, Criterion};
use imap_codec::command::command;
use imap_types::command::Command;

fn parse_command<'a>(input: &'a [u8]) -> Command<'a> {
    let (_remaining, cmd) = command(input).unwrap();

    cmd
}

fn criterion_benchmark(c: &mut Criterion) {
    // # Setup
    let input = b"! FETCH 7 (BODY[1768386412.HEADER.FIELDS.NOT (\"\" `)] BODY[HEADER.FIELDS.NOT (\"\" !`)] BODY[HEADER.FIELDS.NOT (\"\" {0}\r\n)])\r\n";

    c.bench_function("parse_command", |b| {
        b.iter(|| {
            parse_command(black_box(&input[..]));
        })
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
