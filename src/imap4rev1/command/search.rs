use abnf_core::streaming::SP;
use nom::{
    branch::alt,
    bytes::{complete::tag, streaming::tag_no_case},
    combinator::{map, map_opt, opt, value},
    multi::{many1, separated_list1},
    sequence::{delimited, preceded, tuple},
    IResult,
};

use crate::{
    command::{search::SearchKey, CommandBody},
    core::NonEmptyVec,
    imap4rev1::{
        core::{astring, atom, charset, number},
        datetime::date,
        section::header_fld_name,
        sequence::sequence_set,
    },
};

/// `search = "SEARCH" [SP "CHARSET" SP charset] 1*(SP search-key)`
///
/// Note: CHARSET argument MUST be registered with IANA
///
/// errata id: 261
pub fn search(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((
        tag_no_case(b"SEARCH"),
        opt(map(
            tuple((SP, tag_no_case(b"CHARSET"), SP, charset)),
            |(_, _, _, charset)| charset,
        )),
        many1(preceded(SP, search_key(8))),
    ));

    let (remaining, (_, charset, mut criteria)) = parser(input)?;

    let criteria = match criteria.len() {
        0 => unreachable!(),
        1 => criteria.pop().unwrap(),
        _ => SearchKey::And(NonEmptyVec::new_unchecked(criteria)),
    };

    Ok((
        remaining,
        CommandBody::Search {
            charset,
            criteria,
            uid: false,
        },
    ))
}

/// `search-key = "ALL" /
///               "ANSWERED" /
///               "BCC" SP astring /
///               "BEFORE" SP date /
///               "BODY" SP astring /
///               "CC" SP astring /
///               "DELETED" /
///               "FLAGGED" /
///               "FROM" SP astring /
///               "KEYWORD" SP flag-keyword /
///               "NEW" /
///               "OLD" /
///               "ON" SP date /
///               "RECENT" /
///               "SEEN" /
///               "SINCE" SP date /
///               "SUBJECT" SP astring /
///               "TEXT" SP astring /
///               "TO" SP astring /
///               "UNANSWERED" /
///               "UNDELETED" /
///               "UNFLAGGED" /
///               "UNKEYWORD" SP flag-keyword /
///               "UNSEEN" /
///                 ; Above this line were in [IMAP2]
///               "DRAFT" /
///               "HEADER" SP header-fld-name SP astring /
///               "LARGER" SP number /
///               "NOT" SP search-key /
///               "OR" SP search-key SP search-key /
///               "SENTBEFORE" SP date /
///               "SENTON" SP date /
///               "SENTSINCE" SP date /
///               "SMALLER" SP number /
///               "UID" SP sequence-set /
///               "UNDRAFT" /
///               sequence-set /
///               "(" search-key *(SP search-key) ")"`
///
/// This parser is recursively defined. Thus, in order to not overflow the stack,
/// it is needed to limit how may recursions are allowed. (8 should suffice).
pub fn search_key(remaining_recursions: usize) -> impl Fn(&[u8]) -> IResult<&[u8], SearchKey> {
    move |input: &[u8]| search_key_limited(input, remaining_recursions)
}

