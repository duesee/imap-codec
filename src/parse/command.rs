use crate::{
    parse::{
        auth_type,
        base64::base64,
        charset,
        core::{astring, literal, number, nz_number},
        crlf,
        datetime::{date, date_time},
        flag::{flag, flag_keyword, flag_list},
        header::header_fld_name,
        mailbox::{list_mailbox, mailbox},
        section::section,
        sequence::sequence_set,
        sp,
        status::status_att,
        tag as imap_tag,
    },
    types::{
        command::{Command, CommandBody, CommandBodyUid, SearchKey},
        core::AString,
        data_items::{DataItem, Macro, MacroOrDataItems},
        message_attributes::Flag,
        AuthMechanism, StoreResponse, StoreType,
    },
};
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    combinator::{map, map_opt, opt, value},
    multi::{many1, separated_list, separated_nonempty_list},
    sequence::{delimited, tuple},
    IResult,
};

/// command = tag SP (command-any / command-auth / command-nonauth / command-select) CRLF
///            ; Modal based on state
pub fn command(input: &[u8]) -> IResult<&[u8], Command> {
    let parser = tuple((
        imap_tag,
        sp,
        alt((command_any, command_auth, command_nonauth, command_select)),
        crlf,
    ));

    let (remaining, (tag, _, command_body, _)) = parser(input)?;

    Ok((remaining, Command::new(tag, command_body)))
}

/// # Command Any

/// command-any = "CAPABILITY" / "LOGOUT" / "NOOP" / x-command
///                ; Valid in all states
pub fn command_any(input: &[u8]) -> IResult<&[u8], CommandBody> {
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

/// command-auth = append / create / delete / examine / list / lsub / rename / select / status / subscribe / unsubscribe
///                 ; Valid only in Authenticated or Selected state
pub fn command_auth(input: &[u8]) -> IResult<&[u8], CommandBody> {
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
pub fn append(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((
        tag_no_case(b"APPEND"),
        sp,
        mailbox,
        opt(map(tuple((sp, flag_list)), |(_, flag_list)| flag_list)),
        opt(map(tuple((sp, date_time)), |(_, date_time)| date_time)),
        sp,
        literal,
    ));

    let (remaining, (_, _, mailbox, flag_list, date_time, _, literal)) = parser(input)?;

    Ok((
        remaining,
        // FIXME: do not use unwrap()
        CommandBody::Append(
            mailbox,
            flag_list,
            date_time.map(|maybe_date| maybe_date.unwrap()),
            literal.to_vec(),
        ),
    ))
}

/// create = "CREATE" SP mailbox
///           ; Use of INBOX gives a NO error
pub fn create(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"CREATE"), sp, mailbox));

    let (remaining, (_, _, mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::Create(mailbox)))
}

/// delete = "DELETE" SP mailbox
///           ; Use of INBOX gives a NO error
pub fn delete(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"DELETE"), sp, mailbox));

    let (remaining, (_, _, mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::Delete(mailbox)))
}

/// examine = "EXAMINE" SP mailbox
pub fn examine(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"EXAMINE"), sp, mailbox));

    let (remaining, (_, _, mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::Examine(mailbox)))
}

/// list = "LIST" SP mailbox SP list-mailbox
pub fn list(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"LIST"), sp, mailbox, sp, list_mailbox));

    let (remaining, (_, _, mailbox, _, list_mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::List(mailbox, list_mailbox)))
}

/// lsub = "LSUB" SP mailbox SP list-mailbox
pub fn lsub(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"LSUB"), sp, mailbox, sp, list_mailbox));

    let (remaining, (_, _, mailbox, _, list_mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::Lsub(mailbox, list_mailbox)))
}

/// rename = "RENAME" SP mailbox SP mailbox
///           ; Use of INBOX as a destination gives a NO error
pub fn rename(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"RENAME"), sp, mailbox, sp, mailbox));

    let (remaining, (_, _, mailbox_old, _, mailbox_new)) = parser(input)?;

    Ok((remaining, CommandBody::Rename(mailbox_old, mailbox_new)))
}

