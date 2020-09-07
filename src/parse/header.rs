use crate::{parse::core::astring, types::core::astr};
use abnf_core::streaming::SP;
use nom::{bytes::streaming::tag, multi::separated_nonempty_list, sequence::delimited, IResult};

/// header-list = "(" header-fld-name *(SP header-fld-name) ")"
pub(crate) fn header_list(input: &[u8]) -> IResult<&[u8], Vec<astr>> {
    delimited(
        tag(b"("),
        separated_nonempty_list(SP, header_fld_name),
        tag(b")"),
    )(input)
}

#[inline]
/// header-fld-name = astring
pub(crate) fn header_fld_name(input: &[u8]) -> IResult<&[u8], astr> {
    astring(input)
}
