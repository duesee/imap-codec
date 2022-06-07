use criterion::{black_box, criterion_group, criterion_main, Criterion};
use imap_codec::response::response;
use imap_types::response::Response;

fn parse_response<'a>(input: &'a [u8]) -> Response<'a> {
    let (_remaining, rsp) = response(input).unwrap();

    rsp
}

fn criterion_benchmark(c: &mut Criterion) {
    // # Setup
    let input = b"* 12 FETCH (FLAGS (\\Seen) INTERNALDATE \"17-Jul-1996 02:44:25 -0700\" RFC822.SIZE 4286 ENVELOPE (\"Wed, 17 Jul 1996 02:23:25 -0700 (PDT)\" \"IMAP4rev1 WG mtg summary and minutes\" ((\"Terry Gray\" NIL \"gray\" \"cac.washington.edu\")) ((\"Terry Gray\" NIL \"gray\" \"cac.washington.edu\")) ((\"Terry Gray\" NIL \"gray\" \"cac.washington.edu\")) ((NIL NIL \"imap\" \"cac.washington.edu\")) ((NIL NIL \"minutes\" \"CNRI.Reston.VA.US\")(\"John Klensin\" NIL \"KLENSIN\" \"MIT.EDU\")) NIL NIL \"<B27397-0100000@cac.washington.edu>\") BODY (\"TEXT\" \"PLAIN\" (\"CHARSET\" \"US-ASCII\") NIL NIL \"7BIT\" 3028 92))\r\n";

    c.bench_function("parse_response", |b| {
        b.iter(|| {
            parse_response(black_box(&input[..]));
        })
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
