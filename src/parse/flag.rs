use crate::{
    parse::{core::atom, sp},
    types::{
        core::Atom,
        message_attributes::{Flag, SystemFlag},
    },
};
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    combinator::{map, value},
    multi::separated_list,
    sequence::tuple,
    IResult,
};

/// flag = "\Answered" / "\Flagged" / "\Deleted" / "\Seen" / "\Draft" / flag-keyword / flag-extension
///          ; Does not include "\Recent"
pub fn flag(input: &[u8]) -> IResult<&[u8], Flag> {
    use Flag::*;
    use SystemFlag::*;

    let parser = alt((
        value(System(Answered), tag_no_case(b"\\Answered")),
        value(System(Flagged), tag_no_case(b"\\Flagged")),
        value(System(Deleted), tag_no_case(b"\\Deleted")),
        value(System(Seen), tag_no_case(b"\\Seen")),
        value(System(Draft), tag_no_case(b"\\Draft")),
        map(flag_keyword, Keyword),
        map(flag_extension, Extension),
    ));

    let (remaining, parsed_flag) = parser(input)?;

    Ok((remaining, parsed_flag))
}

/// flag-keyword = atom
pub fn flag_keyword(input: &[u8]) -> IResult<&[u8], Atom> {
    atom(input)
}

/// flag-extension = "\" atom
///                   ; Future expansion.  Client implementations
///                   ; MUST accept flag-extension flags.  Server
///                   ; implementations MUST NOT generate
///                   ; flag-extension flags except as defined by
///                   ; future standard or standards-track
///                   ; revisions of this specification.
pub fn flag_extension(input: &[u8]) -> IResult<&[u8], Atom> {
    let parser = tuple((tag_no_case(b"\\"), atom));

    let (remaining, (_, atom)) = parser(input)?;

    Ok((remaining, atom))
}

/// flag-fetch = flag / "\Recent"
pub fn flag_fetch(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = alt((
        map(flag, |_| unimplemented!()),
        map(tag_no_case(b"\\Recent"), |_| unimplemented!()),
    ));

    let (_remaining, _parsed_flag_fetch) = parser(input)?;

    unimplemented!();
}

/// flag-list = "(" [flag *(SP flag)] ")"
pub fn flag_list(input: &[u8]) -> IResult<&[u8], Vec<Flag>> {
    let parser = tuple((tag(b"("), separated_list(sp, flag), tag(b")")));

    let (remaining, (_, flag_list, _)) = parser(input)?;

    Ok((remaining, flag_list))
}

/// flag-perm = flag / "\*"
pub fn flag_perm(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = alt((
        map(flag, |_| unimplemented!()),
        map(tag_no_case(b"\\*"), |_| unimplemented!()),
    ));

    let (_remaining, _parsed_flag_perm) = parser(input)?;

    unimplemented!();
}
