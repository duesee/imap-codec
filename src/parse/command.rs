use crate::{
    parse::{
        auth_type,
        core::{astring, atom, base64, charset, literal, number, nz_number, tag_imap},
        datetime::{date, date_time},
        flag::{flag, flag_list},
        header::header_fld_name,
        mailbox::{list_mailbox, mailbox},
        section::section,
        sequence::sequence_set,
        status::status_att,
    },
    types::{
        command::{Command, CommandBody, SearchKey},
        core::astr,
        data_items::{DataItem, Macro, MacroOrDataItems},
        flag::Flag,
        AuthMechanism, StoreResponse, StoreType,
    },
};
use abnf_core::streaming::{CRLF_relaxed as CRLF, SP};
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    combinator::{map, map_opt, map_res, opt, value},
    multi::{many1, separated_list, separated_nonempty_list},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

/// command = tag SP (command-any /
///                   command-auth /
///                   command-nonauth /
///                   command-select) CRLF
pub fn command(input: &[u8]) -> IResult<&[u8], Command> {
    let parser = tuple((
        tag_imap,
        SP,
        alt((command_any, command_auth, command_nonauth, command_select)),
        CRLF,
    ));

    let (remaining, (tag, _, command_body, _)) = parser(input)?;

    Ok((remaining, Command::new(tag, command_body)))
}

/// # Command Any

/// command-any = "CAPABILITY" / "LOGOUT" / "NOOP" / x-command
///
/// Note: Valid in all states
fn command_any(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = alt((
        value(CommandBody::Capability, tag_no_case(b"CAPABILITY")),
        value(CommandBody::Logout, tag_no_case(b"LOGOUT")),
        value(CommandBody::Noop, tag_no_case(b"NOOP")),
        // x-command = "X" atom <experimental command arguments>
    ));

    let (remaining, parsed_command_any) = parser(input)?;

    Ok((remaining, parsed_command_any))
}

/// # Command Auth

/// command-auth = append / create / delete /
///                examine / list / lsub /
///                rename / select / status /
///                subscribe / unsubscribe /
///                idle ; RFC 2177
///
/// Note: Valid only in Authenticated or Selected state
fn command_auth(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = alt((
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
        idle, // RFC 2177
    ));

    let (remaining, parsed_command_auth) = parser(input)?;

    Ok((remaining, parsed_command_auth))
}

/// append = "APPEND" SP mailbox [SP flag-list] [SP date-time] SP literal
fn append(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((
        tag_no_case(b"APPEND"),
        SP,
        mailbox,
        opt(preceded(SP, flag_list)),
        opt(preceded(SP, date_time)),
        SP,
        literal,
    ));

    let (remaining, (_, _, mailbox, flags, date_time, _, literal)) = parser(input)?;

    Ok((
        remaining,
        // FIXME: do not use unwrap()
        CommandBody::Append {
            mailbox,
            flags: flags.unwrap_or_default(),
            date: date_time.map(|maybe_date| maybe_date.unwrap()),
            message: literal.to_vec(),
        },
    ))
}

/// create = "CREATE" SP mailbox
///
/// Note: Use of INBOX gives a NO error
fn create(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"CREATE"), SP, mailbox));

    let (remaining, (_, _, mailbox_name)) = parser(input)?;

    Ok((remaining, CommandBody::Create { mailbox_name }))
}

/// delete = "DELETE" SP mailbox
///
/// Note: Use of INBOX gives a NO error
fn delete(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"DELETE"), SP, mailbox));

    let (remaining, (_, _, mailbox_name)) = parser(input)?;

    Ok((remaining, CommandBody::Delete { mailbox_name }))
}

/// examine = "EXAMINE" SP mailbox
fn examine(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"EXAMINE"), SP, mailbox));

    let (remaining, (_, _, mailbox_name)) = parser(input)?;

    Ok((remaining, CommandBody::Examine { mailbox_name }))
}

/// list = "LIST" SP mailbox SP list-mailbox
fn list(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"LIST"), SP, mailbox, SP, list_mailbox));

    let (remaining, (_, _, reference, _, mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::List { reference, mailbox }))
}

