use std::str::from_utf8;

use abnf_core::streaming::{CRLF, SP};
use base64::{engine::general_purpose::STANDARD as _base64, Engine};
/// Re-export everything from imap-types.
pub use imap_types::response::*;
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case, take_until, take_while},
    combinator::{map, map_res, opt, value},
    multi::separated_list1,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

#[cfg(feature = "ext_enable")]
use crate::extensions::enable::enable_data;
use crate::{
    core::{atom, charset, nz_number, tag_imap, text, NonEmptyVec, Text},
    fetch::msg_att,
    flag::flag_perm,
    mailbox::mailbox_data,
};

// ----- greeting -----

/// `greeting = "*" SP (resp-cond-auth / resp-cond-bye) CRLF`
pub fn greeting(input: &[u8]) -> IResult<&[u8], Greeting> {
    let mut parser = tuple((
        tag(b"*"),
        SP,
        alt((
            resp_cond_auth,
            map(resp_cond_bye, |resp_text| (GreetingKind::Bye, resp_text)),
        )),
        CRLF,
    ));

    let (remaining, (_, _, (kind, (code, text)), _)) = parser(input)?;

    Ok((remaining, Greeting { kind, code, text }))
}

/// `resp-cond-auth = ("OK" / "PREAUTH") SP resp-text`
///
/// Authentication condition
#[allow(clippy::type_complexity)]
pub fn resp_cond_auth(input: &[u8]) -> IResult<&[u8], (GreetingKind, (Option<Code>, Text))> {
    let mut parser = tuple((
        alt((
            value(GreetingKind::Ok, tag_no_case(b"OK")),
            value(GreetingKind::PreAuth, tag_no_case(b"PREAUTH")),
        )),
        SP,
        resp_text,
    ));

    let (remaining, (kind, _, resp_text)) = parser(input)?;

    Ok((remaining, (kind, resp_text)))
}

/// `resp-text = ["[" resp-text-code "]" SP] text`
pub fn resp_text(input: &[u8]) -> IResult<&[u8], (Option<Code>, Text)> {
    tuple((
        opt(terminated(
            preceded(
                tag(b"["),
                alt((
                    terminated(resp_text_code, tag(b"]")),
                    map(
                        terminated(take_while(|b: u8| b != b']'), tag(b"]")),
                        |bytes: &[u8]| Code::Other(CodeOther::unchecked(bytes)),
                    ),
                )),
            ),
            SP,
        )),
        text,
    ))(input)
}

/// `resp-text-code = "ALERT" /
///                   "BADCHARSET" [SP "(" charset *(SP charset) ")" ] /
///                   capability-data /
///                   "PARSE" /
///                   "PERMANENTFLAGS" SP "(" [flag-perm *(SP flag-perm)] ")" /
///                   "READ-ONLY" /
///                   "READ-WRITE" /
///                   "TRYCREATE" /
///                   "UIDNEXT" SP nz-number /
///                   "UIDVALIDITY" SP nz-number /
///                   "UNSEEN" SP nz-number /
///                   "COMPRESSIONACTIVE" ; RFC 4978
///                   atom [SP 1*<any TEXT-CHAR except "]">]`
///
/// Note: See errata id: 261
pub fn resp_text_code(input: &[u8]) -> IResult<&[u8], Code> {
    alt((
        value(Code::Alert, tag_no_case(b"ALERT")),
        map(
            tuple((
                tag_no_case(b"BADCHARSET"),
                opt(preceded(
                    SP,
                    delimited(tag(b"("), separated_list1(SP, charset), tag(b")")),
                )),
            )),
            |(_, maybe_charsets)| Code::BadCharset {
                allowed: maybe_charsets.unwrap_or_default(),
            },
        ),
        map(capability_data, Code::Capability),
        value(Code::Parse, tag_no_case(b"PARSE")),
        map(
            tuple((
                tag_no_case(b"PERMANENTFLAGS"),
                SP,
                delimited(
                    tag(b"("),
                    map(opt(separated_list1(SP, flag_perm)), |maybe_flags| {
                        maybe_flags.unwrap_or_default()
                    }),
                    tag(b")"),
                ),
            )),
            |(_, _, flags)| Code::PermanentFlags(flags),
        ),
        value(Code::ReadOnly, tag_no_case(b"READ-ONLY")),
        value(Code::ReadWrite, tag_no_case(b"READ-WRITE")),
        value(Code::TryCreate, tag_no_case(b"TRYCREATE")),
        map(
            tuple((tag_no_case(b"UIDNEXT"), SP, nz_number)),
            |(_, _, num)| Code::UidNext(num),
        ),
        map(
            tuple((tag_no_case(b"UIDVALIDITY"), SP, nz_number)),
            |(_, _, num)| Code::UidValidity(num),
        ),
        map(
            tuple((tag_no_case(b"UNSEEN"), SP, nz_number)),
            |(_, _, num)| Code::Unseen(num),
        ),
        #[cfg(feature = "ext_compress")]
        value(Code::CompressionActive, tag_no_case(b"COMPRESSIONACTIVE")),
        #[cfg(feature = "ext_quota")]
        value(Code::OverQuota, tag_no_case(b"OVERQUOTA")),
        #[cfg(feature = "ext_literal")]
        value(Code::TooBig, tag_no_case(b"TOOBIG")),
    ))(input)
}

