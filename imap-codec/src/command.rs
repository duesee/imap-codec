use std::borrow::Cow;

#[cfg(not(feature = "quirk_crlf_relaxed"))]
use abnf_core::streaming::crlf;
#[cfg(feature = "quirk_crlf_relaxed")]
use abnf_core::streaming::crlf_relaxed as crlf;
use abnf_core::streaming::sp;
#[cfg(feature = "ext_binary")]
use imap_types::extensions::binary::LiteralOrLiteral8;
use imap_types::{
    auth::AuthMechanism,
    command::{Command, CommandBody},
    core::AString,
    fetch::{Macro, MacroOrMessageDataItemNames},
    flag::{Flag, StoreResponse, StoreType},
    secret::Secret,
};
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    combinator::{map, opt, value},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, preceded, terminated, tuple},
};

#[cfg(feature = "ext_binary")]
use crate::extensions::binary::literal8;
#[cfg(feature = "ext_id")]
use crate::extensions::id::id;
#[cfg(feature = "ext_metadata")]
use crate::extensions::metadata::{getmetadata, setmetadata};
#[cfg(feature = "ext_uidplus")]
use crate::extensions::uidplus::uid_expunge;
#[cfg(feature = "ext_sort_thread")]
use crate::extensions::{sort::sort, thread::thread};
use crate::{
    auth::auth_type,
    core::{astring, base64, literal, tag_imap},
    datetime::date_time,
    decode::{IMAPErrorKind, IMAPResult},
    extensions::{
        compress::compress,
        enable::enable,
        idle::idle,
        quota::{getquota, getquotaroot, setquota},
        r#move::r#move,
    },
    fetch::fetch_att,
    flag::{flag, flag_list},
    mailbox::{list_mailbox, mailbox},
    search::search,
    sequence::sequence_set,
    status::status_att,
};

/// `command = tag SP (
///                     command-any /
///                     command-auth /
///                     command-nonauth /
///                     command-select
///                   ) CRLF`
pub(crate) fn command(input: &[u8]) -> IMAPResult<&[u8], Command> {
    let mut parser_tag = terminated(tag_imap, sp);
    let mut parser_body = terminated(
        alt((command_any, command_auth, command_nonauth, command_select)),
        crlf,
    );

    let (remaining, obtained_tag) = parser_tag(input)?;

    match parser_body(remaining) {
        Ok((remaining, body)) => Ok((
            remaining,
            Command {
                tag: obtained_tag,
                body,
            },
        )),
        Err(mut error) => {
            // If we got an `IMAPErrorKind::Literal`, we fill in the missing `tag`.
            if let nom::Err::Error(ref mut err) | nom::Err::Failure(ref mut err) = error {
                if let IMAPErrorKind::Literal { ref mut tag, .. } = err.kind {
                    *tag = Some(obtained_tag);
                }
            }

            Err(error)
        }
    }
}

// # Command Any

/// ```abnf
/// command-any = "CAPABILITY" /
///               "LOGOUT" /
///               "NOOP" /
///               x-command /
///               id ; adds id command to command_any (See RFC 2971)
/// ```
///
/// Note: Valid in all states
pub(crate) fn command_any(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    alt((
        value(CommandBody::Capability, tag_no_case(b"CAPABILITY")),
        value(CommandBody::Logout, tag_no_case(b"LOGOUT")),
        value(CommandBody::Noop, tag_no_case(b"NOOP")),
        // x-command = "X" atom <experimental command arguments>
        #[cfg(feature = "ext_id")]
        map(id, |parameters| CommandBody::Id { parameters }),
    ))(input)
}

// # Command Auth

