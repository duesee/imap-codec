use crate::{
    parse::{core::astring, sp},
    types::core::AString,
};
use nom::{bytes::streaming::tag, multi::separated_nonempty_list, sequence::tuple, IResult};

/// header-list = "(" header-fld-name *(SP header-fld-name) ")"
pub fn header_list(input: &[u8]) -> IResult<&[u8], Vec<AString>> {
    let parser = tuple((
        tag(b"("),
        separated_nonempty_list(sp, header_fld_name),
        tag(b")"),
    ));

    let (remaining, (_, header_list, _)) = parser(input)?;

    Ok((remaining, header_list))
}

/// header-fld-name = astring
pub fn header_fld_name(input: &[u8]) -> IResult<&[u8], AString> {
    astring(input)
}
