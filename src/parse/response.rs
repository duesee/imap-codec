use crate::{
    parse::{
        _range, auth_type,
        base64::base64,
        charset,
        core::{atom, nz_number},
        crlf,
        flag::flag_perm,
        mailbox::mailbox_data,
        message::message_data,
        sp, tag as imap_tag,
    },
    types::response::{Code, Continuation},
};
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case, take_while1},
    combinator::{map, opt},
    multi::{many0, many1},
    sequence::tuple,
    IResult,
};

// ----- greeting -----

/// greeting = "*" SP (resp-cond-auth / resp-cond-bye) CRLF
pub fn greeting(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        tag_no_case(b"*"),
        sp,
        alt((
            map(resp_cond_auth, |_| unimplemented!()),
            map(resp_cond_bye, |_| unimplemented!()),
        )),
        crlf,
    ));

    let (_remaining, _parsed_greeting) = parser(input)?;

    unimplemented!();
}

/// resp-cond-auth = ("OK" / "PREAUTH") SP resp-text
///                    ; Authentication condition
fn resp_cond_auth(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        alt((
            map(tag_no_case(b"OK"), |_| unimplemented!()),
            map(tag_no_case(b"PREAUTH"), |_| unimplemented!()),
        )),
        sp,
        resp_text,
    ));

    let (_remaining, _parsed_resp_cond_auth) = parser(input)?;

    unimplemented!();
}

/// resp-text = ["[" resp-text-code "]" SP] text
pub fn resp_text(input: &[u8]) -> IResult<&[u8], (Option<Code>, String)> {
    let parser = tuple((
        opt(map(
            tuple((tag(b"["), resp_text_code, tag(b"]"), sp)),
            |(_, code, _, _)| code,
        )),
        text,
    ));

    let (remaining, parsed_resp_text) = parser(input)?;

    Ok((remaining, parsed_resp_text))
}

/// text = 1*TEXT-CHAR
pub fn text(input: &[u8]) -> IResult<&[u8], String> {
    let parser = take_while1(is_text_char);

    let (remaining, parsed_text) = parser(input)?;

    Ok((
        remaining,
        String::from_utf8(parsed_text.to_owned()).unwrap(),
    ))
}

/// TEXT-CHAR = %x01-09 / %x0B-0C / %x0E-7F
///               ; mod: was <any CHAR except CR and LF>
pub fn is_text_char(c: u8) -> bool {
    match c {
        0x01..=0x09 | 0x0b..=0x0c | 0x0e..=0x7f => true,
        _ => false,
    }
}

/// ; errata id: 261
/// resp-text-code = "ALERT" /
///                  "BADCHARSET" [SP "(" charset *(SP charset) ")" ] /
///                  capability-data / "PARSE" /
///                  "PERMANENTFLAGS" SP "(" [flag-perm *(SP flag-perm)] ")" /
///                  "READ-ONLY" / "READ-WRITE" / "TRYCREATE" /
///                  "UIDNEXT" SP nz-number / "UIDVALIDITY" SP nz-number /
///                  "UNSEEN" SP nz-number /
///                  atom [SP 1*(%x01-09 / %x0B-0C / %x0E-5C / %x5E-7F)]
///                   ; mod: was atom [SP 1*<any TEXT-CHAR except "]">]
fn resp_text_code(input: &[u8]) -> IResult<&[u8], Code> {
    let parser = alt((
        map(tag_no_case(b"ALERT"), |_| unimplemented!()),
        map(
            tuple((
                tag_no_case(b"BADCHARSET"),
                opt(tuple((
                    sp,
                    tag_no_case(b"("),
                    charset,
                    many0(tuple((sp, charset))),
                    tag_no_case(b")"),
                ))),
            )),
            |_| unimplemented!(),
        ),
        map(capability_data, |_| unimplemented!()),
        map(tag_no_case(b"PARSE"), |_| unimplemented!()),
        map(
            tuple((
                tag_no_case(b"PERMANENTFLAGS"),
                sp,
                tag_no_case(b"("),
                opt(tuple((flag_perm, many0(tuple((sp, flag_perm)))))),
                tag_no_case(b")"),
            )),
            |_| unimplemented!(),
        ),
        map(tag_no_case(b"READ-ONLY"), |_| unimplemented!()),
        map(tag_no_case(b"READ-WRITE"), |_| unimplemented!()),
        map(tag_no_case(b"TRYCREATE"), |_| unimplemented!()),
        map(
            tuple((tag_no_case(b"UIDNEXT"), sp, nz_number)),
            |_| unimplemented!(),
        ),
        map(
            tuple((tag_no_case(b"UIDVALIDITY"), sp, nz_number)),
            |_| unimplemented!(),
        ),
        map(
            tuple((tag_no_case(b"UNSEEN"), sp, nz_number)),
            |_| unimplemented!(),
        ),
        map(
            tuple((
                atom,
                opt(tuple((
                    sp,
                    many1(alt((
                        map(_range(0x01, 0x09), |_| unimplemented!()),
                        map(_range(0x0b, 0x0c), |_| unimplemented!()),
                        map(_range(0x0e, 0x5c), |_| unimplemented!()),
                        map(_range(0x5e, 0x7f), |_| unimplemented!()),
                    ))),
                ))),
            )),
            |_| unimplemented!(),
        ),
    ));

    let (_remaining, _parsed_resp_text_code) = parser(input)?;

    unimplemented!();
}

