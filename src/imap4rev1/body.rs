use std::{borrow::Cow, convert::TryFrom};

use abnf_core::streaming::SP;
use imap_types::{
    core::{IString, NString, NonEmptyVec},
    response::data::{
        BasicFields, Body, BodyStructure, MultiPartExtensionData, SinglePartExtensionData,
        SpecificFields,
    },
};
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    combinator::{map, opt, recognize},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, preceded, tuple},
    IResult,
};

use crate::imap4rev1::{
    core::{nil, nstring, number, string},
    envelope::envelope,
};

/// `body = "(" (body-type-1part / body-type-mpart) ")"`
///
/// Note: This parser is recursively defined. Thus, in order to not overflow the stack,
/// it is needed to limit how may recursions are allowed. (8 should suffice).
pub fn body(remaining_recursions: usize) -> impl Fn(&[u8]) -> IResult<&[u8], BodyStructure> {
    move |input: &[u8]| body_limited(input, remaining_recursions)
}

fn body_limited<'a>(
    input: &'a [u8],
    remaining_recursions: usize,
) -> IResult<&'a [u8], BodyStructure> {
    if remaining_recursions == 0 {
        return Err(nom::Err::Failure(nom::error::make_error(
            input,
            nom::error::ErrorKind::TooLarge,
        )));
    }

    let body_type_1part = move |input: &'a [u8]| {
        body_type_1part_limited(input, remaining_recursions.saturating_sub(1))
    };
    let body_type_mpart = move |input: &'a [u8]| {
        body_type_mpart_limited(input, remaining_recursions.saturating_sub(1))
    };

    delimited(
        tag(b"("),
        alt((body_type_1part, body_type_mpart)),
        tag(b")"),
    )(input)
}

/// `body-type-1part = (
///                     body-type-basic /
///                     body-type-msg /
///                     body-type-text
///                    )
///                    [SP body-ext-1part]`
///
/// Note: This parser is recursively defined. Thus, in order to not overflow the stack,
/// it is needed to limit how may recursions are allowed.
fn body_type_1part_limited<'a>(
    input: &'a [u8],
    remaining_recursions: usize,
) -> IResult<&'a [u8], BodyStructure> {
    if remaining_recursions == 0 {
        return Err(nom::Err::Failure(nom::error::make_error(
            input,
            nom::error::ErrorKind::TooLarge,
        )));
    }

    let body_type_msg =
        move |input: &'a [u8]| body_type_msg_limited(input, remaining_recursions.saturating_sub(1));

    let mut parser = tuple((
        alt((body_type_msg, body_type_text, body_type_basic)),
        opt(preceded(SP, body_ext_1part)),
    ));

    let (remaining, ((basic, specific), maybe_extension)) = parser(input)?;

    Ok((
        remaining,
        BodyStructure::Single {
            body: Body { basic, specific },
            extension: maybe_extension,
        },
    ))
}

/// `body-type-basic = media-basic SP body-fields`
///
/// MESSAGE subtype MUST NOT be "RFC822"
pub fn body_type_basic(input: &[u8]) -> IResult<&[u8], (BasicFields, SpecificFields)> {
    let mut parser = tuple((media_basic, SP, body_fields));

    let (remaining, ((type_, subtype), _, basic)) = parser(input)?;

    Ok((remaining, (basic, SpecificFields::Basic { type_, subtype })))
}