/// select = "SELECT" SP mailbox
pub fn select(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"SELECT"), sp, mailbox));

    let (remaining, (_, _, mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::Select(mailbox)))
}

/// status = "STATUS" SP mailbox SP "(" status-att *(SP status-att) ")"
pub fn status(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((
        tag_no_case(b"STATUS"),
        sp,
        mailbox,
        sp,
        delimited(tag(b"("), separated_list(sp, status_att), tag(b")")),
    ));

    let (remaining, (_, _, mailbox, _, status_att)) = parser(input)?;

    Ok((remaining, CommandBody::Status(mailbox, status_att)))
}

/// subscribe = "SUBSCRIBE" SP mailbox
pub fn subscribe(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"SUBSCRIBE"), sp, mailbox));

    let (remaining, (_, _, mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::Subscribe(mailbox)))
}

/// unsubscribe = "UNSUBSCRIBE" SP mailbox
pub fn unsubscribe(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"UNSUBSCRIBE"), sp, mailbox));

    let (remaining, (_, _, mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::Unsubscribe(mailbox)))
}

pub fn idle(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = value(CommandBody::Idle, tag_no_case("IDLE"));

    let (remaining, parsed_idle) = parser(input)?;

    Ok((remaining, parsed_idle))
}

/// # Command NonAuth

/// command-nonauth = login / authenticate / "STARTTLS"
///                    ; Valid only when in Not Authenticated state
pub fn command_nonauth(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = alt((
        login,
        map(authenticate, |mechanism| {
            CommandBody::Authenticate(mechanism)
        }),
        value(CommandBody::StartTLS, tag_no_case(b"STARTTLS")),
    ));

    let (remaining, parsed_command_nonauth) = parser(input)?;

    Ok((remaining, parsed_command_nonauth))
}

/// login = "LOGIN" SP userid SP password
pub fn login(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"LOGIN"), sp, userid, sp, password));

    let (remaining, (_, _, userid, _, password)) = parser(input)?;

    Ok((remaining, CommandBody::Login(userid, password)))
}

/// userid = astring
fn userid(input: &[u8]) -> IResult<&[u8], AString> {
    let parser = astring;

    let (remaining, parsed_userid) = parser(input)?;

    Ok((remaining, parsed_userid))
}

/// password = astring
fn password(input: &[u8]) -> IResult<&[u8], AString> {
    let parser = astring;

    let (remaining, parsed_password) = parser(input)?;

    Ok((remaining, parsed_password))
}

/// authenticate = "AUTHENTICATE" SP auth-type *(CRLF base64)
pub fn authenticate(input: &[u8]) -> IResult<&[u8], AuthMechanism> {
    let parser = tuple((tag_no_case(b"AUTHENTICATE"), sp, auth_type));

    let (remaining, (_, _, auth_type)) = parser(input)?;

    // Server must send "+" at this point...

    let output = match auth_type.0.to_lowercase().as_ref() {
        "plain" => AuthMechanism::Plain,
        other => AuthMechanism::Other(other.to_owned()),
    };

    Ok((remaining, output))
}

pub fn authenticate_data(input: &[u8]) -> IResult<&[u8], String> {
    let parser = map(tuple((base64, crlf)), |(line, _)| line); // FIXME: many0 deleted

    let (remaining, parsed_authenticate_data) = parser(input)?;

    Ok((remaining, parsed_authenticate_data.to_owned()))
}

/// # Command Select

/// command-select = "CHECK" / "CLOSE" / "EXPUNGE" / copy / fetch / store / uid / search
///                   ; Valid only when in Selected state
pub fn command_select(input: &[u8]) -> IResult<&[u8], CommandBody> {
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
pub fn copy(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"COPY"), sp, sequence_set, sp, mailbox));

    let (remaining, (_, _, seq_set, _, mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::Copy(seq_set, mailbox)))
}

