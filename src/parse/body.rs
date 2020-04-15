use crate::{
    parse::{
        core::{nil, nstring, number, string},
        dquote,
        envelope::envelope,
        sp,
    },
    types::core::{NString, String as IMAPString},
};
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    combinator::{map, opt},
    multi::{many0, many1, separated_nonempty_list},
    sequence::{delimited, tuple},
    IResult,
};

/// body = "(" (body-type-1part / body-type-mpart) ")"
pub fn body(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = delimited(
        tag(b"("),
        alt((
            map(body_type_1part, |_| unimplemented!()),
            map(body_type_mpart, |_| unimplemented!()),
        )),
        tag(b")"),
    );

    let (_remaining, _parsed_body) = parser(input)?;

    unimplemented!();
}

// --

/// body-type-1part = (body-type-basic / body-type-msg / body-type-text) [SP body-ext-1part]
pub fn body_type_1part(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        alt((
            map(body_type_basic, |_| unimplemented!()),
            map(body_type_msg, |_| unimplemented!()),
            map(body_type_text, |_| unimplemented!()),
        )),
        opt(tuple((sp, body_ext_1part))),
    ));

    let (_remaining, _parsed_body_type_1part) = parser(input)?;

    unimplemented!();
}

/// body-type-basic = media-basic SP body-fields
///                     ; MESSAGE subtype MUST NOT be "RFC822"
pub fn body_type_basic(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((media_basic, sp, body_fields));

    let (_remaining, _parsed_body_type_basic) = parser(input)?;

    unimplemented!();
}

/// body-type-msg   = media-message SP body-fields SP envelope SP body SP body-fld-lines
pub fn body_type_msg(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        media_message,
        sp,
        body_fields,
        sp,
        envelope,
        sp,
        body,
        sp,
        body_fld_lines,
    ));

    let (_remaining, _parsed_body_type_msg) = parser(input)?;

    unimplemented!();
}

/// body-type-text  = media-text SP body-fields SP body-fld-lines
pub fn body_type_text(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((media_text, sp, body_fields, sp, body_fld_lines));

    let (_remaining, _parsed_body_type_text) = parser(input)?;

    unimplemented!();
}

// ---

/// media-basic = ((DQUOTE ("APPLICATION" / "AUDIO" / "IMAGE" / "MESSAGE" / "VIDEO") DQUOTE) / string) SP media-subtype
///                 ; Defined in [MIME-IMT]
pub fn media_basic(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        alt((
            map(
                delimited(
                    dquote,
                    alt((
                        map(tag_no_case(b"APPLICATION"), |_| unimplemented!()),
                        map(tag_no_case(b"AUDIO"), |_| unimplemented!()),
                        map(tag_no_case(b"IMAGE"), |_| unimplemented!()),
                        map(tag_no_case(b"MESSAGE"), |_| unimplemented!()),
                        map(tag_no_case(b"VIDEO"), |_| unimplemented!()),
                    )),
                    dquote,
                ),
                |_| unimplemented!(),
            ),
            map(string, |_| unimplemented!()),
        )),
        sp,
        media_subtype,
    ));

    let (_remaining, _parsed_media_basic) = parser(input)?;

    unimplemented!();
}

/// media-subtype = string
///                   ; Defined in [MIME-IMT]
pub fn media_subtype(input: &[u8]) -> IResult<&[u8], IMAPString> {
    string(input)
}

// ---

/// body-fields = body-fld-param SP body-fld-id SP body-fld-desc SP body-fld-enc SP body-fld-octets
pub fn body_fields(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        body_fld_param,
        sp,
        body_fld_id,
        sp,
        body_fld_desc,
        sp,
        body_fld_enc,
        sp,
        body_fld_octets,
    ));

    let (_remaining, _parsed_body_fields) = parser(input)?;

    unimplemented!();
}

/// body-fld-param = "("
///                  string SP string *(SP string SP string)
///                  ")" /
///                  nil
pub fn body_fld_param(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = alt((
        map(
            delimited(
                tag(b"("),
                separated_nonempty_list(sp, tuple((string, sp, string))),
                tag(b")"),
            ),
            |_| unimplemented!(),
        ),
        map(nil, |_| unimplemented!()),
    ));

    let (_remaining, _parsed_body_fld_param) = parser(input)?;

    unimplemented!();
}