/// `capability-data = "CAPABILITY" *(SP capability) SP "IMAP4rev1" *(SP capability)`
///
/// Servers MUST implement the STARTTLS, AUTH=PLAIN, and LOGINDISABLED capabilities
/// Servers which offer RFC 1730 compatibility MUST list "IMAP4" as the first capability.
pub fn capability_data(input: &[u8]) -> IResult<&[u8], NonEmptyVec<Capability>> {
    let mut parser = tuple((
        tag_no_case("CAPABILITY"),
        SP,
        separated_list1(SP, capability),
    ));

    let (rem, (_, _, caps)) = parser(input)?;

    Ok((rem, NonEmptyVec::unchecked(caps)))
}

/// `capability = ("AUTH=" auth-type) /
///               "COMPRESS=" algorithm / ; RFC 4978
///               atom`
pub fn capability(input: &[u8]) -> IResult<&[u8], Capability> {
    map(atom, Capability::from)(input)
}

/// `resp-cond-bye = "BYE" SP resp-text`
pub fn resp_cond_bye(input: &[u8]) -> IResult<&[u8], (Option<Code>, Text)> {
    let mut parser = tuple((tag_no_case(b"BYE"), SP, resp_text));

    let (remaining, (_, _, resp_text)) = parser(input)?;

    Ok((remaining, resp_text))
}

// ----- response -----

/// `response = *(continue-req / response-data) response-done`
pub fn response(input: &[u8]) -> IResult<&[u8], Response> {
    // Divert from standard here for better usability.
    // response_data already contains the bye response, thus
    // response_done could also be response_tagged.
    //
    // However, I will keep it as it is for now.
    alt((
        map(continue_req, Response::Continue),
        response_data,
        map(response_done, Response::Status),
    ))(input)
}

/// `continue-req = "+" SP (resp-text / base64) CRLF`
pub fn continue_req(input: &[u8]) -> IResult<&[u8], Continue> {
    // We can't map the output of `resp_text` directly to `Continue::basic()` because we might end
    // up with a subset of `Text` that is valid base64 and will panic on `unwrap()`. Thus, we first
    // let the parsing finish and only later map to `Continue`.

    // A helper struct to postpone the unification to `Continue` in the `alt` combinator below.
    enum Either<A, B> {
        Base64(A),
        Basic(B),
    }

    let mut parser = tuple((
        tag(b"+ "),
        alt((
            map(
                map_res(take_until("\r\n"), |input| _base64.decode(input)),
                Either::Base64,
            ),
            map(resp_text, Either::Basic),
        )),
        CRLF,
    ));

    let (remaining, (_, either, _)) = parser(input)?;

    let continue_request = match either {
        Either::Base64(data) => Continue::base64(data),
        Either::Basic((code, text)) => Continue::basic(code, text).unwrap(),
    };

    Ok((remaining, continue_request))
}

