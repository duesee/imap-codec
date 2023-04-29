use abnf_core::streaming::SP;
use imap_types::message::{Flag, FlagFetch, FlagNameAttribute, FlagPerm};
use nom::{
    branch::alt,
    bytes::streaming::tag,
    character::streaming::char,
    combinator::{map, recognize, value},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, preceded, tuple},
    IResult,
};

use crate::imap4rev1::core::atom;

/// ```abnf
/// flag = "\Answered" /
///        "\Flagged" /
///        "\Deleted" /
///        "\Seen" /
///        "\Draft" /
///        flag-keyword /
///        flag-extension
/// ```
///
/// Note: Does not include "\Recent"
pub fn flag(input: &[u8]) -> IResult<&[u8], Flag> {
    alt((
        map(preceded(char('\\'), atom), Flag::system_or_extension),
        map(atom, Flag::Keyword),
    ))(input)
}

// Note(duesee): This was inlined into [`flag`].
// #[inline]
// /// `flag-keyword = atom`
// pub fn flag_keyword(input: &[u8]) -> IResult<&[u8], Flag> {
//     map(atom, Flag::Keyword)(input)
// }

// Note: This was inlined into `mbx_list_flags`.
// /// ```abnf
// /// flag-extension = "\" atom
// /// ```
// ///
// /// Future expansion.
// ///
// /// Client implementations MUST accept flag-extension flags.
// /// Server implementations MUST NOT generate flag-extension flags
// /// except as defined by future standard or standards-track revisions of this specification.
// pub fn flag_extension(input: &[u8]) -> IResult<&[u8], Atom> {
//     preceded(tag(b"\\"), atom)(input)
// }

/// `flag-list = "(" [flag *(SP flag)] ")"`
pub fn flag_list(input: &[u8]) -> IResult<&[u8], Vec<Flag>> {
    delimited(tag(b"("), separated_list0(SP, flag), tag(b")"))(input)
}

/// `flag-fetch = flag / "\Recent"`
pub fn flag_fetch(input: &[u8]) -> IResult<&[u8], FlagFetch> {
    if let Ok((rem, peek)) = recognize(tuple((char('\\'), atom)))(input) {
        if peek.to_ascii_lowercase() == b"\\recent" {
            return Ok((rem, FlagFetch::Recent));
        }
    }

    map(flag, FlagFetch::Flag)(input)
}

/// `flag-perm = flag / "\*"`
pub fn flag_perm(input: &[u8]) -> IResult<&[u8], FlagPerm> {
    alt((
        value(FlagPerm::AllowNewKeywords, tag("\\*")),
        map(flag, FlagPerm::Flag),
    ))(input)
}

/// ```abnf
/// mbx-list-flags = *(mbx-list-oflag SP) mbx-list-sflag *(SP mbx-list-oflag) /
///                                        mbx-list-oflag *(SP mbx-list-oflag)
/// ```
///
/// TODO(#155): ABNF enforces that sflag is not used more than once.
///             We could parse any flag and check for multiple occurrences of sflag later.
pub fn mbx_list_flags(input: &[u8]) -> IResult<&[u8], Vec<FlagNameAttribute>> {
    let (remaining, flags) =
        separated_list1(SP, map(preceded(char('\\'), atom), FlagNameAttribute::from))(input)?;

    // TODO(#155): Do we really want to enforce this?
    // let sflag_count = flags
    //     .iter()
    //     .filter(|&flag| FlagNameAttribute::is_selectability(flag))
    //     .count();
    //
    // if sflag_count > 1 {
    //     return Err(nom::Err::Failure(nom::error::make_error(
    //         input,
    //         nom::error::ErrorKind::Verify,
    //     )));
    // }

    Ok((remaining, flags))
}

// Note: This was inlined into `mbx_list_flags`.
// /// ```abnf
// /// mbx-list-oflag = "\Noinferiors" / flag-extension
// /// ```
// ///
// /// Other flags; multiple possible per LIST response
// pub fn mbx_list_oflag(input: &[u8]) -> IResult<&[u8], FlagNameAttribute> {
//     alt((
//         value(
//             FlagNameAttribute::Noinferiors,
//             tag_no_case(b"\\Noinferiors"),
//         ),
//         map(flag_extension, FlagNameAttribute::Extension),
//     ))(input)
// }

// Note: This was inlined into `mbx_list_flags`.
// /// ```abnf
// /// mbx-list-sflag = "\Noselect" / "\Marked" / "\Unmarked"
// /// ```
// ///
// /// Selectability flags; only one per LIST response
// pub fn mbx_list_sflag(input: &[u8]) -> IResult<&[u8], FlagNameAttribute> {
//     alt((
//         value(FlagNameAttribute::Noselect, tag_no_case(b"\\Noselect")),
//         value(FlagNameAttribute::Marked, tag_no_case(b"\\Marked")),
//         value(FlagNameAttribute::Unmarked, tag_no_case(b"\\Unmarked")),
//     ))(input)
// }

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use imap_types::{core::Atom, message::FlagExtension};

    use super::*;

    #[test]
    fn test_flag_fetch() {
        let tests = [(
            "iS)",
            FlagFetch::Flag(Flag::Keyword(Atom::try_from("iS").unwrap())),
        )];

        for (test, expected) in tests {
            let (rem, got) = flag_fetch(test.as_bytes()).unwrap();
            assert_eq!(rem.len(), 1);
            assert_eq!(expected, got);
        }
    }

    #[test]
    fn test_flag_perm() {
        let tests = [
            ("\\Deleted)", FlagPerm::Flag(Flag::Deleted)),
            (
                "\\Deletedx)",
                FlagPerm::Flag(Flag::Extension(
                    FlagExtension::try_from(Atom::try_from("Deletedx").unwrap()).unwrap(),
                )),
            ),
            ("\\Seen ", FlagPerm::Flag(Flag::Seen)),
            ("\\*)", FlagPerm::AllowNewKeywords),
        ];

        for (test, expected) in tests {
            let (rem, got) = flag_perm(test.as_bytes()).unwrap();
            assert_eq!(rem.len(), 1);
            assert_eq!(expected, got);
        }
    }

    #[test]
    fn test_mbx_list_flags() {
        let tests = [
            (
                "\\Markedm)",
                vec![FlagNameAttribute::Extension(
                    Atom::try_from("Markedm").unwrap(),
                )],
            ),
            ("\\Marked)", vec![FlagNameAttribute::Marked]),
        ];

        for (test, expected) in tests {
            let (rem, got) = mbx_list_flags(test.as_bytes()).unwrap();
            assert_eq!(expected, got);
            assert_eq!(rem.len(), 1);
        }
    }
}