/// body-fld-id = nstring
pub fn body_fld_id(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

/// body-fld-desc = nstring
pub fn body_fld_desc(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

/// body-fld-enc = (DQUOTE ("7BIT" / "8BIT" / "BINARY" / "BASE64"/ "QUOTED-PRINTABLE") DQUOTE) / string
pub fn body_fld_enc(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = alt((
        map(
            delimited(
                dquote,
                alt((
                    map(tag_no_case(b"7BIT"), |_| unimplemented!()),
                    map(tag_no_case(b"8BIT"), |_| unimplemented!()),
                    map(tag_no_case(b"BINARY"), |_| unimplemented!()),
                    map(tag_no_case(b"BASE64"), |_| unimplemented!()),
                    map(tag_no_case(b"QUOTED-PRINTABLE"), |_| unimplemented!()),
                )),
                dquote,
            ),
            |_| unimplemented!(),
        ),
        map(string, |_| unimplemented!()),
    ));

    let (_remaining, _parsed_body_fld_enc) = parser(input)?;

    unimplemented!();
}

/// body-fld-octets = number
pub fn body_fld_octets(input: &[u8]) -> IResult<&[u8], u32> {
    number(input)
}

// ---

// envelope

// body

/// body-fld-lines = number
pub fn body_fld_lines(input: &[u8]) -> IResult<&[u8], u32> {
    number(input)
}

// ---

/// body-ext-1part = body-fld-md5 [SP body-fld-dsp [SP body-fld-lang [SP body-fld-loc *(SP body-extension)]]]
///                    ; MUST NOT be returned on non-extensible
///                    ; "BODY" fetch
pub fn body_ext_1part(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        body_fld_md5,
        opt(tuple((
            sp,
            body_fld_dsp,
            opt(tuple((
                sp,
                body_fld_lang,
                opt(tuple((
                    sp,
                    body_fld_loc,
                    many0(tuple((sp, body_extension))),
                ))),
            ))),
        ))),
    ));

    let (_remaining, _parsed_body_ext_1part) = parser(input)?;

    unimplemented!();
}

// ---

/// body-fld-md5 = nstring
pub fn body_fld_md5(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

/// body-fld-dsp = "(" string SP body-fld-param ")" / nil
pub fn body_fld_dsp(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = alt((
        map(
            delimited(tag(b"("), tuple((string, sp, body_fld_param)), tag(b")")),
            |_| unimplemented!(),
        ),
        map(nil, |_| unimplemented!()),
    ));

    let (_remaining, _parsed_body_fld_dsp) = parser(input)?;

    unimplemented!();
}

/// body-fld-lang = nstring / "(" string *(SP string) ")"
pub fn body_fld_lang(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = alt((
        map(nstring, |_| unimplemented!()),
        map(
            delimited(tag(b"("), separated_nonempty_list(sp, string), tag(b")")),
            |_| unimplemented!(),
        ),
    ));

    let (_remaining, _parsed_body_fld_lang) = parser(input)?;

    unimplemented!();
}

/// body-fld-loc = nstring
pub fn body_fld_loc(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

// ---

/// body-extension = nstring / number / "(" body-extension *(SP body-extension) ")"
///                    ; Future expansion.  Client implementations
///                    ; MUST accept body-extension fields.  Server
///                    ; implementations MUST NOT generate
///                    ; body-extension fields except as defined by
///                    ; future standard or standards-track
///                    ; revisions of this specification.
pub fn body_extension(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = alt((
        map(nstring, |_| unimplemented!()),
        map(number, |_| unimplemented!()),
        map(
            delimited(
                tag(b"("),
                separated_nonempty_list(sp, body_extension),
                tag(b")"),
            ),
            |_| unimplemented!(),
        ),
    ));

    let (_remaining, _parsed_body_extension) = parser(input)?;

    unimplemented!();
}

// ---

/// body-type-mpart = 1*body SP media-subtype [SP body-ext-mpart]
pub fn body_type_mpart(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        many1(body),
        sp,
        media_subtype,
        opt(tuple((sp, body_ext_mpart))),
    ));

    let (_remaining, _parsed_body_type_mpart) = parser(input)?;

    unimplemented!();
}

/// body-ext-mpart = body-fld-param [SP body-fld-dsp [SP body-fld-lang [SP body-fld-loc *(SP body-extension)]]]
///                    ; MUST NOT be returned on non-extensible
///                    ; "BODY" fetch
pub fn body_ext_mpart(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        body_fld_param,
        opt(tuple((
            sp,
            body_fld_dsp,
            opt(tuple((
                sp,
                body_fld_lang,
                opt(tuple((
                    sp,
                    body_fld_loc,
                    many0(tuple((sp, body_extension))),
                ))),
            ))),
        ))),
    ));

    let (_remaining, _parsed_body_ext_mpart) = parser(input)?;

    unimplemented!();
}

// "message" "rfc822" basic specific-for-message-rfc822 extension

/// media-message = DQUOTE "MESSAGE" DQUOTE SP DQUOTE "RFC822" DQUOTE
///                   ; Defined in [MIME-IMT]
pub fn media_message(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        dquote,
        tag_no_case(b"MESSAGE"),
        dquote,
        sp,
        dquote,
        tag_no_case(b"RFC822"),
        dquote,
    ));

    let (_remaining, _parsed_media_message) = parser(input)?;

    unimplemented!();
}

// "text" "?????" basic specific-for-text extension

/// media-text = DQUOTE "TEXT" DQUOTE SP media-subtype
///                ; Defined in [MIME-IMT]
pub fn media_text(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        delimited(dquote, tag_no_case(b"TEXT"), dquote),
        sp,
        media_subtype,
    ));

    let (_remaining, (_text, _, _media_subtype)) = parser(input)?;

    unimplemented!();
}