/// lsub = "LSUB" SP mailbox SP list-mailbox
fn lsub(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"LSUB"), SP, mailbox, SP, list_mailbox));

    let (remaining, (_, _, reference, _, mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::Lsub { reference, mailbox }))
}

/// rename = "RENAME" SP mailbox SP mailbox
///
/// Note: Use of INBOX as a destination gives a NO error
fn rename(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"RENAME"), SP, mailbox, SP, mailbox));

    let (remaining, (_, _, existing_mailbox_name, _, new_mailbox_name)) = parser(input)?;

    Ok((
        remaining,
        CommandBody::Rename {
            existing_mailbox_name,
            new_mailbox_name,
        },
    ))
}

/// select = "SELECT" SP mailbox
fn select(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"SELECT"), SP, mailbox));

    let (remaining, (_, _, mailbox_name)) = parser(input)?;

    Ok((remaining, CommandBody::Select { mailbox_name }))
}

/// status = "STATUS" SP mailbox SP "(" status-att *(SP status-att) ")"
fn status(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((
        tag_no_case(b"STATUS"),
        SP,
        mailbox,
        SP,
        delimited(tag(b"("), separated_list(SP, status_att), tag(b")")),
    ));

    let (remaining, (_, _, mailbox, _, items)) = parser(input)?;

    Ok((remaining, CommandBody::Status { mailbox, items }))
}

/// subscribe = "SUBSCRIBE" SP mailbox
fn subscribe(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"SUBSCRIBE"), SP, mailbox));

    let (remaining, (_, _, mailbox_name)) = parser(input)?;

    Ok((remaining, CommandBody::Subscribe { mailbox_name }))
}

/// unsubscribe = "UNSUBSCRIBE" SP mailbox
fn unsubscribe(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"UNSUBSCRIBE"), SP, mailbox));

    let (remaining, (_, _, mailbox_name)) = parser(input)?;

    Ok((remaining, CommandBody::Unsubscribe { mailbox_name }))
}

/// idle = "IDLE" CRLF "DONE"
///        ^^^^^^^^^^^
///        |
///        parsed as command (CRLF is consumed in upper command parser)
///
/// Valid only in Authenticated or Selected state
fn idle(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = value(CommandBody::Idle, tag_no_case("IDLE"));

    let (remaining, parsed_idle) = parser(input)?;

    Ok((remaining, parsed_idle))
}

/// This parser must be executed *instead* of the command parser
/// when the server is in the IDLE state.
///
/// idle = "IDLE" CRLF "DONE"
///                    ^^^^^^
///                    |
///                    applied as separate parser (CRLF is not consumed through the command
///                                                parser and must be consumed here)
pub fn idle_done(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag_no_case("DONE\r\n")(input)
}

/// # Command NonAuth

/// command-nonauth = login / authenticate / "STARTTLS"
///
/// Note: Valid only when in Not Authenticated state
fn command_nonauth(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = alt((
        login,
        map(authenticate, |(mechanism, ir)| CommandBody::Authenticate {
            mechanism,
            initial_response: ir.map(|i| i.to_owned()),
        }),
        value(CommandBody::StartTLS, tag_no_case(b"STARTTLS")),
    ));

    let (remaining, parsed_command_nonauth) = parser(input)?;

    Ok((remaining, parsed_command_nonauth))
}

/// login = "LOGIN" SP userid SP password
fn login(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"LOGIN"), SP, userid, SP, password));

    let (remaining, (_, _, username, _, password)) = parser(input)?;

    Ok((
        remaining,
        CommandBody::Login {
            username: username.to_owned(),
            password: password.to_owned(),
        },
    ))
}

/// userid = astring
fn userid(input: &[u8]) -> IResult<&[u8], astr> {
    astring(input)
}

/// password = astring
fn password(input: &[u8]) -> IResult<&[u8], astr> {
    astring(input)
}

