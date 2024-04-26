use criterion::{criterion_group, criterion_main, Criterion};
use imap_codec::{decode::Decoder, CommandCodec, GreetingCodec, ResponseCodec};
use imap_types::{
    command::Command,
    response::{Greeting, Response},
};

fn criterion_benchmark(c: &mut Criterion) {
    // # Setup
    let grt_codec = GreetingCodec::new();
    let cmd_codec = CommandCodec::new();
    let rsp_codec = ResponseCodec::new();
    let instances = create();

    c.bench_function("bench_trace", |b| {
        b.iter(|| {
            let mut tmp = Vec::new();
            for (input, message) in &instances {
                tmp.push(match message {
                    Message::Greeting => MessageOut::Greeting(grt_codec.decode(input).unwrap().1),
                    Message::Command => MessageOut::Command(cmd_codec.decode(input).unwrap().1),
                    Message::Response => MessageOut::Response(rsp_codec.decode(input).unwrap().1),
                });
            }
            tmp
        })
    });
}

enum Message {
    Greeting,
    Command,
    Response,
}

enum MessageOut<'a> {
    #[allow(unused)]
    Greeting(Greeting<'a>),
    #[allow(unused)]
    Command(Command<'a>),
    #[allow(unused)]
    Response(Response<'a>),
}

fn create() -> Vec<(&'static [u8], Message)> {
    vec![
        (
            b"* OK IMAP4rev1 Service Ready\r\n".as_ref(),
            Message::Greeting
        ),
        (
            b"a001 login mrc secret\r\n",
            Message::Command
        ),
        (
            b"a001 OK LOGIN completed\r\n",
            Message::Response
        ),
        (
            b"a002 select inbox\r\n",
            Message::Command
        ),
        (
            b"* 18 EXISTS\r\n",
            Message::Response
        ),
        (
            b"* FLAGS (\\Answered \\Flagged \\Deleted \\Seen \\Draft)\r\n",
            Message::Response
        ),
        (
            b"* 2 RECENT\r\n",
            Message::Response
        ),
        (
            b"* OK [UNSEEN 17] Message 17 is the first unseen message\r\n",
            Message::Response
        ),
        (
            b"* OK [UIDVALIDITY 3857529045] UIDs valid\r\n",
            Message::Response
        ),
        (
            b"a002 OK [READ-WRITE] SELECT completed\r\n",
            Message::Response
        ),
        (
            b"a003 fetch 12 full\r\n",
            Message::Command
        ),
        (
            b"* 12 FETCH (FLAGS (\\Seen) INTERNALDATE \"17-Jul-1996 02:44:25 -0700\" RFC822.SIZE 4286 ENVELOPE (\"Wed, 17 Jul 1996 02:23:25 -0700 (PDT)\" \"IMAP4rev1 WG mtg summary and minutes\" ((\"Terry Gray\" NIL \"gray\" \"cac.washington.edu\")) ((\"Terry Gray\" NIL \"gray\" \"cac.washington.edu\")) ((\"Terry Gray\" NIL \"gray\" \"cac.washington.edu\")) ((NIL NIL \"imap\" \"cac.washington.edu\")) ((NIL NIL \"minutes\" \"CNRI.Reston.VA.US\")(\"John Klensin\" NIL \"KLENSIN\" \"MIT.EDU\")) NIL NIL \"<B27397-0100000@cac.washington.edu>\") BODY (\"TEXT\" \"PLAIN\" (\"CHARSET\" \"US-ASCII\") NIL NIL \"7BIT\" 3028 92))\r\n",
            Message::Response
        ),
        (
            b"a003 OK FETCH completed\r\n",
            Message::Response
        ),
        (
            b"a004 fetch 12 body[header]\r\n",
            Message::Command
        ),
        (
            b"* 12 FETCH (BODY[HEADER] {342}\r
Date: Wed, 17 Jul 1996 02:23:25 -0700 (PDT)\r
From: Terry Gray <gray@cac.washington.edu>\r
Subject: IMAP4rev1 WG mtg summary and minutes\r
To: imap@cac.washington.edu\r
cc: minutes@CNRI.Reston.VA.US, John Klensin <KLENSIN@MIT.EDU>\r
Message-Id: <B27397-0100000@cac.washington.edu>\r
MIME-Version: 1.0\r
Content-Type: TEXT/PLAIN; CHARSET=US-ASCII\r
\r
)\r\n",
            Message::Response
        ),
        (
            b"a004 OK FETCH completed\r\n",
            Message::Response
        ),
        (
            b"a005 store 12 +flags \\deleted\r\n",
            Message::Command,
        ),
        (
            b"* 12 FETCH (FLAGS (\\Seen \\Deleted))\r\n",
            Message::Response,
        ),
        (
            b"a005 OK +FLAGS completed\r\n",
            Message::Response,
        ),
        (
            b"a006 logout\r\n",
            Message::Command,
        ),
        (
            b"* BYE IMAP4rev1 server terminating connection\r\n",
            Message::Response,
        ),
        (
            b"a006 OK LOGOUT completed\r\n",
            Message::Response,
        ),
    ]
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