/// `response-data = "*" SP (
///                    resp-cond-state /
///                    resp-cond-bye /
///                    mailbox-data /
///                    message-data /
///                    capability-data
///                  ) CRLF`
pub fn response_data(input: &[u8]) -> IResult<&[u8], Response> {
    let mut parser = tuple((
        tag(b"*"),
        SP,
        alt((
            map(resp_cond_state, |(raw_status, code, text)| {
                let status = match raw_status.to_ascii_lowercase().as_ref() {
                    "ok" => Status::Ok {
                        tag: None,
                        code,
                        text,
                    },
                    "no" => Status::No {
                        tag: None,
                        code,
                        text,
                    },
                    "bad" => Status::Bad {
                        tag: None,
                        code,
                        text,
                    },
                    _ => unreachable!(),
                };

                Response::Status(status)
            }),
            map(resp_cond_bye, |(code, text)| {
                Response::Status(Status::Bye { code, text })
            }),
            map(mailbox_data, Response::Data),
            map(message_data, Response::Data),
            map(capability_data, |caps| {
                Response::Data(Data::Capability(caps))
            }),
            #[cfg(feature = "ext_enable")]
            map(enable_data, Response::Data),
        )),
        CRLF,
    ));

    let (remaining, (_, _, response, _)) = parser(input)?;

    Ok((remaining, response))
}

/// `resp-cond-state = ("OK" / "NO" / "BAD") SP resp-text`
///
/// Status condition
pub fn resp_cond_state(input: &[u8]) -> IResult<&[u8], (&str, Option<Code>, Text)> {
    let mut parser = tuple((
        alt((tag_no_case("OK"), tag_no_case("NO"), tag_no_case("BAD"))),
        SP,
        resp_text,
    ));

    let (remaining, (raw_status, _, (maybe_code, text))) = parser(input)?;

    Ok((
        remaining,
        // # Safety
        //
        // `raw_status` is always UTF-8.
        (from_utf8(raw_status).unwrap(), maybe_code, text),
    ))
}

/// `response-done = response-tagged / response-fatal`
pub fn response_done(input: &[u8]) -> IResult<&[u8], Status> {
    alt((response_tagged, response_fatal))(input)
}

/// `response-tagged = tag SP resp-cond-state CRLF`
pub fn response_tagged(input: &[u8]) -> IResult<&[u8], Status> {
    let mut parser = tuple((tag_imap, SP, resp_cond_state, CRLF));

    let (remaining, (tag, _, (raw_status, code, text), _)) = parser(input)?;

    let status = match raw_status.to_ascii_lowercase().as_ref() {
        "ok" => Status::Ok {
            tag: Some(tag),
            code,
            text,
        },
        "no" => Status::No {
            tag: Some(tag),
            code,
            text,
        },
        "bad" => Status::Bad {
            tag: Some(tag),
            code,
            text,
        },
        _ => unreachable!(),
    };

    Ok((remaining, status))
}

/// `response-fatal = "*" SP resp-cond-bye CRLF`
///
/// Server closes connection immediately
pub fn response_fatal(input: &[u8]) -> IResult<&[u8], Status> {
    let mut parser = tuple((tag(b"*"), SP, resp_cond_bye, CRLF));

    let (remaining, (_, _, (code, text), _)) = parser(input)?;

    Ok((remaining, { Status::Bye { code, text } }))
}

