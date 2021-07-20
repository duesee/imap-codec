use abnf_core::streaming::SP;
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    combinator::{map, value},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, preceded},
    IResult,
};

use crate::{
    parse::core::atom,
    types::{
        core::atm,
        flag::{Flag, FlagNameAttribute},
    },
};

/// flag = "\Answered" / "\Flagged" / "\Deleted" / "\Seen" / "\Draft" /
///        flag-keyword /
///        flag-extension
///
/// Note: Does not include "\Recent"
pub(crate) fn flag(input: &[u8]) -> IResult<&[u8], Flag> {
    alt((
        value(Flag::Answered, tag_no_case(b"\\Answered")),
        value(Flag::Flagged, tag_no_case(b"\\Flagged")),
        value(Flag::Deleted, tag_no_case(b"\\Deleted")),
        value(Flag::Seen, tag_no_case(b"\\Seen")),
        value(Flag::Draft, tag_no_case(b"\\Draft")),
        flag_keyword,
        map(flag_extension, |a| Flag::Extension(a.to_owned())),
    ))(input)
}

/// flag-fetch = flag / "\Recent"
pub(crate) fn flag_fetch(input: &[u8]) -> IResult<&[u8], Flag> {
    alt((flag, value(Flag::Recent, tag_no_case(b"\\Recent"))))(input)
}

/// flag-perm = flag / "\*"
pub(crate) fn flag_perm(input: &[u8]) -> IResult<&[u8], Flag> {
    alt((flag, value(Flag::Permanent, tag(b"\\*"))))(input)
}

#[inline]
/// flag-keyword = atom
fn flag_keyword(input: &[u8]) -> IResult<&[u8], Flag> {
    map(atom, |a| Flag::Keyword(a.to_owned()))(input)
}

/// flag-list = "(" [flag *(SP flag)] ")"
pub(crate) fn flag_list(input: &[u8]) -> IResult<&[u8], Vec<Flag>> {
    delimited(tag(b"("), separated_list0(SP, flag), tag(b")"))(input)
}

/// mbx-list-flags = *(mbx-list-oflag SP) mbx-list-sflag *(SP mbx-list-oflag) /
///                                       mbx-list-oflag *(SP mbx-list-oflag)
///
/// Note: ABNF enforces that sflag is not used more than once.
///       We parse any flag and check for multiple occurrences of sflag later.
pub(crate) fn mbx_list_flags(input: &[u8]) -> IResult<&[u8], Vec<FlagNameAttribute>> {
    let (remaining, flags) = separated_list1(SP, alt((mbx_list_sflag, mbx_list_oflag)))(input)?;

    let sflag_count = flags
        .iter()
        .filter(|&flag| FlagNameAttribute::is_selectability(flag))
        .count();

    if sflag_count > 1 {
        return Err(nom::Err::Error(nom::error::make_error(
            input,
            nom::error::ErrorKind::Verify, // TODO(verify): use `Failure` or `Error`?
        )));
    }

    Ok((remaining, flags))
}

/// Other flags; multiple possible per LIST response
///
/// mbx-list-oflag = "\Noinferiors" / flag-extension
fn mbx_list_oflag(input: &[u8]) -> IResult<&[u8], FlagNameAttribute> {
    alt((
        value(
            FlagNameAttribute::Noinferiors,
            tag_no_case(b"\\Noinferiors"),
        ),
        map(flag_extension, |a| {
            FlagNameAttribute::Extension(a.to_owned())
        }),
    ))(input)
}

/// Selectability flags; only one per LIST response
///
/// mbx-list-sflag = "\Noselect" / "\Marked" / "\Unmarked"
fn mbx_list_sflag(input: &[u8]) -> IResult<&[u8], FlagNameAttribute> {
    alt((
        value(FlagNameAttribute::Noselect, tag_no_case(b"\\Noselect")),
        value(FlagNameAttribute::Marked, tag_no_case(b"\\Marked")),
        value(FlagNameAttribute::Unmarked, tag_no_case(b"\\Unmarked")),
    ))(input)
}

/// Future expansion.
///
/// Client implementations MUST accept flag-extension flags.
/// Server implementations MUST NOT generate flag-extension flags
/// except as defined by future standard or standards-track revisions of this specification.
///
/// flag-extension = "\" atom
fn flag_extension(input: &[u8]) -> IResult<&[u8], atm> {
    preceded(tag(b"\\"), atom)(input)
}
