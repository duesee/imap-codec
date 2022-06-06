use std::borrow::Cow;

use abnf_core::streaming::{CRLF, SP};
use imap_types::{
    command::{Command, CommandBody, SearchKey},
    core::{AString, NonEmptyVec},
    fetch_attributes::{Macro, MacroOrFetchAttributes},
    flag::{Flag, StoreResponse, StoreType},
    AuthMechanism,
};
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    combinator::{map, map_opt, opt, value},
    multi::{many1, separated_list0, separated_list1},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

#[cfg(feature = "ext_idle")]
use crate::extensions::rfc2177::idle;
#[cfg(feature = "ext_compress")]
use crate::extensions::rfc4987::compress;
#[cfg(feature = "ext_enable")]
use crate::extensions::rfc5161::enable;
use crate::rfc3501::{
    auth_type,
    core::{astring, atom, base64, charset, literal, number, tag_imap},
    datetime::{date, date_time},
    fetch_attributes::fetch_att,
    flag::{flag, flag_list},
    mailbox::{list_mailbox, mailbox},
    section::header_fld_name,
    sequence::sequence_set,
    status_attributes::status_att,
};

/// `command = tag SP (
///                     command-any /
///                     command-auth /
///                     command-nonauth /
///                     command-select
///                   ) CRLF`
pub fn command(input: &[u8]) -> IResult<&[u8], Command> {
    let mut parser = tuple((
        tag_imap,
        SP,
        alt((command_any, command_auth, command_nonauth, command_select)),
        CRLF,
    ));

    let (remaining, (tag, _, command_body, _)) = parser(input)?;

    Ok((remaining, Command::new(tag, command_body)))
}

// # Command Any

/// `command-any = "CAPABILITY" / "LOGOUT" / "NOOP" / x-command`
///
/// Note: Valid in all states
pub fn command_any(input: &[u8]) -> IResult<&[u8], CommandBody> {
    alt((
        value(CommandBody::Capability, tag_no_case(b"CAPABILITY")),
        value(CommandBody::Logout, tag_no_case(b"LOGOUT")),
        value(CommandBody::Noop, tag_no_case(b"NOOP")),
        // x-command = "X" atom <experimental command arguments>
    ))(input)
}

// # Command Auth

/// `command-auth = append /
///                 create /
///                 delete /
///                 examine /
///                 list /
///                 lsub /
///                 rename /
///                 select /
///                 status /
///                 subscribe /
///                 unsubscribe /
///                 idle ; RFC 2177
///                 enable ; RFC 5161
///                 compress ; RFC 4978`
///
/// Note: Valid only in Authenticated or Selected state
pub fn command_auth(input: &[u8]) -> IResult<&[u8], CommandBody> {
    alt((
        append,
        create,
        delete,
        examine,
        list,
        lsub,
        rename,
        select,
        status,
        subscribe,
        unsubscribe,
        #[cfg(feature = "ext_idle")]
        idle,
        #[cfg(feature = "ext_enable")]
        // Note: The formal syntax defines ENABLE in command-any, but describes it to
        // be allowed in the authenticated state only. I will use the authenticated state.
        enable,
        #[cfg(feature = "ext_compress")]
        compress,
    ))(input)
}

/// `append = "APPEND" SP mailbox [SP flag-list] [SP date-time] SP literal`
pub fn append(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((
        tag_no_case(b"APPEND"),
        SP,
        mailbox,
        opt(preceded(SP, flag_list)),
        opt(preceded(SP, date_time)),
        SP,
        literal,
    ));

    let (remaining, (_, _, mailbox, flags, date, _, message)) = parser(input)?;

    Ok((
        remaining,
        CommandBody::Append {
            mailbox,
            flags: flags.unwrap_or_default(),
            date,
            message,
        },
    ))
}

/// `create = "CREATE" SP mailbox`
///
/// Note: Use of INBOX gives a NO error
pub fn create(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"CREATE"), SP, mailbox));

    let (remaining, (_, _, mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::Create { mailbox }))
}

/// `delete = "DELETE" SP mailbox`
///
/// Note: Use of INBOX gives a NO error
pub fn delete(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"DELETE"), SP, mailbox));

    let (remaining, (_, _, mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::Delete { mailbox }))
}

/// `examine = "EXAMINE" SP mailbox`
pub fn examine(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"EXAMINE"), SP, mailbox));

    let (remaining, (_, _, mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::Examine { mailbox }))
}