/// `message-data = nz-number SP ("EXPUNGE" / ("FETCH" SP msg-att))`
pub fn message_data(input: &[u8]) -> IResult<&[u8], Data> {
    let (remaining, seq_or_uid) = terminated(nz_number, SP)(input)?;

    alt((
        map(tag_no_case(b"EXPUNGE"), move |_| Data::Expunge(seq_or_uid)),
        map(
            tuple((tag_no_case(b"FETCH"), SP, msg_att)),
            move |(_, _, attributes)| Data::Fetch {
                seq_or_uid,
                attributes,
            },
        ),
    ))(remaining)
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::*;
    use crate::{
        body::{
            BasicFields, Body, BodyExtension, BodyStructure, Disposition, Language, Location,
            SinglePartExtensionData, SpecificFields,
        },
        core::{IString, NString, QuotedChar, Tag},
        flag::FlagNameAttribute,
        testing::{
            kat_inverse_continue, kat_inverse_greeting, kat_inverse_response,
            known_answer_test_encode,
        },
    };

    #[test]
    fn test_kat_inverse_greeting() {
        kat_inverse_greeting(&[
            (
                b"* OK [badcharset] ...\r\n".as_slice(),
                b"".as_slice(),
                Greeting::ok(Some(Code::BadCharset { allowed: vec![] }), "...").unwrap(),
            ),
            (
                b"* OK [UnSEEN 12345] ...\r\naaa".as_slice(),
                b"aaa".as_slice(),
                Greeting::ok(
                    Some(Code::Unseen(NonZeroU32::try_from(12345).unwrap())),
                    "...",
                )
                .unwrap(),
            ),
            (
                b"* OK [unseen 12345]  \r\n ".as_slice(),
                b" ".as_slice(),
                Greeting::ok(
                    Some(Code::Unseen(NonZeroU32::try_from(12345).unwrap())),
                    " ",
                )
                .unwrap(),
            ),
            (
                b"* PREAUTH [ALERT] hello\r\n".as_ref(),
                b"".as_ref(),
                Greeting::new(GreetingKind::PreAuth, Some(Code::Alert), "hello").unwrap(),
            ),
        ]);
    }

    #[test]
    fn test_kat_inverse_response_data() {
        kat_inverse_response(&[
            (
                b"* CAPABILITY IMAP4REV1\r\n".as_ref(),
                b"".as_ref(),
                Response::Data(Data::Capability(NonEmptyVec::from(Capability::Imap4Rev1))),
            ),
            (
                b"* LIST (\\Noselect) \"/\" bbb\r\n",
                b"",
                Response::Data(Data::List {
                    items: vec![FlagNameAttribute::Noselect],
                    delimiter: Some(QuotedChar::try_from('/').unwrap()),
                    mailbox: "bbb".try_into().unwrap(),
                }),
            ),
            (
                b"* SEARCH 1 2 3 42\r\n",
                b"",
                Response::Data(Data::Search(vec![
                    1.try_into().unwrap(),
                    2.try_into().unwrap(),
                    3.try_into().unwrap(),
                    42.try_into().unwrap(),
                ])),
            ),
            (b"* 42 EXISTS\r\n", b"", Response::Data(Data::Exists(42))),
            (
                b"* 12345 RECENT\r\n",
                b"",
                Response::Data(Data::Recent(12345)),
            ),
            (
                b"* 123 EXPUNGE\r\n",
                b"",
                Response::Data(Data::Expunge(123.try_into().unwrap())),
            ),
        ]);
    }

    #[test]
    fn test_kat_inverse_response_status() {
        kat_inverse_response(&[
            // tagged; Ok, No, Bad
            (
                b"A1 OK [ALERT] hello\r\n".as_ref(),
                b"".as_ref(),
                Response::Status(
                    Status::ok(
                        Some(Tag::try_from("A1").unwrap()),
                        Some(Code::Alert),
                        "hello",
                    )
                    .unwrap(),
                ),
            ),
            (
                b"A1 NO [ALERT] hello\r\n",
                b"".as_ref(),
                Response::Status(
                    Status::no(
                        Some(Tag::try_from("A1").unwrap()),
                        Some(Code::Alert),
                        "hello",
                    )
                    .unwrap(),
                ),
            ),
            (
                b"A1 BAD [ALERT] hello\r\n",
                b"".as_ref(),
                Response::Status(
                    Status::bad(
                        Some(Tag::try_from("A1").unwrap()),
                        Some(Code::Alert),
                        "hello",
                    )
                    .unwrap(),
                ),
            ),
            (
                b"A1 OK hello\r\n",
                b"".as_ref(),
                Response::Status(
                    Status::ok(Some(Tag::try_from("A1").unwrap()), None, "hello").unwrap(),
                ),
            ),
            (
                b"A1 NO hello\r\n",
                b"".as_ref(),
                Response::Status(
                    Status::no(Some(Tag::try_from("A1").unwrap()), None, "hello").unwrap(),
                ),
            ),
            (
                b"A1 BAD hello\r\n",
                b"".as_ref(),
                Response::Status(
                    Status::bad(Some(Tag::try_from("A1").unwrap()), None, "hello").unwrap(),
                ),
            ),
            // untagged; Ok, No, Bad
            (
                b"* OK [ALERT] hello\r\n",
                b"".as_ref(),
                Response::Status(Status::ok(None, Some(Code::Alert), "hello").unwrap()),
            ),
            (
                b"* NO [ALERT] hello\r\n",
                b"".as_ref(),
                Response::Status(Status::no(None, Some(Code::Alert), "hello").unwrap()),
            ),
            (
                b"* BAD [ALERT] hello\r\n",
                b"".as_ref(),
                Response::Status(Status::bad(None, Some(Code::Alert), "hello").unwrap()),
            ),
            (
                b"* OK hello\r\n",
                b"".as_ref(),
                Response::Status(Status::ok(None, None, "hello").unwrap()),
            ),
            (
                b"* NO hello\r\n",
                b"".as_ref(),
                Response::Status(Status::no(None, None, "hello").unwrap()),
            ),
            (
                b"* BAD hello\r\n",
                b"".as_ref(),
                Response::Status(Status::bad(None, None, "hello").unwrap()),
            ),
            // bye
            (
                b"* BYE [ALERT] hello\r\n",
                b"".as_ref(),
                Response::Status(Status::bye(Some(Code::Alert), "hello").unwrap()),
            ),
        ]);
    }

    #[test]
    fn test_kat_inverse_continue() {
        kat_inverse_continue(&[
            (
                b"+ \x01\r\n".as_ref(),
                b"".as_ref(),
                Continue::basic(None, "\x01").unwrap(),
            ),
            (
                b"+ hello\r\n".as_ref(),
                b"".as_ref(),
                Continue::basic(None, "hello").unwrap(),
            ),
            (
                b"+ [READ-WRITE] hello\r\n",
                b"",
                Continue::basic(Some(Code::ReadWrite), "hello").unwrap(),
            ),
        ]);
    }

    #[test]
    fn test_encode_body_structure() {
        let tests = [
            (
                BodyStructure::Single {
                    body: Body {
                        basic: BasicFields {
                            parameter_list: vec![],
                            id: NString(None),
                            description: NString::try_from("description").unwrap(),
                            content_transfer_encoding: IString::try_from("cte").unwrap(),
                            size: 123,
                        },
                        specific: SpecificFields::Basic {
                            r#type: IString::try_from("application").unwrap(),
                            subtype: IString::try_from("voodoo").unwrap(),
                        },
                    },
                    extension_data: None,
                },
                b"(\"application\" \"voodoo\" NIL NIL \"description\" \"cte\" 123)".as_ref(),
            ),
            (
                BodyStructure::Single {
                    body: Body {
                        basic: BasicFields {
                            parameter_list: vec![],
                            id: NString(None),
                            description: NString::try_from("description").unwrap(),
                            content_transfer_encoding: IString::try_from("cte").unwrap(),
                            size: 123,
                        },
                        specific: SpecificFields::Text {
                            subtype: IString::try_from("plain").unwrap(),
                            number_of_lines: 14,
                        },
                    },
                    extension_data: None,
                },
                b"(\"TEXT\" \"plain\" NIL NIL \"description\" \"cte\" 123 14)",
            ),
            (
                BodyStructure::Single {
                    body: Body {
                        basic: BasicFields {
                            parameter_list: vec![],
                            id: NString(None),
                            description: NString::try_from("description").unwrap(),
                            content_transfer_encoding: IString::try_from("cte").unwrap(),
                            size: 123,
                        },
                        specific: SpecificFields::Text {
                            subtype: IString::try_from("plain").unwrap(),
                            number_of_lines: 14,
                        },
                    },
                    extension_data: Some(SinglePartExtensionData {
                        md5: NString::try_from("AABB").unwrap(),
                        tail: Some(Disposition {
                            disposition: None,
                            tail: Some(Language {
                                language: vec![],
                                tail: Some(Location{
                                    location: NString(None),
                                    extensions: vec![BodyExtension::List(NonEmptyVec::from(BodyExtension::Number(1337)))],
                                })
                            })
                        })
                    }),
                },
                b"(\"TEXT\" \"plain\" NIL NIL \"description\" \"cte\" 123 14 \"AABB\" NIL NIL NIL (1337))",
            ),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }

    #[test]
    fn test_parse_response_negative() {
        let tests = [b"+ Nose[CAY a\r\n".as_ref()];

        for test in tests {
            assert!(response(test).is_err());
        }
    }
}
