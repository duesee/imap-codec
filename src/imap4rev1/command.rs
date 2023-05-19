#[cfg(feature = "ext_sasl_ir")]
use std::borrow::Cow;

use abnf_core::streaming::{CRLF, SP};
use imap_types::{
    command::{
        fetch::{Macro, MacroOrFetchAttributes},
        store::{StoreResponse, StoreType},
        AuthenticateData, Command, CommandBody,
    },
    core::AString,
    message::{AuthMechanism, Flag},
    security::Secret,
};
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    combinator::{map, opt, value},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

#[cfg(feature = "ext_compress")]
use crate::extensions::compress::compress;
#[cfg(feature = "ext_enable")]
use crate::extensions::enable::enable;
#[cfg(feature = "ext_idle")]
use crate::extensions::idle::idle;
#[cfg(feature = "ext_quota")]
use crate::extensions::quota::{getquota, getquotaroot, setquota};
use crate::imap4rev1::{
    auth_type,
    command::search::search,
    core::{astring, base64, literal, tag_imap},
    datetime::date_time,
    fetch_attributes::fetch_att,
    flag::{flag, flag_list},
    mailbox::{list_mailbox, mailbox},
    sequence::sequence_set,
    status_attributes::status_att,
};

pub mod search;

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

    let (remaining, (tag, _, body, _)) = parser(input)?;

    Ok((remaining, Command { tag, body }))
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
        enable,
        #[cfg(feature = "ext_compress")]
        compress,
        #[cfg(feature = "ext_quota")]
        getquota,
        #[cfg(feature = "ext_quota")]
        getquotaroot,
        #[cfg(feature = "ext_quota")]
        setquota,
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
            from: mailbox,
            to: new_mailbox,
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
        #[cfg(not(feature = "ext_sasl_ir"))]
        map(authenticate, |mechanism| CommandBody::Authenticate {
            mechanism,
        }),
        #[cfg(feature = "ext_sasl_ir")]
        map(authenticate_sasl_ir, |(mechanism, initial_response)| {
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
pub fn login(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case(b"LOGIN"), SP, userid, SP, password));

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
/// authenticate = "AUTHENTICATE" SP auth-type *(CRLF base64)
///                ^^^^^^^^^^^^^^^^^^^^^^^^^^^
///                |
///                This is parsed here.
///                CRLF is parsed by upper command parser.
/// ```
#[cfg(not(feature = "ext_sasl_ir"))]
pub fn authenticate(input: &[u8]) -> IResult<&[u8], AuthMechanism> {
    let mut parser = preceded(tag_no_case(b"AUTHENTICATE "), auth_type);

    let (remaining, auth_type) = parser(input)?;

    // Server must send continuation ("+ ") at this point...

    Ok((remaining, auth_type))
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
#[cfg(feature = "ext_sasl_ir")]
#[allow(clippy::type_complexity)]
pub fn authenticate_sasl_ir(
    input: &[u8],
) -> IResult<&[u8], (AuthMechanism, Option<Secret<Cow<[u8]>>>)> {
    let mut parser = tuple((
        tag_no_case(b"AUTHENTICATE "),
        auth_type,
        opt(preceded(
            SP,
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
pub fn authenticate_data(input: &[u8]) -> IResult<&[u8], AuthenticateData> {
    map(terminated(base64, CRLF), |data| {
        AuthenticateData(Secret::new(data))
    })(input) // FIXME: many0 deleted
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
        #[cfg(feature = "ext_unselect")]
        value(CommandBody::Unselect, tag_no_case(b"UNSELECT")),
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

#[cfg(test)]
mod tests {
    use std::{convert::TryFrom, num::NonZeroU32};

    use imap_types::{command::fetch::FetchAttribute, message::Section};

    use super::*;

    #[test]
    fn test_parse_fetch() {
        println!("{:#?}", fetch(b"fetch 1:1 (flags)???"));
    }

    #[test]
    fn test_parse_fetch_att() {
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
}
