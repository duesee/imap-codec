use crate::{
    parse::{
        core::{
            astring, is_atom_char, is_resp_specials, nil, number, nz_number, quoted_char, string,
        },
        flag::{flag_list, mbx_list_flags},
        status::status_att_list,
    },
    types::{
        core::{astr, istr},
        mailbox::{ListMailbox, Mailbox},
        response::Data,
    },
};
use abnf_core::streaming::{DQUOTE, SP};
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case, take_while1},
    combinator::{map, opt, value},
    multi::many0,
    sequence::{delimited, preceded, tuple},
    IResult,
};

/// list-mailbox= 1*list-char / string
pub fn list_mailbox(input: &[u8]) -> IResult<&[u8], ListMailbox> {
    let parser = alt((
        map(take_while1(is_list_char), |bytes: &[u8]| {
            ListMailbox::Token(String::from_utf8(bytes.to_vec()).unwrap())
        }),
        map(string, |istr| ListMailbox::String(istr.to_owned())),
    ));

    let (remaining, parsed_list_mailbox) = parser(input)?;

    Ok((remaining, parsed_list_mailbox))
}

/// list-char = ATOM-CHAR / list-wildcards / resp-specials
pub fn is_list_char(i: u8) -> bool {
    is_atom_char(i) || is_list_wildcards(i) || is_resp_specials(i)
}

/// list-wildcards = "%" / "*"
pub fn is_list_wildcards(i: u8) -> bool {
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
    let (remaining, mailbox) = astring(input)?;

    let mailbox = match mailbox {
        astr::Atom(str) => {
            if str.to_lowercase() == "inbox" {
                Mailbox::Inbox
            } else {
                Mailbox::Other(mailbox.to_owned())
            }
        }
        astr::String(ref imap_str) => match imap_str {
            istr::Quoted(ref str) => {
                if str.to_lowercase() == "inbox" {
                    Mailbox::Inbox
                } else {
                    Mailbox::Other(mailbox.to_owned())
                }
            }
            istr::Literal(bytes) => {
                // "INBOX" (in any case) is certainly valid ASCII/UTF-8...
                if let Ok(str) = std::str::from_utf8(bytes) {
                    // After the conversion we ignore the case...
                    if str.to_lowercase() == "inbox" {
                        // ...and return the Inbox variant.
                        Mailbox::Inbox
                    } else {
                        Mailbox::Other(mailbox.to_owned())
                    }
                } else {
                    // ... If not, it must be something else.
                    Mailbox::Other(mailbox.to_owned())
                }
            }
        },
    };

    Ok((remaining, mailbox))
}

/// mailbox-data = "FLAGS" SP flag-list /
///                "LIST" SP mailbox-list /
///                "LSUB" SP mailbox-list /
///                "SEARCH" *(SP nz-number) /
///                "STATUS" SP mailbox SP "(" [status-att-list] ")" /
///                number SP "EXISTS" /
///                number SP "RECENT"
pub fn mailbox_data(input: &[u8]) -> IResult<&[u8], Data> {
    alt((
        map(
            tuple((tag_no_case(b"FLAGS"), SP, flag_list)),
            |(_, _, flags)| Data::Flags(flags),
        ),
        map(
            tuple((tag_no_case(b"LIST"), SP, mailbox_list)),
            |(_, _, data)| data,
        ),
        map(
            tuple((tag_no_case(b"LSUB"), SP, mailbox_list)),
            |(_, _, data)| data,
        ),
        map(
            tuple((tag_no_case(b"SEARCH"), many0(preceded(SP, nz_number)))),
            |(_, nums)| Data::Search(nums),
        ),
        map(
            tuple((
                tag_no_case(b"STATUS"),
                SP,
                mailbox,
                SP,
                delimited(tag(b"("), opt(status_att_list), tag(b")")),
            )),
            |(_, _, name, _, items)| Data::Status {
                name,
                items: items.unwrap_or_default(),
            },
        ),
        map(
            tuple((number, SP, tag_no_case(b"EXISTS"))),
            |(num, _, _)| Data::Exists(num),
        ),
        map(
            tuple((number, SP, tag_no_case(b"RECENT"))),
            |(num, _, _)| Data::Recent(num),
        ),
    ))(input)
}

/// mailbox-list = "(" [mbx-list-flags] ")" SP (DQUOTE QUOTED-CHAR DQUOTE / nil) SP mailbox
pub fn mailbox_list(input: &[u8]) -> IResult<&[u8], Data> {
    let parser = tuple((
        delimited(tag(b"("), opt(mbx_list_flags), tag(b")")),
        SP,
        alt((
            map(delimited(DQUOTE, quoted_char, DQUOTE), Option::Some),
            value(None, nil),
        )),
        SP,
        mailbox,
    ));

    let (remaining, (mbx_list_flags, _, maybe_delimiter, _, mailbox)) = parser(input)?;

    Ok((
        remaining,
        Data::List {
            items: mbx_list_flags.unwrap_or_default(),
            delimiter: maybe_delimiter,
            mailbox,
        },
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mailbox() {
        assert!(mailbox(b"\"iNbOx\"").is_ok());
        assert!(mailbox(b"{3}\r\naaa\r\n").is_ok());
        assert!(mailbox(b"inbox ").is_ok());
        assert!(mailbox(b"inbox.sent ").is_ok());
        assert!(mailbox(b"aaa").is_err());
    }
}