/// `list = "LIST" SP mailbox SP list-mailbox`
pub fn list(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"LIST"), SP, mailbox, SP, list_mailbox));

    let (remaining, (_, _, reference, _, mailbox_wildcard)) = parser(input)?;

    Ok((
        remaining,
        CommandBody::List {
            reference,
            mailbox_wildcard,
        },
    ))
}

/// `lsub = "LSUB" SP mailbox SP list-mailbox`
pub fn lsub(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"LSUB"), SP, mailbox, SP, list_mailbox));

    let (remaining, (_, _, reference, _, mailbox_wildcard)) = parser(input)?;

    Ok((
        remaining,
        CommandBody::Lsub {
            reference,
            mailbox_wildcard,
        },
    ))
}

/// `rename = "RENAME" SP mailbox SP mailbox`
///
/// Note: Use of INBOX as a destination gives a NO error
pub fn rename(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"RENAME"), SP, mailbox, SP, mailbox));

    let (remaining, (_, _, mailbox, _, new_mailbox)) = parser(input)?;

    Ok((
        remaining,
        CommandBody::Rename {
            mailbox,
            new_mailbox,
        },
    ))
}

/// `select = "SELECT" SP mailbox`
pub fn select(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"SELECT"), SP, mailbox));

    let (remaining, (_, _, mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::Select { mailbox }))
}

/// `status = "STATUS" SP mailbox SP "(" status-att *(SP status-att) ")"`
pub fn status(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((
        tag_no_case(b"STATUS"),
        SP,
        mailbox,
        SP,
        delimited(tag(b"("), separated_list0(SP, status_att), tag(b")")),
    ));

    let (remaining, (_, _, mailbox, _, attributes)) = parser(input)?;

    Ok((
        remaining,
        CommandBody::Status {
            mailbox,
            attributes,
        },
    ))
}

/// `subscribe = "SUBSCRIBE" SP mailbox`
pub fn subscribe(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"SUBSCRIBE"), SP, mailbox));

    let (remaining, (_, _, mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::Subscribe { mailbox }))
}

/// `unsubscribe = "UNSUBSCRIBE" SP mailbox`
pub fn unsubscribe(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"UNSUBSCRIBE"), SP, mailbox));

    let (remaining, (_, _, mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::Unsubscribe { mailbox }))
}

// # Command NonAuth

/// `command-nonauth = login / authenticate / "STARTTLS"`
///
/// Note: Valid only when in Not Authenticated state
pub fn command_nonauth(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = alt((
        login,
        map(authenticate, |(mechanism, initial_response)| {
            CommandBody::Authenticate {
                mechanism,
                initial_response,
            }
        }),
        value(CommandBody::StartTLS, tag_no_case(b"STARTTLS")),
    ));

    let (remaining, parsed_command_nonauth) = parser(input)?;

    Ok((remaining, parsed_command_nonauth))
}

/// `login = "LOGIN" SP userid SP password`
pub fn login(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"LOGIN"), SP, userid, SP, password));

    let (remaining, (_, _, username, _, password)) = parser(input)?;

    Ok((remaining, CommandBody::Login { username, password }))
}

#[inline]
/// `userid = astring`
pub fn userid(input: &[u8]) -> IResult<&[u8], AString> {
    astring(input)
}

#[inline]
/// `password = astring`
pub fn password(input: &[u8]) -> IResult<&[u8], AString> {
    astring(input)
}

/// `authenticate = "AUTHENTICATE" SP auth-type *(CRLF base64)` (edited)
///
/// ```text
///                                            Added by SASL-IR
///                                            |
///                                            vvvvvvvvvvvvvvvvvvv
/// authenticate = "AUTHENTICATE" SP auth-type [SP (base64 / "=")] *(CRLF base64)
///                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///                |
///                This is parsed here.
///                CRLF is parsed by upper command parser.
/// ```
pub fn authenticate(input: &[u8]) -> IResult<&[u8], (AuthMechanism, Option<Cow<[u8]>>)> {
    let mut parser = tuple((
        tag_no_case(b"AUTHENTICATE"),
        SP,
        auth_type,
        opt(preceded(
            SP,
            alt((
                map(base64, Cow::Owned),
                value(Cow::Borrowed(&b""[..]), tag("=")),
            )),
        )),
    ));

    let (remaining, (_, _, auth_type, raw_data)) = parser(input)?;

    // Server must send continuation ("+ ") at this point...

    Ok((remaining, (auth_type, raw_data)))
}

