use imap_codec::rfc3501::{
    command::command,
    response::{greeting, response},
};
use imap_types::codec::Encode;

enum Who {
    Client,
    Server,
}

struct TraceLines<'a> {
    trace: &'a [u8],
    offset: usize,
}

impl<'a> Iterator for TraceLines<'a> {
    type Item = (Who, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let input = &self.trace[self.offset..];

        if let Some(pos) = input.iter().position(|b| *b == b'\n') {
            let who = match &input[..3] {
                b"C: " => Who::Client,
                b"S: " => Who::Server,
                _ => panic!("Line must begin with \"C: \" or \"S: \"."),
            };

            self.offset += pos + 1;

            Some((who, &input[3..pos + 1]))
        } else {
            None
        }
    }
}

fn split_trace(trace: &[u8]) -> impl Iterator<Item = (Who, &[u8])> {
    TraceLines { trace, offset: 0 }
}

fn test_lines_of_trace(trace: &[u8]) {
    for (who, line) in split_trace(trace) {
        // Replace last "\n" with "\r\n".
        let line = {
            let mut line = line[..line.len().saturating_sub(1)].to_vec();
            line.extend_from_slice(b"\r\n");
            line
        };

        match who {
            Who::Client => {
                println!("C:          {}", String::from_utf8_lossy(&line).trim());
                let (rem, parsed) = command(&line).unwrap();
                assert!(rem.is_empty());
                println!("Parsed      {:?}", parsed);
                let mut serialized = Vec::new();
                parsed.encode(&mut serialized).unwrap();
                println!(
                    "Serialized: {}",
                    String::from_utf8_lossy(&serialized).trim()
                );
                let (rem, parsed2) = command(&serialized).unwrap();
                assert!(rem.is_empty());
                assert_eq!(parsed, parsed2);
                println!()
            }
            Who::Server => {
                println!("S:          {}", String::from_utf8_lossy(&line).trim());
                let (rem, parsed) = response(&line).unwrap();
                println!("Parsed:     {:?}", parsed);
                assert!(rem.is_empty());
                let mut serialized = Vec::new();
                parsed.encode(&mut serialized).unwrap();
                println!(
                    "Serialized: {}",
                    String::from_utf8_lossy(&serialized).trim()
                );
                let (rem, parsed2) = response(&serialized).unwrap();
                assert!(rem.is_empty());
                assert_eq!(parsed, parsed2);
                println!()
            }
        }
    }
}

#[cfg(feature = "starttls")]
#[test]
fn test_from_capability() {
    let trace = b"C: abcd CAPABILITY
S: * CAPABILITY IMAP4rev1 STARTTLS AUTH=GSSAPI LOGINDISABLED
S: abcd OK CAPABILITY completed
C: efgh STARTTLS
S: efgh OK STARTLS completed
C: ijkl CAPABILITY
S: * CAPABILITY IMAP4rev1 AUTH=GSSAPI AUTH=PLAIN
S: ijkl OK CAPABILITY completed
";

    test_lines_of_trace(trace);
}

