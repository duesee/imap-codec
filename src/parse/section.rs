use std::{convert::TryFrom, num::NonZeroU32};

use abnf_core::streaming::SP;
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    combinator::{map, opt, value},
    multi::separated_list1,
    sequence::{delimited, tuple},
    IResult,
};

use crate::{
    parse::core::{astring, nz_number},
    types::{
        core::{AString, AStringRef, NonEmptyVec},
        section::{Part, PartSpecifier, Section},
    },
};

/// section = "[" [section-spec] "]"
pub fn section(input: &[u8]) -> IResult<&[u8], Option<Section>> {
    delimited(tag(b"["), opt(section_spec), tag(b"]"))(input)
}

/// section-spec = section-msgtext / (section-part ["." section-text])
fn section_spec(input: &[u8]) -> IResult<&[u8], Section> {
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

/// Top-level or MESSAGE/RFC822 part
///
/// section-msgtext = "HEADER" / "HEADER.FIELDS" [".NOT"] SP header-list / "TEXT"
fn section_msgtext(input: &[u8]) -> IResult<&[u8], PartSpecifier> {
    alt((
        map(
            tuple((tag_no_case(b"HEADER.FIELDS.NOT"), SP, header_list)),
            |(_, _, header_list)| {
                PartSpecifier::HeaderFieldsNot(
                    NonEmptyVec::try_from(
                        header_list
                            .iter()
                            .map(|item| item.to_owned())
                            .collect::<Vec<AString>>(),
                    )
                    .unwrap(),
                )
            },
        ),
        map(
            tuple((tag_no_case(b"HEADER.FIELDS"), SP, header_list)),
            |(_, _, header_list)| {
                PartSpecifier::HeaderFields(
                    NonEmptyVec::try_from(
                        header_list
                            .iter()
                            .map(|item| item.to_owned())
                            .collect::<Vec<AString>>(),
                    )
                    .unwrap(),
                )
            },
        ),
        value(PartSpecifier::Header, tag_no_case(b"HEADER")),
        value(PartSpecifier::Text, tag_no_case(b"TEXT")),
    ))(input)
}

#[inline]
/// Body part nesting
///
/// section-part = nz-number *("." nz-number)
fn section_part(input: &[u8]) -> IResult<&[u8], NonEmptyVec<NonZeroU32>> {
    map(separated_list1(tag(b"."), nz_number), |vec| {
        NonEmptyVec::try_from(vec).unwrap()
    })(input)
}

/// Text other than actual body part (headers, etc.)
///
/// section-text = section-msgtext / "MIME"
fn section_text(input: &[u8]) -> IResult<&[u8], PartSpecifier> {
    alt((
        section_msgtext,
        value(PartSpecifier::Mime, tag_no_case(b"MIME")),
    ))(input)
}

/// header-list = "(" header-fld-name *(SP header-fld-name) ")"
fn header_list(input: &[u8]) -> IResult<&[u8], NonEmptyVec<AStringRef>> {
    map(
        delimited(tag(b"("), separated_list1(SP, header_fld_name), tag(b")")),
        |vec| NonEmptyVec::try_from(vec).unwrap(),
    )(input)
}

#[inline]
/// header-fld-name = astring
pub(crate) fn header_fld_name(input: &[u8]) -> IResult<&[u8], AStringRef> {
    astring(input)
}