/// authenticate = "AUTHENTICATE" SP auth-type *(CRLF base64)
///
/// SASL-IR:
/// authenticate = "AUTHENTICATE" SP auth-type [SP (base64 / "=")] (CRLF base64)
///                 ; redefine AUTHENTICATE from [RFC3501]
fn authenticate(input: &[u8]) -> IResult<&[u8], (AuthMechanism, Option<&str>)> {
    let parser = tuple((
        tag_no_case(b"AUTHENTICATE"),
        SP,
        auth_type,
        opt(preceded(
            SP,
            alt((base64, map_res(tag("="), std::str::from_utf8))),
        )),
    ));

    let (remaining, (_, _, auth_type, ir)) = parser(input)?;

    // Server must send "+" at this point...

    Ok((remaining, (auth_type, ir)))
}

/// Use this parser instead of command when doing authentication.
///
///                                                                Parsed here (because this is not parsed through command,
///                                                                             CRLF must be parsed additionally)
///                                                                |
///                                                                vvvvvvvvvvvvvv
/// authenticate = "AUTHENTICATE" SP auth-type [SP (base64 / "=")] *(CRLF base64) // TODO: why the "="?
///                                            ^^^^^^^^^^^^^^^^^^^
///                                            |
///                                            Added by SASL-IR (RFC RFC 4959)
pub fn authenticate_data(input: &[u8]) -> IResult<&[u8], String> {
    let parser = terminated(base64, CRLF); // FIXME: many0 deleted

    let (remaining, parsed_authenticate_data) = parser(input)?;

    Ok((remaining, parsed_authenticate_data.to_owned()))
}

/// # Command Select

/// command-select = "CHECK" / "CLOSE" / "EXPUNGE" / copy / fetch / store / uid / search
///
/// Note: Valid only when in Selected state
fn command_select(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = alt((
        value(CommandBody::Check, tag_no_case(b"CHECK")),
        value(CommandBody::Close, tag_no_case(b"CLOSE")),
        value(CommandBody::Expunge, tag_no_case(b"EXPUNGE")),
        copy,
        fetch,
        store,
        uid,
        search,
    ));

    let (remaining, parsed_command_select) = parser(input)?;

    Ok((remaining, parsed_command_select))
}

/// copy = "COPY" SP sequence-set SP mailbox
fn copy(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"COPY"), SP, sequence_set, SP, mailbox));

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

/// fetch = "FETCH" SP sequence-set SP ("ALL" /
///                                     "FULL" /
///                                     "FAST" /
///                                     fetch-att / "(" fetch-att *(SP fetch-att) ")")
fn fetch(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((
        tag_no_case(b"FETCH"),
        SP,
        sequence_set,
        SP,
        alt((
            value(MacroOrDataItems::Macro(Macro::All), tag_no_case(b"ALL")),
            value(MacroOrDataItems::Macro(Macro::Fast), tag_no_case(b"FAST")),
            value(MacroOrDataItems::Macro(Macro::Full), tag_no_case(b"FULL")),
            map(fetch_att, |fetch_att| {
                MacroOrDataItems::DataItems(vec![fetch_att])
            }),
            map(
                delimited(tag(b"("), separated_list(SP, fetch_att), tag(b")")),
                MacroOrDataItems::DataItems,
            ),
        )),
    ));

    let (remaining, (_, _, sequence_set, _, items)) = parser(input)?;

    Ok((
        remaining,
        CommandBody::Fetch {
            sequence_set,
            items,
            uid: false,
        },
    ))
}