/// `authenticate = "AUTHENTICATE" SP auth-type *(CRLF base64)` (edited)
///
/// ```text
/// authenticate = base64 CRLF
///                vvvvvvvvvvvv
///                |
///                This is parsed here.
///                CRLF is additionally parsed in this parser.
///                FIXME: Multiline base64 currently does not work.
/// ```
pub fn authenticate_data(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    terminated(base64, CRLF)(input) // FIXME: many0 deleted
}

// # Command Select

/// `command-select = "CHECK" /
///                   "CLOSE" /
///                   "EXPUNGE" /
///                   copy /
///                   fetch /
///                   store /
///                   uid /
///                   search`
///
/// Note: Valid only when in Selected state
pub fn command_select(input: &[u8]) -> IResult<&[u8], CommandBody> {
    alt((
        value(CommandBody::Check, tag_no_case(b"CHECK")),
        value(CommandBody::Close, tag_no_case(b"CLOSE")),
        value(CommandBody::Expunge, tag_no_case(b"EXPUNGE")),
        copy,
        fetch,
        store,
        uid,
        search,
    ))(input)
}

/// `copy = "COPY" SP sequence-set SP mailbox`
pub fn copy(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"COPY"), SP, sequence_set, SP, mailbox));

    let (remaining, (_, _, sequence_set, _, mailbox)) = parser(input)?;

    Ok((
        remaining,
        CommandBody::Copy {
            sequence_set,
            mailbox,
            uid: false,
        },
    ))
}

/// `fetch = "FETCH" SP sequence-set SP ("ALL" /
///                                      "FULL" /
///                                      "FAST" /
///                                      fetch-att / "(" fetch-att *(SP fetch-att) ")")`
pub fn fetch(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((
        tag_no_case(b"FETCH"),
        SP,
        sequence_set,
        SP,
        alt((
            value(
                MacroOrFetchAttributes::Macro(Macro::All),
                tag_no_case(b"ALL"),
            ),
            value(
                MacroOrFetchAttributes::Macro(Macro::Fast),
                tag_no_case(b"FAST"),
            ),
            value(
                MacroOrFetchAttributes::Macro(Macro::Full),
                tag_no_case(b"FULL"),
            ),
            map(fetch_att, |fetch_att| {
                MacroOrFetchAttributes::FetchAttributes(vec![fetch_att])
            }),
            map(
                delimited(tag(b"("), separated_list0(SP, fetch_att), tag(b")")),
                MacroOrFetchAttributes::FetchAttributes,
            ),
        )),
    ));

    let (remaining, (_, _, sequence_set, _, attributes)) = parser(input)?;

    Ok((
        remaining,
        CommandBody::Fetch {
            sequence_set,
            attributes,
            uid: false,
        },
    ))
}

/// `store = "STORE" SP sequence-set SP store-att-flags`
pub fn store(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"STORE"), SP, sequence_set, SP, store_att_flags));

    let (remaining, (_, _, sequence_set, _, (kind, response, flags))) = parser(input)?;

    Ok((
        remaining,
        CommandBody::Store {
            sequence_set,
            kind,
            response,
            flags,
            uid: false,
        },
    ))
}

/// `store-att-flags = (["+" / "-"] "FLAGS" [".SILENT"]) SP (flag-list / (flag *(SP flag)))`
pub fn store_att_flags(input: &[u8]) -> IResult<&[u8], (StoreType, StoreResponse, Vec<Flag>)> {
    let mut parser = tuple((
        tuple((
            map(
                opt(alt((
                    value(StoreType::Add, tag(b"+")),
                    value(StoreType::Remove, tag(b"-")),
                ))),
                |type_| match type_ {
                    Some(type_) => type_,
                    None => StoreType::Replace,
                },
            ),
            tag_no_case(b"FLAGS"),
            map(opt(tag_no_case(b".SILENT")), |x| match x {
                Some(_) => StoreResponse::Silent,
                None => StoreResponse::Answer,
            }),
        )),
        SP,
        alt((flag_list, separated_list1(SP, flag))),
    ));

    let (remaining, ((store_type, _, store_response), _, flag_list)) = parser(input)?;

    Ok((remaining, (store_type, store_response, flag_list)))
}