/// ```abnf
/// command-auth = append /
///                create /
///                delete /
///                examine /
///                list /
///                lsub /
///                rename /
///                select /
///                status /
///                subscribe /
///                unsubscribe /
///                idle /         ; RFC 2177
///                enable /       ; RFC 5161
///                compress /     ; RFC 4978
///                getquota /     ; RFC 9208
///                getquotaroot / ; RFC 9208
///                setquota /     ; RFC 9208
///                setmetadata /  ; RFC 5464
///                getmetadata    ; RFC 5464
/// ```
///
/// Note: Valid only in Authenticated or Selected state
pub(crate) fn command_auth(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
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
        idle,
        enable,
        compress,
        getquota,
        getquotaroot,
        setquota,
        #[cfg(feature = "ext_metadata")]
        setmetadata,
        #[cfg(feature = "ext_metadata")]
        getmetadata,
    ))(input)
}

/// `append = "APPEND" SP mailbox [SP flag-list] [SP date-time] SP literal`
pub(crate) fn append(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let mut parser = tuple((
        tag_no_case(b"APPEND "),
        mailbox,
        opt(preceded(sp, flag_list)),
        opt(preceded(sp, date_time)),
        sp,
        #[cfg(not(feature = "ext_binary"))]
        literal,
        #[cfg(feature = "ext_binary")]
        alt((
            map(literal, LiteralOrLiteral8::Literal),
            map(literal8, LiteralOrLiteral8::Literal8),
        )),
    ));

    let (remaining, (_, mailbox, flags, date, _, message)) = parser(input)?;

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
pub(crate) fn create(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let mut parser = preceded(tag_no_case(b"CREATE "), mailbox);

    let (remaining, mailbox) = parser(input)?;

    Ok((remaining, CommandBody::Create { mailbox }))
}

/// `delete = "DELETE" SP mailbox`
///
/// Note: Use of INBOX gives a NO error
pub(crate) fn delete(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let mut parser = preceded(tag_no_case(b"DELETE "), mailbox);

    let (remaining, mailbox) = parser(input)?;

    Ok((remaining, CommandBody::Delete { mailbox }))
}

/// `examine = "EXAMINE" SP mailbox`
pub(crate) fn examine(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let mut parser = preceded(tag_no_case(b"EXAMINE "), mailbox);

    let (remaining, mailbox) = parser(input)?;

    Ok((remaining, CommandBody::Examine { mailbox }))
}

/// `list = "LIST" SP mailbox SP list-mailbox`
pub(crate) fn list(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"LIST "), mailbox, sp, list_mailbox));

    let (remaining, (_, reference, _, mailbox_wildcard)) = parser(input)?;

    Ok((
        remaining,
        CommandBody::List {
            reference,
            mailbox_wildcard,
        },
    ))
}

/// `lsub = "LSUB" SP mailbox SP list-mailbox`
pub(crate) fn lsub(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"LSUB "), mailbox, sp, list_mailbox));

    let (remaining, (_, reference, _, mailbox_wildcard)) = parser(input)?;

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
pub(crate) fn rename(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"RENAME "), mailbox, sp, mailbox));

    let (remaining, (_, mailbox, _, new_mailbox)) = parser(input)?;

    Ok((
        remaining,
        CommandBody::Rename {
            from: mailbox,
            to: new_mailbox,
        },
    ))
}

/// `select = "SELECT" SP mailbox`
pub(crate) fn select(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let mut parser = preceded(tag_no_case(b"SELECT "), mailbox);

    let (remaining, mailbox) = parser(input)?;

    Ok((remaining, CommandBody::Select { mailbox }))
}

/// `status = "STATUS" SP mailbox SP "(" status-att *(SP status-att) ")"`
pub(crate) fn status(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let mut parser = tuple((
        tag_no_case(b"STATUS "),
        mailbox,
        delimited(tag(b" ("), separated_list0(sp, status_att), tag(b")")),
    ));

    let (remaining, (_, mailbox, item_names)) = parser(input)?;

    Ok((
        remaining,
        CommandBody::Status {
            mailbox,
            item_names: item_names.into(),
        },
    ))
}

/// `subscribe = "SUBSCRIBE" SP mailbox`
pub(crate) fn subscribe(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let mut parser = preceded(tag_no_case(b"SUBSCRIBE "), mailbox);

    let (remaining, mailbox) = parser(input)?;

    Ok((remaining, CommandBody::Subscribe { mailbox }))
}