/// capability-data = "CAPABILITY" *(SP capability) SP "IMAP4rev1" *(SP capability)
///                     ; Servers MUST implement the STARTTLS, AUTH=PLAIN,
///                     ; and LOGINDISABLED capabilities
///                     ; Servers which offer RFC 1730 compatibility MUST
///                     ; list "IMAP4" as the first capability.
pub fn capability_data(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        tag_no_case(b"CAPABILITY"),
        many0(tuple((sp, capability))),
        sp,
        tag_no_case(b"IMAP4rev1"),
        many0(tuple((sp, capability))),
    ));

    let (_remaining, _parsed_capability_data) = parser(input)?;

    unimplemented!();
}

/// capability = ("AUTH=" auth-type) / atom
///                ; New capabilities MUST begin with "X" or be
///                ; registered with IANA as standard or
///                ; standards-track
pub fn capability(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = alt((
        map(
            tuple((tag_no_case(b"AUTH="), auth_type)),
            |_| unimplemented!(),
        ),
        map(atom, |_| unimplemented!()),
    ));

    let (_remaining, _parsed_capability) = parser(input)?;

    unimplemented!();
}

/// resp-cond-bye = "BYE" SP resp-text
pub fn resp_cond_bye(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((tag_no_case(b"BYE"), sp, resp_text));

    let (_remaining, _parsed_resp_cond_bye) = parser(input)?;

    unimplemented!();
}

// ----- response -----

/// response = *(continue-req / response-data) response-done
pub fn response(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        many0(alt((
            map(continue_req, |_| unimplemented!()),
            map(response_data, |_| unimplemented!()),
        ))),
        response_done,
    ));

    let (_remaining, _parsed_response) = parser(input)?;

    unimplemented!();
}

/// continue-req = "+" SP (resp-text / base64) CRLF
pub fn continue_req(input: &[u8]) -> IResult<&[u8], Continuation> {
    let parser = tuple((
        tag_no_case(b"+"),
        sp,
        alt((
            map(resp_text, |(code, text)| Continuation::Basic { code, text }),
            map(base64, Continuation::Base64),
        )),
        crlf,
    ));

    let (remaining, (_, _, continuation, _)) = parser(input)?;

    Ok((remaining, continuation))
}

/// response-data = "*" SP (resp-cond-state / resp-cond-bye /
///                   mailbox-data / message-data / capability-data) CRLF
pub fn response_data(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        tag_no_case(b"*"),
        sp,
        alt((
            map(resp_cond_state, |_| unimplemented!()),
            map(resp_cond_bye, |_| unimplemented!()),
            map(mailbox_data, |_| unimplemented!()),
            map(message_data, |_| unimplemented!()),
            map(capability_data, |_| unimplemented!()),
        )),
        crlf,
    ));

    let (_remaining, _parsed_response_data) = parser(input)?;

    unimplemented!();
}

/// resp-cond-state = ("OK" / "NO" / "BAD") SP resp-text
///                     ; Status condition
pub fn resp_cond_state(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        alt((
            map(tag_no_case(b"OK"), |_| unimplemented!()),
            map(tag_no_case(b"NO"), |_| unimplemented!()),
            map(tag_no_case(b"BAD"), |_| unimplemented!()),
        )),
        sp,
        resp_text,
    ));

    let (_remaining, _parsed_resp_cond_state) = parser(input)?;

    unimplemented!();
}

/// response-done = response-tagged / response-fatal
pub fn response_done(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = alt((
        map(response_tagged, |_| unimplemented!()),
        map(response_fatal, |_| unimplemented!()),
    ));

    let (_remaining, _parsed_response_done) = parser(input)?;

    unimplemented!();
}

/// response-tagged = tag SP resp-cond-state CRLF
pub fn response_tagged(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((imap_tag, sp, resp_cond_state, crlf));

    let (_remaining, _parsed_response_tagged) = parser(input)?;

    unimplemented!();
}

/// response-fatal = "*" SP resp-cond-bye CRLF
///                    ; Server closes connection immediately
pub fn response_fatal(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((tag_no_case(b"*"), sp, resp_cond_bye, crlf));

    let (_remaining, _parsed_response_fatal) = parser(input)?;

    unimplemented!();
}