/// `uid = "UID" SP (copy / fetch / search / store)`
///
/// Note: Unique identifiers used instead of message sequence numbers
pub fn uid(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"UID"), SP, alt((copy, fetch, search, store))));

    let (remaining, (_, _, mut cmd)) = parser(input)?;

    match cmd {
        CommandBody::Copy { ref mut uid, .. }
        | CommandBody::Fetch { ref mut uid, .. }
        | CommandBody::Search { ref mut uid, .. }
        | CommandBody::Store { ref mut uid, .. } => *uid = true,
        _ => unreachable!(),
    }

    Ok((remaining, cmd))
}

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

    let (remaining, (_, charset, criteria)) = parser(input)?;

    let criteria = match criteria.len() {
        0 => unreachable!(),
        1 => criteria[0].clone(),
        _ => SearchKey::And(unsafe { NonEmptyVec::new_unchecked(criteria) }),
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
                |val| match val.len() {
                    0 => unreachable!(),
                    1 => val[0].clone(),
                    _ => SearchKey::And(unsafe { NonEmptyVec::new_unchecked(val) }),
                },
            ),
        )),
    ))(input)
}

#[cfg(test)]
mod test {
    use std::{
        convert::{TryFrom, TryInto},
        num::NonZeroU32,
    };

    use imap_types::{
        fetch_attributes::FetchAttribute,
        section::Section,
        sequence::{SeqNo, Sequence, SequenceSet as SequenceSetData},
    };

    use super::*;

    #[test]
    fn test_fetch() {
        //let (rem, val) = fetch(b"fetch 1:5 (flags)").unwrap();
        //println!("{:?}, {:?}", rem, val);

        println!("{:#?}", fetch(b"fetch 1:1 (flags)???"));
    }

    #[test]
    fn test_fetch_att() {
        let tests = [
            (FetchAttribute::Envelope, "ENVELOPE???"),
            (FetchAttribute::Flags, "FLAGS???"),
            (FetchAttribute::InternalDate, "INTERNALDATE???"),
            (FetchAttribute::Rfc822, "RFC822???"),
            (FetchAttribute::Rfc822Header, "RFC822.HEADER???"),
            (FetchAttribute::Rfc822Size, "RFC822.SIZE???"),
            (FetchAttribute::Rfc822Text, "RFC822.TEXT???"),
            (FetchAttribute::Body, "BODY???"),
            (FetchAttribute::BodyStructure, "BODYSTRUCTURE???"),
            (FetchAttribute::Uid, "UID???"),
            (
                FetchAttribute::BodyExt {
                    partial: None,
                    peek: false,
                    section: None,
                },
                "BODY[]???",
            ),
            (
                FetchAttribute::BodyExt {
                    partial: None,
                    peek: true,
                    section: None,
                },
                "BODY.PEEK[]???",
            ),
            (
                FetchAttribute::BodyExt {
                    partial: None,
                    peek: true,
                    section: Some(Section::Text(None)),
                },
                "BODY.PEEK[TEXT]???",
            ),
            (
                FetchAttribute::BodyExt {
                    partial: Some((42, NonZeroU32::try_from(1337).unwrap())),
                    peek: true,
                    section: Some(Section::Text(None)),
                },
                "BODY.PEEK[TEXT]<42.1337>???",
            ),
        ];

        let expected_remainder = "???".as_bytes();

        for (expected, test) in tests {
            let (got_remainder, got) = fetch_att(test.as_bytes()).unwrap();

            assert_eq!(expected, got);
            assert_eq!(expected_remainder, got_remainder);
        }
    }

    #[test]
    fn test_search() {
        use SearchKey::*;
        use SeqNo::Value;
        use Sequence::*;

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
    fn test_search_key() {
        assert!(search_key(1)(b"1:5|").is_ok());
        assert!(search_key(1)(b"(1:5)|").is_err());
        assert!(search_key(2)(b"(1:5)|").is_ok());
        assert!(search_key(2)(b"((1:5))|").is_err());
    }

    #[test]
    #[cfg(feature = "ext_enable")]
    fn test_enable() {
        use imap_types::extensions::rfc5161::{CapabilityEnable, Utf8Kind};

        let got = command(b"A123 enable UTF8=ACCEPT\r\n").unwrap().1;
        assert_eq!(
            Command::new(
                "A123".try_into().unwrap(),
                CommandBody::Enable {
                    capabilities: vec![CapabilityEnable::Utf8(Utf8Kind::Accept),]
                        .try_into()
                        .unwrap()
                }
            ),
            got
        );
    }
}