/// fetch-att = "ENVELOPE" /
///             "FLAGS" /
///             "INTERNALDATE" /
///             "RFC822" [".HEADER" / ".SIZE" / ".TEXT"] /
///             "BODY" ["STRUCTURE"] /
///             "UID" /
///             "BODY" section ["<" number "." nz-number ">"] /
///             "BODY.PEEK" section ["<" number "." nz-number ">"]
fn fetch_att(input: &[u8]) -> IResult<&[u8], DataItem> {
    let parser = alt((
        value(DataItem::Envelope, tag_no_case(b"ENVELOPE")),
        value(DataItem::Flags, tag_no_case(b"FLAGS")),
        value(DataItem::InternalDate, tag_no_case(b"INTERNALDATE")),
        value(DataItem::BodyStructure, tag_no_case(b"BODYSTRUCTURE")),
        map(
            tuple((
                tag_no_case(b"BODY.PEEK"),
                section,
                opt(delimited(
                    tag(b"<"),
                    tuple((number, tag(b"."), nz_number)),
                    tag(b">"),
                )),
            )),
            |(_, section, byterange)| DataItem::BodyExt {
                section,
                partial: byterange.map(|(start, _, end)| (start, end)),
                peek: true,
            },
        ),
        map(
            tuple((
                tag_no_case(b"BODY"),
                section,
                opt(delimited(
                    tag(b"<"),
                    tuple((number, tag(b"."), nz_number)),
                    tag(b">"),
                )),
            )),
            |(_, section, byterange)| DataItem::BodyExt {
                section,
                partial: byterange.map(|(start, _, end)| (start, end)),
                peek: false,
            },
        ),
        value(DataItem::Body, tag_no_case(b"BODY")),
        value(DataItem::Uid, tag_no_case(b"UID")),
        value(DataItem::Rfc822Header, tag_no_case(b"RFC822.HEADER")),
        value(DataItem::Rfc822Size, tag_no_case(b"RFC822.SIZE")),
        value(DataItem::Rfc822Text, tag_no_case(b"RFC822.TEXT")),
    ));

    let (remaining, parsed_fetch_att) = parser(input)?;

    Ok((remaining, parsed_fetch_att))
}

/// store = "STORE" SP sequence-set SP store-att-flags
fn store(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"STORE"), SP, sequence_set, SP, store_att_flags));

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

/// store-att-flags = (["+" / "-"] "FLAGS" [".SILENT"]) SP (flag-list / (flag *(SP flag)))
fn store_att_flags(input: &[u8]) -> IResult<&[u8], (StoreType, StoreResponse, Vec<Flag>)> {
    let parser = tuple((
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
        alt((flag_list, separated_nonempty_list(SP, flag))),
    ));

    let (remaining, ((store_type, _, store_response), _, flag_list)) = parser(input)?;

    Ok((remaining, (store_type, store_response, flag_list)))
}

/// uid = "UID" SP (copy / fetch / search / store)
///
/// Note: Unique identifiers used instead of message sequence numbers
fn uid(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"UID"), SP, alt((copy, fetch, search, store))));

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