/// `unsubscribe = "UNSUBSCRIBE" SP mailbox`
pub(crate) fn unsubscribe(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let mut parser = preceded(tag_no_case(b"UNSUBSCRIBE "), mailbox);

    let (remaining, mailbox) = parser(input)?;

    Ok((remaining, CommandBody::Unsubscribe { mailbox }))
}

// # Command NonAuth

/// `command-nonauth = login / authenticate / "STARTTLS"`
///
/// Note: Valid only when in Not Authenticated state
pub(crate) fn command_nonauth(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let mut parser = alt((
        login,
        map(authenticate, |(mechanism, initial_response)| {
            CommandBody::Authenticate {
                mechanism,
                initial_response,
            }
        }),
        #[cfg(feature = "starttls")]
        value(CommandBody::StartTLS, tag_no_case(b"STARTTLS")),
    ));

    let (remaining, parsed_command_nonauth) = parser(input)?;

    Ok((remaining, parsed_command_nonauth))
}

/// `login = "LOGIN" SP userid SP password`
pub(crate) fn login(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"LOGIN"), sp, userid, sp, password));

    let (remaining, (_, _, username, _, password)) = parser(input)?;

    Ok((
        remaining,
        CommandBody::Login {
            username,
            password: Secret::new(password),
        },
    ))
}

#[inline]
/// `userid = astring`
pub(crate) fn userid(input: &[u8]) -> IMAPResult<&[u8], AString> {
    astring(input)
}

#[inline]
/// `password = astring`
pub(crate) fn password(input: &[u8]) -> IMAPResult<&[u8], AString> {
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
#[allow(clippy::type_complexity)]
pub(crate) fn authenticate(
    input: &[u8],
) -> IMAPResult<&[u8], (AuthMechanism, Option<Secret<Cow<[u8]>>>)> {
    let mut parser = tuple((
        tag_no_case(b"AUTHENTICATE "),
        auth_type,
        opt(preceded(
            sp,
            alt((
                map(base64, |data| Secret::new(Cow::Owned(data))),
                value(Secret::new(Cow::Borrowed(&b""[..])), tag("=")),
            )),
        )),
    ));

    let (remaining, (_, auth_type, raw_data)) = parser(input)?;

    // Server must send continuation ("+ ") at this point...

    Ok((remaining, (auth_type, raw_data)))
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
pub(crate) fn command_select(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    alt((
        value(CommandBody::Check, tag_no_case(b"CHECK")),
        value(CommandBody::Close, tag_no_case(b"CLOSE")),
        value(CommandBody::Expunge, tag_no_case(b"EXPUNGE")),
        #[cfg(feature = "ext_uidplus")]
        uid_expunge,
        copy,
        fetch,
        store,
        uid,
        search,
        #[cfg(feature = "ext_sort_thread")]
        sort,
        #[cfg(feature = "ext_sort_thread")]
        thread,
        value(CommandBody::Unselect, tag_no_case(b"UNSELECT")),
        r#move,
    ))(input)
}

/// `copy = "COPY" SP sequence-set SP mailbox`
pub(crate) fn copy(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"COPY"), sp, sequence_set, sp, mailbox));

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
pub(crate) fn fetch(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let mut parser = tuple((
        tag_no_case(b"FETCH"),
        sp,
        sequence_set,
        sp,
        alt((
            value(
                MacroOrMessageDataItemNames::Macro(Macro::All),
                tag_no_case(b"ALL"),
            ),
            value(
                MacroOrMessageDataItemNames::Macro(Macro::Fast),
                tag_no_case(b"FAST"),
            ),
            value(
                MacroOrMessageDataItemNames::Macro(Macro::Full),
                tag_no_case(b"FULL"),
            ),
            map(fetch_att, |fetch_att| {
                MacroOrMessageDataItemNames::MessageDataItemNames(vec![fetch_att])
            }),
            map(
                delimited(tag(b"("), separated_list0(sp, fetch_att), tag(b")")),
                MacroOrMessageDataItemNames::MessageDataItemNames,
            ),
        )),
    ));

    let (remaining, (_, _, sequence_set, _, macro_or_item_names)) = parser(input)?;

    Ok((
        remaining,
        CommandBody::Fetch {
            sequence_set,
            macro_or_item_names,
            uid: false,
        },
    ))
}