#[test]
fn test_from_noop() {
    let trace = br#"C: a002 NOOP
S: a002 OK NOOP completed
C: a047 NOOP
S: * 22 EXPUNGE
S: * 23 EXISTS
S: * 3 RECENT
S: * 14 FETCH (FLAGS (\Seen \Deleted))
S: a047 OK NOOP completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_logout() {
    let trace = br#"C: A023 LOGOUT
S: * BYE IMAP4rev1 Server logging out
S: A023 OK LOGOUT completed
"#;

    test_lines_of_trace(trace);
}

#[cfg(feature = "starttls")]
#[test]
fn test_from_starttls() {
    let trace = br#"C: a001 CAPABILITY
S: * CAPABILITY IMAP4rev1 STARTTLS LOGINDISABLED
S: a001 OK CAPABILITY completed
C: a002 STARTTLS
S: a002 OK Begin TLS negotiation now
C: a003 CAPABILITY
S: * CAPABILITY IMAP4rev1 AUTH=PLAIN
S: a003 OK CAPABILITY completed
C: a004 LOGIN joe password
S: a004 OK LOGIN completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_authenticate() {
    // S: * OK IMAP4rev1 Server
    // C: A001 AUTHENTICATE GSSAPI
    // S: +
    // C: YIIB+wYJKoZIhvcSAQICAQBuggHqMIIB5qADAgEFoQMCAQ6iBw
    //    MFACAAAACjggEmYYIBIjCCAR6gAwIBBaESGxB1Lndhc2hpbmd0
    //    b24uZWR1oi0wK6ADAgEDoSQwIhsEaW1hcBsac2hpdmFtcy5jYW
    //    Mud2FzaGluZ3Rvbi5lZHWjgdMwgdCgAwIBAaEDAgEDooHDBIHA
    //    cS1GSa5b+fXnPZNmXB9SjL8Ollj2SKyb+3S0iXMljen/jNkpJX
    //    AleKTz6BQPzj8duz8EtoOuNfKgweViyn/9B9bccy1uuAE2HI0y
    //    C/PHXNNU9ZrBziJ8Lm0tTNc98kUpjXnHZhsMcz5Mx2GR6dGknb
    //    I0iaGcRerMUsWOuBmKKKRmVMMdR9T3EZdpqsBd7jZCNMWotjhi
    //    vd5zovQlFqQ2Wjc2+y46vKP/iXxWIuQJuDiisyXF0Y8+5GTpAL
    //    pHDc1/pIGmMIGjoAMCAQGigZsEgZg2on5mSuxoDHEA1w9bcW9n
    //    FdFxDKpdrQhVGVRDIzcCMCTzvUboqb5KjY1NJKJsfjRQiBYBdE
    //    NKfzK+g5DlV8nrw81uOcP8NOQCLR5XkoMHC0Dr/80ziQzbNqhx
    //    O6652Npft0LQwJvenwDI13YxpwOdMXzkWZN/XrEqOWp6GCgXTB
    //    vCyLWLlWnbaUkZdEYbKHBPjd8t/1x5Yg==
    // S: + YGgGCSqGSIb3EgECAgIAb1kwV6ADAgEFoQMCAQ+iSzBJoAMC
    //    AQGiQgRAtHTEuOP2BXb9sBYFR4SJlDZxmg39IxmRBOhXRKdDA0
    //    uHTCOT9Bq3OsUTXUlk0CsFLoa8j+gvGDlgHuqzWHPSQg==
    // C:
    // S: + YDMGCSqGSIb3EgECAgIBAAD/////6jcyG4GE3KkTzBeBiVHe
    //    ceP2CWY0SR0fAQAgAAQEBAQ=
    // C: YDMGCSqGSIb3EgECAgIBAAD/////3LQBHXTpFfZgrejpLlLImP
    //    wkhbfa2QteAQAgAG1yYwE=
    // S: A001 OK GSSAPI authentication successful
}

#[test]
fn test_from_login() {
    let trace = b"C: a001 LOGIN SMITH SESAME
S: a001 OK LOGIN completed
";

    test_lines_of_trace(trace);
}

#[test]
fn test_from_select() {
    let trace = br#"C: A142 SELECT INBOX
S: * 172 EXISTS
S: * 1 RECENT
S: * OK [UNSEEN 12] Message 12 is first unseen
S: * OK [UIDVALIDITY 3857529045] UIDs valid
S: * OK [UIDNEXT 4392] Predicted next UID
S: * FLAGS (\Answered \Flagged \Deleted \Seen \Draft)
S: * OK [PERMANENTFLAGS (\Deleted \Seen \*)] Limited
S: A142 OK [READ-WRITE] SELECT completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_examine() {
    let trace = br#"C: A932 EXAMINE blurdybloop
S: * 17 EXISTS
S: * 2 RECENT
S: * OK [UNSEEN 8] Message 8 is first unseen
S: * OK [UIDVALIDITY 3857529045] UIDs valid
S: * OK [UIDNEXT 4392] Predicted next UID
S: * FLAGS (\Answered \Flagged \Deleted \Seen \Draft)
S: * OK [PERMANENTFLAGS ()] No permanent flags permitted
S: A932 OK [READ-ONLY] EXAMINE completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_create() {
    let trace = br#"C: A003 CREATE owatagusiam/
S: A003 OK CREATE completed
C: A004 CREATE owatagusiam/blurdybloop
S: A004 OK CREATE completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_delete() {
    let trace = br#"C: A682 LIST "" *
S: * LIST () "/" blurdybloop
S: * LIST (\Noselect) "/" foo
S: * LIST () "/" foo/bar
S: A682 OK LIST completed
C: A683 DELETE blurdybloop
S: A683 OK DELETE completed
C: A684 DELETE foo
S: A684 NO Name "foo" has inferior hierarchical names
C: A685 DELETE foo/bar
S: A685 OK DELETE Completed
C: A686 LIST "" *
S: * LIST (\Noselect) "/" foo
S: A686 OK LIST completed
C: A687 DELETE foo
S: A687 OK DELETE Completed
C: A82 LIST "" *
S: * LIST () "." blurdybloop
S: * LIST () "." foo
S: * LIST () "." foo.bar
S: A82 OK LIST completed
C: A83 DELETE blurdybloop
S: A83 OK DELETE completed
C: A84 DELETE foo
S: A84 OK DELETE Completed
C: A85 LIST "" *
S: * LIST () "." foo.bar
S: A85 OK LIST completed
C: A86 LIST "" %
S: * LIST (\Noselect) "." foo
S: A86 OK LIST completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_rename() {
    let trace = br#"C: A682 LIST "" *
S: * LIST () "/" blurdybloop
S: * LIST (\Noselect) "/" foo
S: * LIST () "/" foo/bar
S: A682 OK LIST completed
C: A683 RENAME blurdybloop sarasoop
S: A683 OK RENAME completed
C: A684 RENAME foo zowie
S: A684 OK RENAME Completed
C: A685 LIST "" *
S: * LIST () "/" sarasoop
S: * LIST (\Noselect) "/" zowie
S: * LIST () "/" zowie/bar
S: A685 OK LIST completed
C: Z432 LIST "" *
S: * LIST () "." INBOX
S: * LIST () "." INBOX.bar
S: Z432 OK LIST completed
C: Z433 RENAME INBOX old-mail
S: Z433 OK RENAME completed
C: Z434 LIST "" *
S: * LIST () "." INBOX
S: * LIST () "." INBOX.bar
S: * LIST () "." old-mail
S: Z434 OK LIST completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_subscribe() {
    let trace = br#"C: A002 SUBSCRIBE #news.comp.mail.mime
S: A002 OK SUBSCRIBE completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_unsubscribe() {
    let trace = br#"C: A002 UNSUBSCRIBE #news.comp.mail.mime
S: A002 OK UNSUBSCRIBE completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_list() {
    let trace = br#"C: A101 LIST "" ""
S: * LIST (\Noselect) "/" ""
S: A101 OK LIST Completed
C: A102 LIST #news.comp.mail.misc ""
S: * LIST (\Noselect) "." #news.
S: A102 OK LIST Completed
C: A103 LIST /usr/staff/jones ""
S: * LIST (\Noselect) "/" /
S: A103 OK LIST Completed
C: A202 LIST ~/Mail/ %
S: * LIST (\Noselect) "/" ~/Mail/foo
S: * LIST () "/" ~/Mail/meetings
S: A202 OK LIST completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_lsub() {
    let trace = br#"C: A002 LSUB "news." "comp.mail.*"
S: * LSUB () "." #news.comp.mail.mime
S: * LSUB () "." #news.comp.mail.misc
S: A002 OK LSUB completed
C: A003 LSUB "news." "comp.%"
S: * LSUB (\NoSelect) "." #news.comp.mail
S: A003 OK LSUB completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_status() {
    let trace = br#"C: A042 STATUS blurdybloop (UIDNEXT MESSAGES)
S: * STATUS blurdybloop (MESSAGES 231 UIDNEXT 44292)
S: A042 OK STATUS completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_append() {
    // C: A003 APPEND saved-messages (\Seen) {310}
    // S: + Ready for literal data
    // C: Date: Mon, 7 Feb 1994 21:52:25 -0800 (PST)
    // C: From: Fred Foobar <foobar@Blurdybloop.COM>
    // C: Subject: afternoon meeting
    // C: To: mooch@owatagu.siam.edu
    // C: Message-Id: <B27397-0100000@Blurdybloop.COM>
    // C: MIME-Version: 1.0
    // C: Content-Type: TEXT/PLAIN; CHARSET=US-ASCII
    // C:
    // C: Hello Joe, do you think we can meet at 3:30 tomorrow?
    // C:
    // S: A003 OK APPEND completed
}

#[test]
fn test_from_check() {
    let trace = br#"C: FXXZ CHECK
S: FXXZ OK CHECK Completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_close() {
    let trace = br#"C: A341 CLOSE
S: A341 OK CLOSE completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_expunge() {
    let trace = br#"C: A202 EXPUNGE
S: * 3 EXPUNGE
S: * 3 EXPUNGE
S: * 5 EXPUNGE
S: * 8 EXPUNGE
S: A202 OK EXPUNGE completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_search() {
    // C: A284 SEARCH CHARSET UTF-8 TEXT {6}
    // C: XXXXXX
    let trace = br#"C: A282 SEARCH FLAGGED SINCE 1-Feb-1994 NOT FROM "Smith"
S: * SEARCH 2 84 882
S: A282 OK SEARCH completed
C: A283 SEARCH TEXT "string not in mailbox"
S: * SEARCH
S: A283 OK SEARCH completed
S: * SEARCH 43
S: A284 OK SEARCH completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_fetch() {
    // S: * 2 FETCH ....
    // S: * 3 FETCH ....
    // S: * 4 FETCH ....
    let trace = br#"C: A654 FETCH 2:4 (FLAGS BODY[HEADER.FIELDS (DATE FROM)])
S: A654 OK FETCH completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_store() {
    let trace = br#"C: A003 STORE 2:4 +FLAGS (\Deleted)
S: * 2 FETCH (FLAGS (\Deleted \Seen))
S: * 3 FETCH (FLAGS (\Deleted))
S: * 4 FETCH (FLAGS (\Deleted \Flagged \Seen))
S: A003 OK STORE completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_copy() {
    let trace = br#"C: A003 COPY 2:4 MEETING
S: A003 OK COPY completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_from_uid() {
    let trace = br#"C: A999 UID FETCH 4827313:4828442 FLAGS
S: * 23 FETCH (FLAGS (\Seen) UID 4827313)
S: * 24 FETCH (FLAGS (\Seen) UID 4827943)
S: * 25 FETCH (FLAGS (\Seen) UID 4828442)
S: A999 OK UID FETCH completed
"#;

    test_lines_of_trace(trace);
}

//#[test]
//fn test_from_X() {
//    let trace = br#"C: a441 CAPABILITY
//S: * CAPABILITY IMAP4rev1 XPIG-LATIN
//S: a441 OK CAPABILITY completed
//C: A442 XPIG-LATIN
//S: * XPIG-LATIN ow-nay eaking-spay ig-pay atin-lay
//S: A442 OK XPIG-LATIN ompleted-cay"#;
//
//    test_lines_of_trace(trace);
//}

#[test]
fn test_transcript_from_rfc() {
    // S:  * 12 FETCH (BODY[HEADER] {342}
    // S:  Date: Wed, 17 Jul 1996 02:23:25 -0700 (PDT)
    // S:  From: Terry Gray <gray@cac.washington.edu>
    // S:  Subject: IMAP4rev1 WG mtg summary and minutes
    // S:  To: imap@cac.washington.edu
    // S:  cc: minutes@CNRI.Reston.VA.US, John Klensin <KLENSIN@MIT.EDU>
    // S:  Message-Id: <B27397-0100000@cac.washington.edu>
    // S:  MIME-Version: 1.0
    // S:  Content-Type: TEXT/PLAIN; CHARSET=US-ASCII
    // S:
    // S:  )

    let trace = br#"S: * OK IMAP4rev1 Service Ready
C: a001 login mrc secret
S: a001 OK LOGIN completed
C: a002 select inbox
S: * 18 EXISTS
S: * FLAGS (\Answered \Flagged \Deleted \Seen \Draft)
S: * 2 RECENT
S: * OK [UNSEEN 17] Message 17 is the first unseen message
S: * OK [UIDVALIDITY 3857529045] UIDs valid
S: a002 OK [READ-WRITE] SELECT completed
C: a003 fetch 12 full
S: * 12 FETCH (FLAGS (\Seen) INTERNALDATE "17-Jul-1996 02:44:25 -0700" RFC822.SIZE 4286 ENVELOPE ("Wed, 17 Jul 1996 02:23:25 -0700 (PDT)" "IMAP4rev1 WG mtg summary and minutes" (("Terry Gray" NIL "gray" "cac.washington.edu")) (("Terry Gray" NIL "gray" "cac.washington.edu")) (("Terry Gray" NIL "gray" "cac.washington.edu")) ((NIL NIL "imap" "cac.washington.edu")) ((NIL NIL "minutes" "CNRI.Reston.VA.US")("John Klensin" NIL "KLENSIN" "MIT.EDU")) NIL NIL "<B27397-0100000@cac.washington.edu>") BODY ("TEXT" "PLAIN" ("CHARSET" "US-ASCII") NIL NIL "7BIT" 3028 92))
S: a003 OK FETCH completed
C: a004 fetch 12 body[header]
S: a004 OK FETCH completed
C: a005 store 12 +flags \deleted
S: * 12 FETCH (FLAGS (\Seen \Deleted))
S: a005 OK +FLAGS completed
C: a006 logout
S: * BYE IMAP4rev1 server terminating connection
S: a006 OK LOGOUT completed
"#;

    test_lines_of_trace(trace);
}

#[test]
#[cfg(feature = "ext_enable")]
fn test_transcript_from_rfc5161() {
    let trace = br#"C: t1 CAPABILITY
S: * CAPABILITY IMAP4rev1 ID LITERAL+ ENABLE X-GOOD-IDEA
S: t1 OK foo
C: t2 ENABLE CONDSTORE X-GOOD-IDEA
S: * ENABLED X-GOOD-IDEA
S: t2 OK foo
C: t3 CAPABILITY
S: * CAPABILITY IMAP4rev1 ID LITERAL+ ENABLE X-GOOD-IDEA
S: t3 OK foo again
C: a1 ENABLE CONDSTORE
S: * ENABLED CONDSTORE
S: a1 OK Conditional Store enabled"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_response_status_ok() {
    let trace = br#"S: * OK IMAP4rev1 server ready
C: A001 LOGIN fred blurdybloop
S: * OK [ALERT] System shutdown in 10 minutes
S: A001 OK LOGIN Completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_response_status_no() {
    let trace = br#"C: A222 COPY 1:2 owatagusiam
S: * NO Disk is 98% full, please delete unnecessary data
S: A222 OK COPY completed
C: A223 COPY 3:200 blurdybloop
S: * NO Disk is 98% full, please delete unnecessary data
S: * NO Disk is 99% full, please delete unnecessary data
S: A223 NO COPY failed: disk is full
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_response_status_bad() {
    let trace = br#"S: * BAD Command line too long
S: * BAD Empty command line
C: A443 EXPUNGE
S: * BAD Disk crash, attempting salvage to a new disk!
S: * OK Salvage successful, no data lost
S: A443 OK Expunge completed
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_response_status_preauth() {
    // This can only be parsed with `greeting`
    let line = b"* PREAUTH IMAP4rev1 server logged in as Smith\r\n";

    println!("S:          {}", String::from_utf8_lossy(line).trim());
    let (rem, parsed) = greeting(line).unwrap();
    println!("Parsed:     {:?}", parsed);
    assert!(rem.is_empty());
    let mut serialized = Vec::new();
    parsed.encode(&mut serialized).unwrap();
    println!(
        "Serialized: {}",
        String::from_utf8_lossy(&serialized).trim()
    );
    let (rem, parsed2) = greeting(&serialized).unwrap();
    assert!(rem.is_empty());
    assert_eq!(parsed, parsed2);
    println!()
}

#[test]
fn test_response_status_bye() {
    let trace = br#"S: * BYE Autologout; idle for too long
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_response_data_capability() {
    let trace = br#"S: * CAPABILITY IMAP4rev1 STARTTLS AUTH=GSSAPI XPIG-LATIN
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_response_data_list() {
    let trace = br#"S: * LIST (\Noselect) "/" ~/Mail/foo
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_response_data_lsub() {
    let trace = br#"S: * LSUB () "." #news.comp.mail.misc
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_response_data_status() {
    let trace = br#"S: * STATUS blurdybloop (MESSAGES 231 UIDNEXT 44292)
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_response_data_search() {
    let trace = br#"S: * SEARCH 2 3 6
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_response_data_flags() {
    let trace = br#"S: * FLAGS (\Answered \Flagged \Deleted \Seen \Draft)
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_response_data_exists() {
    let trace = br#"S: * 23 EXISTS
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_response_data_recent() {
    let trace = br#"S: * 5 RECENT
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_response_data_expunge() {
    let trace = br#"S: * 44 EXPUNGE
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_response_data_fetch() {
    let trace = br#"S: * 23 FETCH (FLAGS (\Seen) RFC822.SIZE 44827)
"#;

    test_lines_of_trace(trace);
}

#[test]
fn test_response_data_continuation() {
    // C: A001 LOGIN {11}
    // C: FRED FOOBAR {7}
    // C: fat man
    // C: A044 BLURDYBLOOP {102856}

    let trace = br#"S: + Ready for additional command text
S: A001 OK LOGIN completed
S: A044 BAD No such command as "BLURDYBLOOP"
"#;

    test_lines_of_trace(trace);
}