/// search = "SEARCH" [SP "CHARSET" SP charset] 1*(SP search-key)
///
/// Note: CHARSET argument to MUST be registered with IANA
///
/// errata id: 261
fn search(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((
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
        1 => criteria.first().unwrap().clone(),
        _ => SearchKey::And(criteria),
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

/// This parser is recursively defined. Thus, in order to not overflow the stack,
/// it is needed to limit how may recursions are allowed. (8 should suffice).
fn search_key(remaining_recursions: usize) -> impl Fn(&[u8]) -> IResult<&[u8], SearchKey> {
    move |input: &[u8]| search_key_limited(input, remaining_recursions)
}

/// search-key = "ALL" /
///              "ANSWERED" /
///              "BCC" SP astring /
///              "BEFORE" SP date /
///              "BODY" SP astring /
///              "CC" SP astring /
///              "DELETED" /
///              "FLAGGED" /
///              "FROM" SP astring /
///              "KEYWORD" SP flag-keyword /
///              "NEW" /
///              "OLD" /
///              "ON" SP date /
///              "RECENT" /
///              "SEEN" /
///              "SINCE" SP date /
///              "SUBJECT" SP astring /
///              "TEXT" SP astring /
///              "TO" SP astring /
///              "UNANSWERED" /
///              "UNDELETED" /
///              "UNFLAGGED" /
///              "UNKEYWORD" SP flag-keyword /
///              "UNSEEN" /
///                ; Above this line were in [IMAP2]
///              "DRAFT" /
///              "HEADER" SP header-fld-name SP astring /
///              "LARGER" SP number /
///              "NOT" SP search-key /
///              "OR" SP search-key SP search-key /
///              "SENTBEFORE" SP date /
///              "SENTON" SP date /
///              "SENTSINCE" SP date /
///              "SMALLER" SP number /
///              "UID" SP sequence-set /
///              "UNDRAFT" /
///              sequence-set /
///              "(" search-key *(SP search-key) ")"
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

    let parser = alt((
        alt((
            value(SearchKey::All, tag_no_case(b"ALL")),
            value(SearchKey::Answered, tag_no_case(b"ANSWERED")),
            map(tuple((tag_no_case(b"BCC"), SP, astring)), |(_, _, val)| {
                SearchKey::Bcc(val.to_owned())
            }),
            map(
                tuple((tag_no_case(b"BEFORE"), SP, map_opt(date, |date| date))),
                |(_, _, date)| SearchKey::Before(date),
            ),
            map(tuple((tag_no_case(b"BODY"), SP, astring)), |(_, _, val)| {
                SearchKey::Body(val.to_owned())
            }),
            map(tuple((tag_no_case(b"CC"), SP, astring)), |(_, _, val)| {
                SearchKey::Cc(val.to_owned())
            }),
            value(SearchKey::Deleted, tag_no_case(b"DELETED")),
            value(SearchKey::Flagged, tag_no_case(b"FLAGGED")),
            map(tuple((tag_no_case(b"FROM"), SP, astring)), |(_, _, val)| {
                SearchKey::From(val.to_owned())
            }),
            map(
                // Note: `flag_keyword` parser returns `Flag`. Because Rust does not have first-class enum variants
                // it is not possible to fix SearchKey(Flag::Keyword), but only SearchKey(Flag).
                // Thus `SearchKey::Keyword(Atom)` is used instead. This is, why we use also `atom` parser here and not `flag_keyword` parser.
                tuple((tag_no_case(b"KEYWORD"), SP, atom)),
                |(_, _, val)| SearchKey::Keyword(val.to_owned()),
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
                |(_, _, val)| SearchKey::Subject(val.to_owned()),
            ),
            map(tuple((tag_no_case(b"TEXT"), SP, astring)), |(_, _, val)| {
                SearchKey::Text(val.to_owned())
            }),
            map(tuple((tag_no_case(b"TO"), SP, astring)), |(_, _, val)| {
                SearchKey::To(val.to_owned())
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
                |(_, _, val)| SearchKey::Unkeyword(val.to_owned()),
            ),
            value(SearchKey::Unseen, tag_no_case(b"UNSEEN")),
            value(SearchKey::Draft, tag_no_case(b"DRAFT")),
            map(
                tuple((tag_no_case(b"HEADER"), SP, header_fld_name, SP, astring)),
                |(_, _, key, _, val)| SearchKey::Header(key.to_owned(), val.to_owned()),
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
                delimited(
                    tag(b"("),
                    separated_nonempty_list(SP, search_key),
                    tag(b")"),
                ),
                |val| match val.len() {
                    0 => unreachable!(),
                    1 => val.first().unwrap().clone(),
                    _ => SearchKey::And(val),
                },
            ),
        )),
    ));

    let (remaining, parsed_search_key) = parser(input)?;

    Ok((remaining, parsed_search_key))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::sequence::{SeqNo, Sequence};

    #[test]
    fn test_fetch() {
        //let (rem, val) = fetch(b"fetch 1:5 (flags)").unwrap();
        //println!("{:?}, {:?}", rem, val);

        println!("{:#?}", fetch(b"fetch 1:1 (flags)???"));
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
                criteria: Uid(vec![Single(Value(5))]),
                uid: false,
            }
        );

        let (_rem, val) = search(b"search (uid 5 or uid 5 (uid 1 uid 2) not (uid 5))???").unwrap();
        let expected = CommandBody::Search {
            charset: None,
            criteria: And(vec![
                Uid(vec![Single(Value(5))]),
                Or(
                    Box::new(Uid(vec![Single(Value(5))])),
                    Box::new(And(vec![
                        Uid(vec![Single(Value(1))]),
                        Uid(vec![Single(Value(2))]),
                    ])),
                ),
                Not(Box::new(Uid(vec![Single(Value(5))]))),
            ]),
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
}
