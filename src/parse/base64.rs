use crate::parse::{is_alpha, is_digit};
use nom::{
    branch::alt,
    bytes::streaming::{tag as nom_tag, take_while},
    combinator::{map_res, opt, recognize},
    sequence::tuple,
    IResult,
};
use std::str::from_utf8;

/// base64 = *(4base64-char) [base64-terminal]
pub fn base64(input: &[u8]) -> IResult<&[u8], &str> {
    let parser = map_res(
        recognize(tuple((
            take_while(is_base64_char),
            opt(alt((nom_tag("=="), nom_tag("=")))),
        ))),
        from_utf8,
    );

    let (remaining, base64) = parser(input)?;

    Ok((remaining, base64))
}

/// base64-char = ALPHA / DIGIT / "+" / "/" ; Case-sensitive
fn is_base64_char(i: u8) -> bool {
    is_alpha(i) || is_digit(i) || i == b'+' || i == b'/'
}

// base64-terminal = (2base64-char "==") / (3base64-char "=")
