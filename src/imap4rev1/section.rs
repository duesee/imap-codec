use std::num::NonZeroU32;

use abnf_core::streaming::SP;
use imap_types::{
    core::{AString, NonEmptyVec},
    message::{Part, PartSpecifier, Section},
};
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    combinator::{map, opt, value},
    multi::separated_list1,
    sequence::{delimited, tuple},
    IResult,
};

use crate::imap4rev1::core::{astring, nz_number};

/// `section = "[" [section-spec] "]"`
pub fn section(input: &[u8]) -> IResult<&[u8], Option<Section>> {
    delimited(tag(b"["), opt(section_spec), tag(b"]"))(input)
}

/// `section-spec = section-msgtext / (section-part ["." section-text])`
pub fn section_spec(input: &[u8]) -> IResult<&[u8], Section> {
    alt((
        map(section_msgtext, |part_specifier| match part_specifier {
            PartSpecifier::PartNumber(_) => unreachable!(),
            PartSpecifier::Header => Section::Header(None),
            PartSpecifier::HeaderFields(fields) => Section::HeaderFields(None, fields),
            PartSpecifier::HeaderFieldsNot(fields) => Section::HeaderFieldsNot(None, fields),
            PartSpecifier::Text => Section::Text(None),
            PartSpecifier::Mime => unreachable!(),
        }),
        map(
            tuple((section_part, opt(tuple((tag(b"."), section_text))))),
            |(part_number, maybe_part_specifier)| {
                if let Some((_, part_specifier)) = maybe_part_specifier {
                    match part_specifier {
                        PartSpecifier::PartNumber(_) => unreachable!(),
                        PartSpecifier::Header => Section::Header(Some(Part(part_number))),
                        PartSpecifier::HeaderFields(fields) => {
                            Section::HeaderFields(Some(Part(part_number)), fields)
                        }
                        PartSpecifier::HeaderFieldsNot(fields) => {
                            Section::HeaderFieldsNot(Some(Part(part_number)), fields)
                        }
                        PartSpecifier::Text => Section::Text(Some(Part(part_number))),
                        PartSpecifier::Mime => Section::Mime(Part(part_number)),
                    }
                } else {
                    Section::Part(Part(part_number))
                }
            },
        ),
    ))(input)
}

/// `section-msgtext = "HEADER" / "HEADER.FIELDS" [".NOT"] SP header-list / "TEXT"`
///
/// Top-level or MESSAGE/RFC822 part
pub fn section_msgtext(input: &[u8]) -> IResult<&[u8], PartSpecifier> {
    alt((
        map(
            tuple((tag_no_case(b"HEADER.FIELDS.NOT"), SP, header_list)),
            |(_, _, header_list)| PartSpecifier::HeaderFieldsNot(header_list),
        ),
        map(
            tuple((tag_no_case(b"HEADER.FIELDS"), SP, header_list)),
            |(_, _, header_list)| PartSpecifier::HeaderFields(header_list),
        ),
        value(PartSpecifier::Header, tag_no_case(b"HEADER")),
        value(PartSpecifier::Text, tag_no_case(b"TEXT")),
    ))(input)
}

#[inline]
/// `section-part = nz-number *("." nz-number)`
///
/// Body part nesting
pub fn section_part(input: &[u8]) -> IResult<&[u8], NonEmptyVec<NonZeroU32>> {
    map(
        separated_list1(tag(b"."), nz_number),
        NonEmptyVec::unchecked,
    )(input)
}

/// `section-text = section-msgtext / "MIME"`
///
/// Text other than actual body part (headers, etc.)
pub fn section_text(input: &[u8]) -> IResult<&[u8], PartSpecifier> {
    alt((
        section_msgtext,
        value(PartSpecifier::Mime, tag_no_case(b"MIME")),
    ))(input)
}

/// `header-list = "(" header-fld-name *(SP header-fld-name) ")"`
pub fn header_list(input: &[u8]) -> IResult<&[u8], NonEmptyVec<AString>> {
    map(
        delimited(tag(b"("), separated_list1(SP, header_fld_name), tag(b")")),
        NonEmptyVec::unchecked,
    )(input)
}

#[inline]
/// `header-fld-name = astring`
pub fn header_fld_name(input: &[u8]) -> IResult<&[u8], AString> {
    astring(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::known_answer_test_encode;

    #[test]
    fn test_encode_section() {
        let tests = [
            (
                Section::Part(Part(NonEmptyVec::from(NonZeroU32::try_from(1).unwrap()))),
                b"1".as_ref(),
            ),
            (Section::Header(None), b"HEADER"),
            (
                Section::Header(Some(Part(NonEmptyVec::from(
                    NonZeroU32::try_from(1).unwrap(),
                )))),
                b"1.HEADER",
            ),
            (
                Section::HeaderFields(None, NonEmptyVec::from(AString::try_from("").unwrap())),
                b"HEADER.FIELDS (\"\")",
            ),
            (
                Section::HeaderFields(
                    Some(Part(NonEmptyVec::from(NonZeroU32::try_from(1).unwrap()))),
                    NonEmptyVec::from(AString::try_from("").unwrap()),
                ),
                b"1.HEADER.FIELDS (\"\")",
            ),
            (
                Section::HeaderFieldsNot(None, NonEmptyVec::from(AString::try_from("").unwrap())),
                b"HEADER.FIELDS.NOT (\"\")",
            ),
            (
                Section::HeaderFieldsNot(
                    Some(Part(NonEmptyVec::from(NonZeroU32::try_from(1).unwrap()))),
                    NonEmptyVec::from(AString::try_from("").unwrap()),
                ),
                b"1.HEADER.FIELDS.NOT (\"\")",
            ),
            (Section::Text(None), b"TEXT"),
            (
                Section::Text(Some(Part(NonEmptyVec::from(
                    NonZeroU32::try_from(1).unwrap(),
                )))),
                b"1.TEXT",
            ),
            (
                Section::Mime(Part(NonEmptyVec::from(NonZeroU32::try_from(1).unwrap()))),
                b"1.MIME",
            ),
        ];

        for test in tests {
            known_answer_test_encode(test)
        }
    }
}