/// `store = "STORE" SP sequence-set SP store-att-flags`
pub(crate) fn store(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"STORE"), sp, sequence_set, sp, store_att_flags));

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
pub(crate) fn store_att_flags(
    input: &[u8],
) -> IMAPResult<&[u8], (StoreType, StoreResponse, Vec<Flag>)> {
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
        sp,
        alt((flag_list, separated_list1(sp, flag))),
    ));

    let (remaining, ((store_type, _, store_response), _, flag_list)) = parser(input)?;

    Ok((remaining, (store_type, store_response, flag_list)))
}

/// `uid = "UID" SP (copy / fetch / search / store)`
///
/// Note: Unique identifiers used instead of message sequence numbers
pub(crate) fn uid(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let mut parser = tuple((
        tag_no_case(b"UID"),
        sp,
        alt((copy, fetch, search, store, r#move)),
    ));

    let (remaining, (_, _, mut cmd)) = parser(input)?;

    match cmd {
        CommandBody::Copy { ref mut uid, .. }
        | CommandBody::Fetch { ref mut uid, .. }
        | CommandBody::Search { ref mut uid, .. }
        | CommandBody::Store { ref mut uid, .. }
        | CommandBody::Move { ref mut uid, .. } => *uid = true,
        _ => unreachable!(),
    }

    Ok((remaining, cmd))
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use imap_types::{
        core::Tag,
        fetch::{MessageDataItemName, Section},
    };

    use super::*;
    use crate::{encode::Encoder, CommandCodec};

    #[test]
    fn test_parse_fetch() {
        println!("{:#?}", fetch(b"fetch 1:1 (flags)???"));
    }

    #[test]
    fn test_parse_fetch_att() {
        let tests = [
            (MessageDataItemName::Envelope, "ENVELOPE???"),
            (MessageDataItemName::Flags, "FLAGS???"),
            (MessageDataItemName::InternalDate, "INTERNALDATE???"),
            (MessageDataItemName::Rfc822, "RFC822???"),
            (MessageDataItemName::Rfc822Header, "RFC822.HEADER???"),
            (MessageDataItemName::Rfc822Size, "RFC822.SIZE???"),
            (MessageDataItemName::Rfc822Text, "RFC822.TEXT???"),
            (MessageDataItemName::Body, "BODY???"),
            (MessageDataItemName::BodyStructure, "BODYSTRUCTURE???"),
            (MessageDataItemName::Uid, "UID???"),
            (
                MessageDataItemName::BodyExt {
                    partial: None,
                    peek: false,
                    section: None,
                },
                "BODY[]???",
            ),
            (
                MessageDataItemName::BodyExt {
                    partial: None,
                    peek: true,
                    section: None,
                },
                "BODY.PEEK[]???",
            ),
            (
                MessageDataItemName::BodyExt {
                    partial: None,
                    peek: true,
                    section: Some(Section::Text(None)),
                },
                "BODY.PEEK[TEXT]???",
            ),
            (
                MessageDataItemName::BodyExt {
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
    fn test_that_empty_ir_is_encoded_correctly() {
        let command = Command::new(
            Tag::try_from("A").unwrap(),
            CommandBody::Authenticate {
                mechanism: AuthMechanism::Plain,
                initial_response: Some(Secret::new(Cow::Borrowed(&b""[..]))),
            },
        )
        .unwrap();

        let buffer = CommandCodec::default().encode(&command).dump();

        assert_eq!(buffer, b"A AUTHENTICATE PLAIN =\r\n")
    }
}