fn search_key_limited<'a>(
    input: &'a [u8],
    remaining_recursion: usize,
) -> IResult<&'a [u8], SearchKey> {
    if remaining_recursion == 0 {
        return Err(nom::Err::Failure(nom::error::make_error(
            input,
            nom::error::ErrorKind::TooLarge,
        )));
    }

    let search_key =
        move |input: &'a [u8]| search_key_limited(input, remaining_recursion.saturating_sub(1));

    alt((
        alt((
            value(SearchKey::All, tag_no_case(b"ALL")),
            value(SearchKey::Answered, tag_no_case(b"ANSWERED")),
            map(tuple((tag_no_case(b"BCC"), SP, astring)), |(_, _, val)| {
                SearchKey::Bcc(val)
            }),
            map(
                tuple((tag_no_case(b"BEFORE"), SP, map_opt(date, |date| date))),
                |(_, _, date)| SearchKey::Before(date),
            ),
            map(tuple((tag_no_case(b"BODY"), SP, astring)), |(_, _, val)| {
                SearchKey::Body(val)
            }),
            map(tuple((tag_no_case(b"CC"), SP, astring)), |(_, _, val)| {
                SearchKey::Cc(val)
            }),
            value(SearchKey::Deleted, tag_no_case(b"DELETED")),
            value(SearchKey::Flagged, tag_no_case(b"FLAGGED")),
            map(tuple((tag_no_case(b"FROM"), SP, astring)), |(_, _, val)| {
                SearchKey::From(val)
            }),
            map(
                // Note: `flag_keyword` parser returns `Flag`. Because Rust does not have first-class enum variants
                // it is not possible to fix SearchKey(Flag::Keyword), but only SearchKey(Flag).
                // Thus `SearchKey::Keyword(Atom)` is used instead. This is, why we use also `atom` parser here and not `flag_keyword` parser.
                tuple((tag_no_case(b"KEYWORD"), SP, atom)),
                |(_, _, val)| SearchKey::Keyword(val),
            ),
            value(SearchKey::New, tag_no_case(b"NEW")),
            value(SearchKey::Old, tag_no_case(b"OLD")),
            map(
                tuple((tag_no_case(b"ON"), SP, map_opt(date, |date| date))),
                |(_, _, date)| SearchKey::On(date),
            ),
            value(SearchKey::Recent, tag_no_case(b"RECENT")),
            value(SearchKey::Seen, tag_no_case(b"SEEN")),
            map(
                tuple((tag_no_case(b"SINCE"), SP, map_opt(date, |date| date))),
                |(_, _, date)| SearchKey::Since(date),
            ),
            map(
                tuple((tag_no_case(b"SUBJECT"), SP, astring)),
                |(_, _, val)| SearchKey::Subject(val),
            ),
            map(tuple((tag_no_case(b"TEXT"), SP, astring)), |(_, _, val)| {
                SearchKey::Text(val)
            }),
            map(tuple((tag_no_case(b"TO"), SP, astring)), |(_, _, val)| {
                SearchKey::To(val)
            }),
        )),
        alt((
            value(SearchKey::Unanswered, tag_no_case(b"UNANSWERED")),
            value(SearchKey::Undeleted, tag_no_case(b"UNDELETED")),
            value(SearchKey::Unflagged, tag_no_case(b"UNFLAGGED")),
            map(
                // Note: `flag_keyword` parser returns `Flag`. Because Rust does not have first-class enum variants
                // it is not possible to fix SearchKey(Flag::Keyword), but only SearchKey(Flag).
                // Thus `SearchKey::Keyword(Atom)` is used instead. This is, why we use also `atom` parser here and not `flag_keyword` parser.
                tuple((tag_no_case(b"UNKEYWORD"), SP, atom)),
                |(_, _, val)| SearchKey::Unkeyword(val),
            ),
            value(SearchKey::Unseen, tag_no_case(b"UNSEEN")),
            value(SearchKey::Draft, tag_no_case(b"DRAFT")),
            map(
                tuple((tag_no_case(b"HEADER"), SP, header_fld_name, SP, astring)),
                |(_, _, key, _, val)| SearchKey::Header(key, val),
            ),
            map(
                tuple((tag_no_case(b"LARGER"), SP, number)),
                |(_, _, val)| SearchKey::Larger(val),
            ),
            map(
                tuple((tag_no_case(b"NOT"), SP, search_key)),
                |(_, _, val)| SearchKey::Not(Box::new(val)),
            ),
            map(
                tuple((tag_no_case(b"OR"), SP, search_key, SP, search_key)),
                |(_, _, alt1, _, alt2)| SearchKey::Or(Box::new(alt1), Box::new(alt2)),
            ),
            map(
                tuple((tag_no_case(b"SENTBEFORE"), SP, map_opt(date, |date| date))),
                |(_, _, date)| SearchKey::SentBefore(date),
            ),
            map(
                tuple((tag_no_case(b"SENTON"), SP, map_opt(date, |date| date))),
                |(_, _, date)| SearchKey::SentOn(date),
            ),
            map(
                tuple((tag_no_case(b"SENTSINCE"), SP, map_opt(date, |date| date))),
                |(_, _, date)| SearchKey::SentSince(date),
            ),
            map(
                tuple((tag_no_case(b"SMALLER"), SP, number)),
                |(_, _, val)| SearchKey::Smaller(val),
            ),
            map(
                tuple((tag_no_case(b"UID"), SP, sequence_set)),
                |(_, _, val)| SearchKey::Uid(val),
            ),
            value(SearchKey::Undraft, tag_no_case(b"UNDRAFT")),
            map(sequence_set, SearchKey::SequenceSet),
            map(
                delimited(tag(b"("), separated_list1(SP, search_key), tag(b")")),
                |mut val| match val.len() {
                    0 => unreachable!(),
                    1 => val.pop().unwrap(),
                    _ => SearchKey::And(NonEmptyVec::new_unchecked(val)),
                },
            ),
        )),
    ))(input)
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use imap_types::command::SequenceSet as SequenceSetData;

    use super::*;

    #[test]
    fn test_parse_search() {
        use crate::command::{search::SearchKey::*, SeqOrUid::Value, Sequence::*};

        let (_rem, val) = search(b"search (uid 5)???").unwrap();
        assert_eq!(
            val,
            CommandBody::Search {
                charset: None,
                criteria: Uid(SequenceSetData(
                    vec![Single(Value(5.try_into().unwrap()))]
                        .try_into()
                        .unwrap()
                )),
                uid: false,
            }
        );

        let (_rem, val) = search(b"search (uid 5 or uid 5 (uid 1 uid 2) not (uid 5))???").unwrap();
        let expected = CommandBody::Search {
            charset: None,
            criteria: And(vec![
                Uid(SequenceSetData(
                    vec![Single(Value(5.try_into().unwrap()))]
                        .try_into()
                        .unwrap(),
                )),
                Or(
                    Box::new(Uid(SequenceSetData(
                        vec![Single(Value(5.try_into().unwrap()))]
                            .try_into()
                            .unwrap(),
                    ))),
                    Box::new(And(vec![
                        Uid(SequenceSetData(
                            vec![Single(Value(1.try_into().unwrap()))]
                                .try_into()
                                .unwrap(),
                        )),
                        Uid(SequenceSetData(
                            vec![Single(Value(2.try_into().unwrap()))]
                                .try_into()
                                .unwrap(),
                        )),
                    ]
                    .try_into()
                    .unwrap())),
                ),
                Not(Box::new(Uid(SequenceSetData(
                    vec![Single(Value(5.try_into().unwrap()))]
                        .try_into()
                        .unwrap(),
                )))),
            ]
            .try_into()
            .unwrap()),
            uid: false,
        };
        assert_eq!(val, expected);
    }

    #[test]
    fn test_parse_search_key() {
        assert!(search_key(1)(b"1:5|").is_ok());
        assert!(search_key(1)(b"(1:5)|").is_err());
        assert!(search_key(2)(b"(1:5)|").is_ok());
        assert!(search_key(2)(b"((1:5))|").is_err());
    }
}
