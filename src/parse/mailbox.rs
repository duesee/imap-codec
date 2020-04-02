use crate::{
    parse::{
        core::{
            astring, is_atom_char, is_resp_specials, nil, number, nz_number, quoted_char, string,
        },
        dquote,
        flag::{flag_extension, flag_list},
        sp,
        status::status_att_list,
    },
    types::{
        core::{AString, Atom, String as IMAPString},
        mailbox::{Mailbox, MailboxWithWildcards},
    },
};
use nom::{
    branch::alt,
    bytes::streaming::{tag_no_case, take_while1},
    combinator::{map, opt, value},
    multi::many0,
    sequence::tuple,
    IResult,
};

/// list-mailbox= 1*list-char / string
pub fn list_mailbox(input: &[u8]) -> IResult<&[u8], MailboxWithWildcards> {
    let parser = alt((
        map(take_while1(is_list_char), |bytes: &[u8]| {
            MailboxWithWildcards::V1(String::from_utf8(bytes.to_vec()).unwrap())
        }),
        map(string, MailboxWithWildcards::V2),
    ));

    let (remaining, parsed_list_mailbox) = parser(input)?;

    Ok((remaining, parsed_list_mailbox))
}

/// list-char = ATOM-CHAR / list-wildcards / resp-specials
fn is_list_char(i: u8) -> bool {
    is_atom_char(i) || is_list_wildcards(i) || is_resp_specials(i)
}

/// list-wildcards = "%" / "*"
fn is_list_wildcards(i: u8) -> bool {
    i == b'%' || i == b'*'
}

/// mailbox = "INBOX" / astring
///             ; INBOX is case-insensitive.  All case variants of
///             ; INBOX (e.g., "iNbOx") MUST be interpreted as INBOX
///             ; not as an astring.  An astring which consists of
///             ; the case-insensitive sequence "I" "N" "B" "O" "X"
///             ; is considered to be INBOX and not an astring.
///             ;  Refer to section 5.1 for further
///             ; semantic details of mailbox names.
/// FIXME: this is only to keep in mind that there are several string types
pub fn mailbox(input: &[u8]) -> IResult<&[u8], Mailbox> {
    let parser = alt((
        value(Mailbox::Inbox, tag_no_case(b"INBOX")),
        map(astring, |a_str| match a_str {
            AString::Atom(Atom(str)) => {
                if str.to_lowercase() == "inbox" {
                    Mailbox::Inbox
                } else {
                    Mailbox::Other(AString::Atom(Atom(str)))
                }
            }
            AString::String(imap_str) => match imap_str {
                IMAPString::Quoted(ref str) => {
                    if str.to_lowercase() == "inbox" {
                        Mailbox::Inbox
                    } else {
                        Mailbox::Other(AString::String(imap_str))
                    }
                }
                IMAPString::Literal(ref bytes) => {
                    // "INBOX" (in any case) is certainly valid ASCII/UTF-8...
                    if let Ok(str) = String::from_utf8(bytes.clone()) {
                        // After the conversion we ignore the case...
                        if str.to_lowercase() == "inbox" {
                            // ...and return the Inbox variant.
                            return Mailbox::Inbox;
                        }
                    }

                    // ... If not, it must be something else.
                    Mailbox::Other(AString::String(imap_str))
                }
            },
        }),
    ));

    let (remaining, parsed_mailbox) = parser(input)?;

    Ok((remaining, parsed_mailbox))
}

/// mailbox-data = "FLAGS" SP flag-list /
///                "LIST" SP mailbox-list /
///                "LSUB" SP mailbox-list /
///                "SEARCH" *(SP nz-number) /
///                "STATUS" SP mailbox SP "(" [status-att-list] ")" /
///                number SP "EXISTS" /
///                number SP "RECENT"
pub fn mailbox_data(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = alt((
        map(
            tuple((tag_no_case(b"FLAGS"), sp, flag_list)),
            |_| unimplemented!(),
        ),
        map(
            tuple((tag_no_case(b"LIST"), sp, mailbox_list)),
            |_| unimplemented!(),
        ),
        map(
            tuple((tag_no_case(b"LSUB"), sp, mailbox_list)),
            |_| unimplemented!(),
        ),
        map(
            tuple((tag_no_case(b"SEARCH"), many0(tuple((sp, nz_number))))),
            |_| unimplemented!(),
        ),
        map(
            tuple((
                tag_no_case(b"STATUS"),
                sp,
                mailbox,
                sp,
                tag_no_case(b"("),
                opt(status_att_list),
                tag_no_case(b")"),
            )),
            |_| unimplemented!(),
        ),
        map(
            tuple((number, sp, tag_no_case(b"EXISTS"))),
            |_| unimplemented!(),
        ),
        map(
            tuple((number, sp, tag_no_case(b"RECENT"))),
            |_| unimplemented!(),
        ),
    ));

    let (_remaining, _parsed_mailbox_data) = parser(input)?;

    unimplemented!();
}

/// mailbox-list = "(" [mbx-list-flags] ")" SP (DQUOTE QUOTED-CHAR DQUOTE / nil) SP mailbox
pub fn mailbox_list(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = tuple((
        tag_no_case(b"("),
        opt(mbx_list_flags),
        tag_no_case(b")"),
        sp,
        alt((
            map(tuple((dquote, quoted_char, dquote)), |_| unimplemented!()),
            map(nil, |_| unimplemented!()),
        )),
        sp,
        mailbox,
    ));

    let (_remaining, _parsed_mailbox_list) = parser(input)?;

    unimplemented!();
}

/// mbx-list-flags = *(mbx-list-oflag SP) mbx-list-sflag *(SP mbx-list-oflag) / mbx-list-oflag *(SP mbx-list-oflag)
pub fn mbx_list_flags(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = alt((
        map(
            tuple((
                many0(tuple((mbx_list_oflag, sp))),
                mbx_list_sflag,
                many0(tuple((sp, mbx_list_oflag))),
            )),
            |_| unimplemented!(),
        ),
        map(
            tuple((mbx_list_oflag, many0(tuple((sp, mbx_list_oflag))))),
            |_| unimplemented!(),
        ),
    ));

    let (_remaining, _parsed_mbx_list_flags) = parser(input)?;

    unimplemented!();
}

/// mbx-list-oflag = "\Noinferiors" / flag-extension
///                    ; Other flags; multiple possible per LIST response
pub fn mbx_list_oflag(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = alt((
        map(tag_no_case(b"\\Noinferiors"), |_| unimplemented!()),
        map(flag_extension, |_| unimplemented!()),
    ));

    let (_remaining, _parsed_mbx_list_oflag) = parser(input)?;

    unimplemented!();
}

/// mbx-list-sflag = "\Noselect" / "\Marked" / "\Unmarked"
///                    ; Selectability flags; only one per LIST response
pub fn mbx_list_sflag(input: &[u8]) -> IResult<&[u8], ()> {
    let parser = alt((
        map(tag_no_case(b"\\Noselect"), |_| unimplemented!()),
        map(tag_no_case(b"\\Marked"), |_| unimplemented!()),
        map(tag_no_case(b"\\Unmarked"), |_| unimplemented!()),
    ));

    let (_remaining, _parsed_mbx_list_sflag) = parser(input)?;

    unimplemented!();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mailbox() {
        assert!(mailbox(b"\"iNbOx\"").is_ok());
        assert!(mailbox(b"{3}\r\naaa\r\n").is_ok());
        assert!(mailbox(b"inbox").is_ok());
        assert!(mailbox(b"aaa").is_err());
    }
}