/// fetch = "FETCH" SP sequence-set SP ("ALL" / "FULL" / "FAST" / fetch-att / "(" fetch-att *(SP fetch-att) ")")
pub fn fetch(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((
        tag_no_case(b"FETCH"),
        sp,
        sequence_set,
        sp,
        alt((
            value(MacroOrDataItems::Macro(Macro::All), tag_no_case(b"ALL")),
            value(MacroOrDataItems::Macro(Macro::Fast), tag_no_case(b"FAST")),
            value(MacroOrDataItems::Macro(Macro::Full), tag_no_case(b"FULL")),
            map(fetch_att, |fetch_att| {
                MacroOrDataItems::DataItems(vec![fetch_att])
            }),
            map(
                delimited(tag(b"("), separated_list(sp, fetch_att), tag(b")")),
                |fetch_attrs| MacroOrDataItems::DataItems(fetch_attrs),
            ),
        )),
    ));

    let (remaining, (_, _, seq, _, fetch_attrs)) = parser(input)?;

    Ok((remaining, CommandBody::Fetch(seq, fetch_attrs)))
}

/// fetch-att = "ENVELOPE" / "FLAGS" / "INTERNALDATE" /
///             "RFC822" [".HEADER" / ".SIZE" / ".TEXT"] /
///             "BODY" ["STRUCTURE"] / "UID" /
///             "BODY" section ["<" number "." nz-number ">"] /
///             "BODY.PEEK" section ["<" number "." nz-number ">"]
fn fetch_att(input: &[u8]) -> IResult<&[u8], DataItem> {
    let parser = alt((
        // TODO: ordering is important
        value(DataItem::Envelope, tag_no_case(b"ENVELOPE")),
        value(DataItem::Flags, tag_no_case(b"FLAGS")),
        value(DataItem::InternalDate, tag_no_case(b"INTERNALDATE")),
        value(DataItem::BodyStructure, tag_no_case(b"BODYSTRUCTURE")),
        map(
            tuple((
                tag_no_case(b"BODY.PEEK"),
                section,
                opt(tuple((
                    tag_no_case(b"<"),
                    number,
                    tag_no_case(b"."),
                    nz_number,
                    tag_no_case(b">"),
                ))),
            )),
            |(_, section, byterange)| {
                DataItem::BodyPeek(section, byterange.map(|(_, start, _, end, _)| (start, end)))
            },
        ),
        map(
            tuple((
                tag_no_case(b"BODY"),
                section,
                opt(tuple((
                    tag_no_case(b"<"),
                    number,
                    tag_no_case(b"."),
                    nz_number,
                    tag_no_case(b">"),
                ))),
            )),
            |(_, section, byterange)| {
                DataItem::BodyExt(section, byterange.map(|(_, start, _, end, _)| (start, end)))
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
pub fn store(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"STORE"), sp, sequence_set, sp, store_att_flags));

    let (remaining, (_, _, sequence_set, _, (store_type, store_response, flag_list))) =
        parser(input)?;

    Ok((
        remaining,
        CommandBody::Store(sequence_set, store_type, store_response, flag_list),
    ))
}

/// store-att-flags = (["+" / "-"] "FLAGS" [".SILENT"]) SP (flag-list / (flag *(SP flag)))
fn store_att_flags(input: &[u8]) -> IResult<&[u8], (StoreType, StoreResponse, Vec<Flag>)> {
    let parser = tuple((
        tuple((
            map(
                opt(alt((
                    value(StoreType::Add, tag_no_case(b"+")),
                    value(StoreType::Remove, tag_no_case(b"-")),
                ))),
                |type_| match type_ {
                    Some(type_) => type_,
                    None => StoreType::Replace,
                },
            ),
            tag_no_case(b"FLAGS"),
            map(opt(tag_no_case(b".SILENT")), |x| match x {
                Some(_) => StoreResponse::Answer,
                None => StoreResponse::Silent,
            }),
        )),
        sp,
        alt((flag_list, separated_nonempty_list(sp, flag))),
    ));

    let (remaining, ((store_type, _, store_response), _, flag_list)) = parser(input)?;

    Ok((remaining, (store_type, store_response, flag_list)))
}

/// uid = "UID" SP (copy / fetch / search / store)
///        ; Unique identifiers used instead of message
///        ; sequence numbers
pub fn uid(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((tag_no_case(b"UID"), sp, alt((copy, fetch, search, store))));

    let (remaining, (_, _, cmd)) = parser(input)?;

    let uid_body = match cmd {
        CommandBody::Copy(seq, mailbox) => CommandBodyUid::Copy(seq, mailbox),
        CommandBody::Fetch(seq, attrs) => CommandBodyUid::Fetch(seq, attrs),
        CommandBody::Search(charset, criteria) => CommandBodyUid::Search(charset, criteria),
        CommandBody::Store(seq, store_type, store_response, flag_list) => {
            CommandBodyUid::Store(seq, store_type, store_response, flag_list)
        }
        _ => unreachable!(),
    };

    Ok((remaining, CommandBody::Uid(uid_body)))
}

/// ; errata id: 261
/// search = "SEARCH" [SP "CHARSET" SP charset] 1*(SP search-key)
///           ; CHARSET argument to MUST be registered with IANA
pub fn search(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let parser = tuple((
        tag_no_case(b"SEARCH"),
        opt(map(
            tuple((sp, tag_no_case(b"CHARSET"), sp, charset)),
            |(_, _, _, charset)| charset,
        )),
        many1(map(tuple((sp, search_key)), |(_, search_key)| search_key)),
    ));

    let (remaining, (_, maybe_charset, criteria)) = parser(input)?;

    let criteria = match criteria.len() {
        0 => unreachable!(),
        1 => criteria.first().unwrap().clone(),
        _ => SearchKey::And(criteria),
    };

    Ok((remaining, CommandBody::Search(maybe_charset, criteria)))
}

/// search-key = "ALL" / "ANSWERED" / "BCC" SP astring /
///              "BEFORE" SP date / "BODY" SP astring /
///              "CC" SP astring / "DELETED" / "FLAGGED" /
///              "FROM" SP astring / "KEYWORD" SP flag-keyword /
///              "NEW" / "OLD" / "ON" SP date / "RECENT" / "SEEN" /
///              "SINCE" SP date / "SUBJECT" SP astring /
///              "TEXT" SP astring / "TO" SP astring /
///              "UNANSWERED" / "UNDELETED" / "UNFLAGGED" /
///              "UNKEYWORD" SP flag-keyword / "UNSEEN" /
///                ; Above this line were in [IMAP2]
///              "DRAFT" / "HEADER" SP header-fld-name SP astring /
///              "LARGER" SP number / "NOT" SP search-key /
///              "OR" SP search-key SP search-key /
///              "SENTBEFORE" SP date / "SENTON" SP date /
///              "SENTSINCE" SP date / "SMALLER" SP number /
///              "UID" SP sequence-set / "UNDRAFT" / sequence-set /
///              "(" search-key *(SP search-key) ")"
pub fn search_key(input: &[u8]) -> IResult<&[u8], SearchKey> {
    let parser = alt((
        alt((
            value(SearchKey::All, tag_no_case(b"ALL")),
            value(SearchKey::Answered, tag_no_case(b"ANSWERED")),
            map(tuple((tag_no_case(b"BCC"), sp, astring)), |(_, _, val)| {
                SearchKey::Bcc(val)
            }),
            map(
                tuple((tag_no_case(b"BEFORE"), sp, map_opt(date, |date| date))),
                |(_, _, date)| SearchKey::Before(date),
            ),
            map(tuple((tag_no_case(b"BODY"), sp, astring)), |(_, _, val)| {
                SearchKey::Body(val)
            }),
            map(tuple((tag_no_case(b"CC"), sp, astring)), |(_, _, val)| {
                SearchKey::Cc(val)
            }),
            value(SearchKey::Deleted, tag_no_case(b"DELETED")),
            value(SearchKey::Flagged, tag_no_case(b"FLAGGED")),
            map(tuple((tag_no_case(b"FROM"), sp, astring)), |(_, _, val)| {
                SearchKey::From(val)
            }),
            map(
                tuple((tag_no_case(b"KEYWORD"), sp, flag_keyword)),
                |(_, _, val)| SearchKey::Keyword(val),
            ),
            value(SearchKey::New, tag_no_case(b"NEW")),
            value(SearchKey::Old, tag_no_case(b"OLD")),
            map(
                tuple((tag_no_case(b"ON"), sp, map_opt(date, |date| date))),
                |(_, _, date)| SearchKey::On(date),
            ),
            value(SearchKey::Recent, tag_no_case(b"RECENT")),
            value(SearchKey::Seen, tag_no_case(b"SEEN")),
            map(
                tuple((tag_no_case(b"SINCE"), sp, map_opt(date, |date| date))),
                |(_, _, date)| SearchKey::Since(date),
            ),
            map(
                tuple((tag_no_case(b"SUBJECT"), sp, astring)),
                |(_, _, val)| SearchKey::Subject(val),
            ),
            map(tuple((tag_no_case(b"TEXT"), sp, astring)), |(_, _, val)| {
                SearchKey::Text(val)
            }),
            map(tuple((tag_no_case(b"TO"), sp, astring)), |(_, _, val)| {
                SearchKey::To(val)
            }),
        )),
        alt((
            value(SearchKey::Unanswered, tag_no_case(b"UNANSWERED")),
            value(SearchKey::Undeleted, tag_no_case(b"UNDELETED")),
            value(SearchKey::Unflagged, tag_no_case(b"UNFLAGGED")),
            map(
                tuple((tag_no_case(b"UNKEYWORD"), sp, flag_keyword)),
                |(_, _, val)| SearchKey::Unkeyword(val),
            ),
            value(SearchKey::Unseen, tag_no_case(b"UNSEEN")),
            value(SearchKey::Draft, tag_no_case(b"DRAFT")),
            map(
                tuple((tag_no_case(b"HEADER"), sp, header_fld_name, sp, astring)),
                |(_, _, key, _, val)| SearchKey::Header(key, val),
            ),
            map(
                tuple((tag_no_case(b"LARGER"), sp, number)),
                |(_, _, val)| SearchKey::Larger(val),
            ),
            map(
                tuple((tag_no_case(b"NOT"), sp, search_key)),
                |(_, _, val)| SearchKey::Not(Box::new(val)),
            ),
            map(
                tuple((tag_no_case(b"OR"), sp, search_key, sp, search_key)),
                |(_, _, alt1, _, alt2)| SearchKey::Or(Box::new(alt1), Box::new(alt2)),
            ),
            map(
                tuple((tag_no_case(b"SENTBEFORE"), sp, map_opt(date, |date| date))),
                |(_, _, date)| SearchKey::SentBefore(date),
            ),
            map(
                tuple((tag_no_case(b"SENTON"), sp, map_opt(date, |date| date))),
                |(_, _, date)| SearchKey::SentOn(date),
            ),
            map(
                tuple((tag_no_case(b"SENTSINCE"), sp, map_opt(date, |date| date))),
                |(_, _, date)| SearchKey::SentSince(date),
            ),
            map(
                tuple((tag_no_case(b"SMALLER"), sp, number)),
                |(_, _, val)| SearchKey::Smaller(val),
            ),
            map(
                tuple((tag_no_case(b"UID"), sp, sequence_set)),
                |(_, _, val)| SearchKey::Uid(val),
            ),
            value(SearchKey::Undraft, tag_no_case(b"UNDRAFT")),
            map(sequence_set, SearchKey::SequenceSet),
            map(
                delimited(
                    tag(b"("),
                    separated_nonempty_list(sp, search_key),
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

// TODO: abnf definition from IDLE extension
pub fn idle_done(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = value((), tuple((tag_no_case("DONE"), crlf)));

    let (remaining, parsed_idle_done) = parser(input)?;

    Ok((remaining, parsed_idle_done))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{SeqNo, Sequence};

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
        assert_eq!(val, CommandBody::Search(None, Uid(vec![Single(Value(5))])));

        let (_rem, val) = search(b"search (uid 5 or uid 5 (uid 1 uid 2) not (uid 5))???").unwrap();
        let expected = CommandBody::Search(
            None,
            And(vec![
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
        );
        assert_eq!(val, expected);
    }
}