/// `body-type-msg = media-message SP
///                 body-fields SP
///                 envelope SP
///                 body SP
///                 body-fld-lines`
///
/// Note: This parser is recursively defined. Thus, in order to not overflow the stack,
/// it is needed to limit how may recursions are allowed. (8 should suffice).
fn body_type_msg_limited<'a>(
    input: &'a [u8],
    remaining_recursions: usize,
) -> IResult<&'a [u8], (BasicFields, SpecificFields)> {
    if remaining_recursions == 0 {
        return Err(nom::Err::Failure(nom::error::make_error(
            input,
            nom::error::ErrorKind::TooLarge,
        )));
    }

    let body = move |input: &'a [u8]| body_limited(input, remaining_recursions.saturating_sub(1));

    let mut parser = tuple((
        media_message,
        SP,
        body_fields,
        SP,
        envelope,
        SP,
        body,
        SP,
        body_fld_lines,
    ));

    let (remaining, (_, _, basic, _, envelope, _, body_structure, _, number_of_lines)) =
        parser(input)?;

    Ok((
        remaining,
        (
            basic,
            SpecificFields::Message {
                envelope: Box::new(envelope),
                body_structure: Box::new(body_structure),
                number_of_lines,
            },
        ),
    ))
}

/// `body-type-text = media-text SP
///                   body-fields SP
///                   body-fld-lines`
pub fn body_type_text(input: &[u8]) -> IResult<&[u8], (BasicFields, SpecificFields)> {
    let mut parser = tuple((media_text, SP, body_fields, SP, body_fld_lines));

    let (remaining, (subtype, _, basic, _, number_of_lines)) = parser(input)?;

    Ok((
        remaining,
        (
            basic,
            SpecificFields::Text {
                subtype,
                number_of_lines,
            },
        ),
    ))
}

/// `body-fields = body-fld-param SP
///                body-fld-id SP
///                body-fld-desc SP
///                body-fld-enc SP
///                body-fld-octets`
pub fn body_fields(input: &[u8]) -> IResult<&[u8], BasicFields> {
    let mut parser = tuple((
        body_fld_param,
        SP,
        body_fld_id,
        SP,
        body_fld_desc,
        SP,
        body_fld_enc,
        SP,
        body_fld_octets,
    ));

    let (remaining, (parameter_list, _, id, _, description, _, content_transfer_encoding, _, size)) =
        parser(input)?;

    Ok((
        remaining,
        BasicFields {
            parameter_list,
            id,
            description,
            content_transfer_encoding,
            size,
        },
    ))
}

/// `body-fld-param = "("
///                   string SP
///                   string *(SP string SP string)
///                   ")" / nil`
pub fn body_fld_param(input: &[u8]) -> IResult<&[u8], Vec<(IString, IString)>> {
    let mut parser = alt((
        delimited(
            tag(b"("),
            separated_list1(
                SP,
                map(tuple((string, SP, string)), |(key, _, value)| (key, value)),
            ),
            tag(b")"),
        ),
        map(nil, |_| vec![]),
    ));

    let (remaining, parsed_body_fld_param) = parser(input)?;

    Ok((remaining, parsed_body_fld_param))
}

