use crate::parse::{is_alpha, is_digit};
use nom::{
    branch::alt,
    bytes::streaming::{tag as nom_tag, take_while},
    combinator::opt,
    sequence::tuple,
    IResult,
};

/// base64 = *(4base64-char) [base64-terminal]
pub fn base64(input: &[u8]) -> IResult<&[u8], String> {
    let parser = tuple((
        take_while(is_base64_char),
        opt(alt((nom_tag("=="), nom_tag("=")))),
    ));

    let (remaining, (data, pad)) = parser(input)?;

    let mut output = Vec::new();
    output.extend(data);
    if let Some(pad) = pad {
        output.extend(pad);
    }

    Ok((remaining, String::from_utf8(output).unwrap()))
}

fn is_base64_char(i: u8) -> bool {
    is_alpha(i) || is_digit(i) || i == b'+' || i == b'/'
}

// base64-char = ALPHA / DIGIT / "+" / "/" ; Case-sensitive

// base64-terminal = (2base64-char "==") / (3base64-char "=")