#[inline]
/// `body-fld-id = nstring`
pub fn body_fld_id(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

#[inline]
/// `body-fld-desc = nstring`
pub fn body_fld_desc(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

#[inline]
/// `body-fld-enc = (
///                   DQUOTE (
///                     "7BIT" /
///                     "8BIT" /
///                     "BINARY" /
///                     "BASE64"/
///                     "QUOTED-PRINTABLE"
///                   ) DQUOTE
///                 ) / string`
///
/// Simplified...
///
/// `body-fld-enc = string`
///
/// TODO: why the special case?
pub fn body_fld_enc(input: &[u8]) -> IResult<&[u8], IString> {
    string(input)
}

#[inline]
/// `body-fld-octets = number`
pub fn body_fld_octets(input: &[u8]) -> IResult<&[u8], u32> {
    number(input)
}

#[inline]
/// `body-fld-lines = number`
pub fn body_fld_lines(input: &[u8]) -> IResult<&[u8], u32> {
    number(input)
}

/// `body-ext-1part = body-fld-md5
///                   [SP body-fld-dsp
///                     [SP body-fld-lang
///                       [SP body-fld-loc *(SP body-extension)]
///                     ]
///                   ]`
///
/// MUST NOT be returned on non-extensible "BODY" fetch
///
/// TODO(cleanup): this is insane... define macro?
pub fn body_ext_1part(input: &[u8]) -> IResult<&[u8], SinglePartExtensionData> {
    let mut disposition = None;
    let mut language = None;
    let mut location = None;
    let mut extension = Cow::Borrowed(&b""[..]);

    let (mut rem, md5) = body_fld_md5(input)?;

    let (rem_, dsp_) = opt(preceded(SP, body_fld_dsp))(rem)?;
    if let Some(dsp_) = dsp_ {
        rem = rem_;
        disposition = Some(dsp_);

        let (rem_, lang_) = opt(preceded(SP, body_fld_lang))(rem)?;
        if let Some(lang_) = lang_ {
            rem = rem_;
            language = Some(lang_);

            let (rem_, loc_) = opt(preceded(SP, body_fld_loc))(rem)?;
            if let Some(loc_) = loc_ {
                rem = rem_;
                location = Some(loc_);

                let (rem_, ext_) = recognize(many0(preceded(SP, body_extension(8))))(rem)?;
                rem = rem_;
                extension = Cow::Borrowed(ext_);
            }
        }
    }

    Ok((
        rem,
        SinglePartExtensionData {
            md5,
            disposition,
            language,
            location,
            extension,
        },
    ))
}

#[inline]
/// `body-fld-md5 = nstring`
pub fn body_fld_md5(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

/// `body-fld-dsp = "(" string SP body-fld-param ")" / nil`
#[allow(clippy::type_complexity)]
pub fn body_fld_dsp(input: &[u8]) -> IResult<&[u8], Option<(IString, Vec<(IString, IString)>)>> {
    alt((
        delimited(
            tag(b"("),
            map(
                tuple((string, SP, body_fld_param)),
                |(string, _, body_fld_param)| Some((string, body_fld_param)),
            ),
            tag(b")"),
        ),
        map(nil, |_| None),
    ))(input)
}

/// `body-fld-lang = nstring / "(" string *(SP string) ")"`
pub fn body_fld_lang(input: &[u8]) -> IResult<&[u8], Vec<IString>> {
    alt((
        map(nstring, |nstring| match nstring.0 {
            Some(item) => vec![item],
            None => vec![],
        }),
        delimited(tag(b"("), separated_list1(SP, string), tag(b")")),
    ))(input)
}

#[inline]
/// `body-fld-loc = nstring`
pub fn body_fld_loc(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

/// `body-extension = nstring /
///                   number /
///                   "(" body-extension *(SP body-extension) ")"`
///
/// Future expansion.
///
/// Client implementations MUST accept body-extension fields.
/// Server implementations MUST NOT generate body-extension fields except as defined by
/// future standard or standards-track revisions of this specification.
///
/// Note: This parser is recursively defined. Thus, in order to not overflow the stack,
/// it is needed to limit how may recursions are allowed. (8 should suffice).
///
/// FIXME: This recognizes extension data and returns &[u8].
pub fn body_extension(remaining_recursions: usize) -> impl Fn(&[u8]) -> IResult<&[u8], &[u8]> {
    move |input: &[u8]| body_extension_limited(input, remaining_recursions)
}

fn body_extension_limited<'a>(
    input: &'a [u8],
    remaining_recursion: usize,
) -> IResult<&'a [u8], &[u8]> {
    if remaining_recursion == 0 {
        return Err(nom::Err::Failure(nom::error::make_error(
            input,
            nom::error::ErrorKind::TooLarge,
        )));
    }

    let body_extension =
        move |input: &'a [u8]| body_extension_limited(input, remaining_recursion.saturating_sub(1));

    alt((
        recognize(nstring),
        recognize(number),
        recognize(delimited(
            tag(b"("),
            separated_list1(SP, body_extension),
            tag(b")"),
        )),
    ))(input)
}

// ---

/// `body-type-mpart = 1*body SP media-subtype [SP body-ext-mpart]`
///
/// Note: This parser is recursively defined. Thus, in order to not overflow the stack,
/// it is needed to limit how may recursions are allowed.
fn body_type_mpart_limited(
    input: &[u8],
    remaining_recursion: usize,
) -> IResult<&[u8], BodyStructure> {
    if remaining_recursion == 0 {
        return Err(nom::Err::Failure(nom::error::make_error(
            input,
            nom::error::ErrorKind::TooLarge,
        )));
    }

    let mut parser = tuple((
        many1(body(remaining_recursion)),
        SP,
        media_subtype,
        opt(preceded(SP, body_ext_mpart)),
    ));

    let (remaining, (bodies, _, subtype, extension_data)) = parser(input)?;

    Ok((
        remaining,
        BodyStructure::Multi {
            // Safety: `unwrap` can't panic due to the use of `many1`.
            bodies: NonEmptyVec::try_from(bodies).unwrap(),
            subtype,
            extension_data,
        },
    ))
}

/// `body-ext-mpart = body-fld-param
///                   [SP body-fld-dsp
///                     [SP body-fld-lang
///                       [SP body-fld-loc *(SP body-extension)]
///                     ]
///                   ]`
///
/// MUST NOT be returned on non-extensible "BODY" fetch
///
/// TODO(cleanup): this is insane, too... define macro?
pub fn body_ext_mpart(input: &[u8]) -> IResult<&[u8], MultiPartExtensionData> {
    let mut disposition = None;
    let mut language = None;
    let mut location = None;
    let mut extension = Cow::Borrowed(&b""[..]);

    let (mut rem, parameter_list) = body_fld_param(input)?;

    let (rem_, dsp_) = opt(preceded(SP, body_fld_dsp))(rem)?;
    if let Some(dsp_) = dsp_ {
        rem = rem_;
        disposition = Some(dsp_);

        let (rem_, lang_) = opt(preceded(SP, body_fld_lang))(rem)?;
        if let Some(lang_) = lang_ {
            rem = rem_;
            language = Some(lang_);

            let (rem_, loc_) = opt(preceded(SP, body_fld_loc))(rem)?;
            if let Some(loc_) = loc_ {
                rem = rem_;
                location = Some(loc_);

                let (rem_, ext_) = recognize(many0(preceded(SP, body_extension(8))))(rem)?;
                rem = rem_;
                extension = Cow::Borrowed(ext_);
            }
        }
    }

    Ok((
        rem,
        MultiPartExtensionData {
            parameter_list,
            disposition,
            language,
            location,
            extension,
        },
    ))
}

// ---

/// `media-basic = (
///                  ( DQUOTE
///                    (
///                      "APPLICATION" /
///                      "AUDIO" /
///                      "IMAGE" /
///                      "MESSAGE" /
///                      "VIDEO"
///                    ) DQUOTE
///                  ) / string
///                ) SP media-subtype`
///
/// Simplified...
///
/// `media-basic = string SP media-subtype`
///
/// TODO: Why the special case?
///
/// Defined in [MIME-IMT]
pub fn media_basic(input: &[u8]) -> IResult<&[u8], (IString, IString)> {
    let mut parser = tuple((string, SP, media_subtype));

    let (remaining, (type_, _, subtype)) = parser(input)?;

    Ok((remaining, (type_, subtype)))
}

#[inline]
/// `media-subtype = string`
///
/// Defined in [MIME-IMT]
pub fn media_subtype(input: &[u8]) -> IResult<&[u8], IString> {
    string(input)
}

#[inline]
/// `media-message = DQUOTE "MESSAGE" DQUOTE SP
///                  DQUOTE "RFC822" DQUOTE`
///
/// Simplified:
///
/// `media-message = "\"MESSAGE\" \"RFC822\""`
///
/// Defined in [MIME-IMT]
///
/// "message" "rfc822" basic specific-for-message-rfc822 extension
pub fn media_message(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag_no_case(b"\"MESSAGE\" \"RFC822\"")(input)
}

/// `media-text = DQUOTE "TEXT" DQUOTE SP media-subtype`
///
/// Defined in [MIME-IMT]
///
/// "text" "?????" basic specific-for-text extension
pub fn media_text(input: &[u8]) -> IResult<&[u8], IString> {
    let mut parser = preceded(tag_no_case(b"\"TEXT\" "), media_subtype);

    let (remaining, media_subtype) = parser(input)?;

    Ok((remaining, media_subtype))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_basic() {
        media_basic(b"\"application\" \"xxx\"").unwrap();
        media_basic(b"\"unknown\" \"test\"").unwrap();
        media_basic(b"\"x\" \"xxx\"").unwrap();
    }

    #[test]
    fn test_media_message() {
        media_message(b"\"message\" \"rfc822\"").unwrap();
    }

    #[test]
    fn test_media_text() {
        media_text(b"\"text\" \"html\"").unwrap();
    }

    #[test]
    fn test_body_ext_1part() {
        for test in [
            b"nil|xxx".as_ref(),
            b"\"md5\"|xxx".as_ref(),
            b"\"md5\" nil|xxx".as_ref(),
            b"\"md5\" (\"dsp\" nil)|xxx".as_ref(),
            b"\"md5\" (\"dsp\" (\"key\" \"value\")) nil|xxx".as_ref(),
            b"\"md5\" (\"dsp\" (\"key\" \"value\")) \"swedish\"|xxx".as_ref(),
            b"\"md5\" (\"dsp\" (\"key\" \"value\")) (\"german\" \"russian\")|xxx".as_ref(),
            b"\"md5\" (\"dsp\" (\"key\" \"value\")) (\"german\" \"russian\") nil|xxx".as_ref(),
            b"\"md5\" (\"dsp\" (\"key\" \"value\")) (\"german\" \"russian\") \"loc\"|xxx".as_ref(),
            b"\"md5\" (\"dsp\" (\"key\" \"value\")) (\"german\" \"russian\") \"loc\" (1 \"2\" (nil 4))|xxx".as_ref(),
        ]
        .iter()
        {
            let (rem, out) = body_ext_1part(test).unwrap();
            println!("{:?}", out);
            assert_eq!(rem, b"|xxx");
        }
    }

    #[test]
    fn test_body_rec() {
        let _ = body(8)(str::repeat("(", 1_000_000).as_bytes());
    }

    #[test]
    fn test_body_ext_mpart() {
        for test in [
            b"nil|xxx".as_ref(),
            b"(\"key\" \"value\")|xxx".as_ref(),
            b"(\"key\" \"value\") nil|xxx".as_ref(),
            b"(\"key\" \"value\") (\"dsp\" nil)|xxx".as_ref(),
            b"(\"key\" \"value\") (\"dsp\" (\"key\" \"value\")) nil|xxx".as_ref(),
            b"(\"key\" \"value\") (\"dsp\" (\"key\" \"value\")) \"swedish\"|xxx".as_ref(),
            b"(\"key\" \"value\") (\"dsp\" (\"key\" \"value\")) (\"german\" \"russian\")|xxx".as_ref(),
            b"(\"key\" \"value\") (\"dsp\" (\"key\" \"value\")) (\"german\" \"russian\") nil|xxx".as_ref(),
            b"(\"key\" \"value\") (\"dsp\" (\"key\" \"value\")) (\"german\" \"russian\") \"loc\"|xxx".as_ref(),
            b"(\"key\" \"value\") (\"dsp\" (\"key\" \"value\")) (\"german\" \"russian\") \"loc\" (1 \"2\" (nil 4))|xxx".as_ref(),
        ]
            .iter()
        {
            let (rem, out) = body_ext_mpart(test).unwrap();
            println!("{:?}", out);
            assert_eq!(rem, b"|xxx");
        }
    }
}
