use abnf_core::streaming::sp;
/// Re-export everything from imap-types.
pub use imap_types::envelope::*;
use nom::{
    branch::alt,
    bytes::streaming::tag,
    combinator::map,
    multi::many1,
    sequence::{delimited, tuple},
};

use crate::{
    codec::IMAPResult,
    core::{nil, nstring, NString},
};

/// ```abnf
/// envelope = "("
///              env-date SP
///              env-subject SP
///              env-from SP
///              env-sender SP
///              env-reply-to SP
///              env-to SP
///              env-cc SP
///              env-bcc SP
///              env-in-reply-to SP
///              env-message-id
///            ")"
/// ```
pub fn envelope(input: &[u8]) -> IMAPResult<&[u8], Envelope> {
    let mut parser = delimited(
        tag(b"("),
        tuple((
            env_date,
            sp,
            env_subject,
            sp,
            env_from,
            sp,
            env_sender,
            sp,
            env_reply_to,
            sp,
            env_to,
            sp,
            env_cc,
            sp,
            env_bcc,
            sp,
            env_in_reply_to,
            sp,
            env_message_id,
        )),
        tag(b")"),
    );

    let (
        remaining,
        (
            date,
            _,
            subject,
            _,
            from,
            _,
            sender,
            _,
            reply_to,
            _,
            to,
            _,
            cc,
            _,
            bcc,
            _,
            in_reply_to,
            _,
            message_id,
        ),
    ) = parser(input)?;

    Ok((
        remaining,
        Envelope {
            date,
            subject,
            from,
            sender,
            reply_to,
            to,
            cc,
            bcc,
            in_reply_to,
            message_id,
        },
    ))
}

#[inline]
/// `env-date = nstring`
pub fn env_date(input: &[u8]) -> IMAPResult<&[u8], NString> {
    nstring(input)
}

#[inline]
/// `env-subject = nstring`
pub fn env_subject(input: &[u8]) -> IMAPResult<&[u8], NString> {
    nstring(input)
}

/// `env-from = "(" 1*address ")" / nil`
pub fn env_from(input: &[u8]) -> IMAPResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

/// `env-sender = "(" 1*address ")" / nil`
pub fn env_sender(input: &[u8]) -> IMAPResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

/// `env-reply-to = "(" 1*address ")" / nil`
pub fn env_reply_to(input: &[u8]) -> IMAPResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

/// `env-to = "(" 1*address ")" / nil`
pub fn env_to(input: &[u8]) -> IMAPResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

/// `env-cc = "(" 1*address ")" / nil`
pub fn env_cc(input: &[u8]) -> IMAPResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

/// `env-bcc = "(" 1*address ")" / nil`
pub fn env_bcc(input: &[u8]) -> IMAPResult<&[u8], Vec<Address>> {
    alt((
        delimited(tag(b"("), many1(address), tag(b")")),
        map(nil, |_| Vec::new()),
    ))(input)
}

#[inline]
/// `env-in-reply-to = nstring`
pub fn env_in_reply_to(input: &[u8]) -> IMAPResult<&[u8], NString> {
    nstring(input)
}

#[inline]
/// `env-message-id = nstring`
pub fn env_message_id(input: &[u8]) -> IMAPResult<&[u8], NString> {
    nstring(input)
}

/// `address = "("
///             addr-name SP
///             addr-adl SP
///             addr-mailbox SP
///             addr-host
///             ")"`
pub fn address(input: &[u8]) -> IMAPResult<&[u8], Address> {
    let mut parser = delimited(
        tag(b"("),
        tuple((addr_name, sp, addr_adl, sp, addr_mailbox, sp, addr_host)),
        tag(b")"),
    );

    let (remaining, (name, _, adl, _, mailbox, _, host)) = parser(input)?;

    Ok((
        remaining,
        Address {
            name,
            adl,
            mailbox,
            host,
        },
    ))
}

#[inline]
/// `addr-name = nstring`
///
/// If non-NIL, holds phrase from [RFC-2822]
/// mailbox after removing [RFC-2822] quoting
/// TODO(misuse): use `Phrase`?
pub fn addr_name(input: &[u8]) -> IMAPResult<&[u8], NString> {
    nstring(input)
}

#[inline]
/// `addr-adl = nstring`
///
/// Holds route from [RFC-2822] route-addr if non-NIL
/// TODO(misuse): use `Route`?
pub fn addr_adl(input: &[u8]) -> IMAPResult<&[u8], NString> {
    nstring(input)
}

#[inline]
/// `addr-mailbox = nstring`
///
/// NIL indicates end of [RFC-2822] group;
/// if non-NIL and addr-host is NIL, holds [RFC-2822] group name.
/// Otherwise, holds [RFC-2822] local-part after removing [RFC-2822] quoting
/// TODO(misuse): use `GroupName` or `LocalPart`?
pub fn addr_mailbox(input: &[u8]) -> IMAPResult<&[u8], NString> {
    nstring(input)
}

#[inline]
/// `addr-host = nstring`
///
/// NIL indicates [RFC-2822] group syntax.
/// Otherwise, holds [RFC-2822] domain name
/// TODO(misuse): use `DomainName`?
pub fn addr_host(input: &[u8]) -> IMAPResult<&[u8], NString> {
    nstring(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{IString, NString};

    #[test]
    fn test_parse_address() {
        let (rem, val) = address(b"(nil {3}\r\nxxx \"xxx\" nil)").unwrap();
        assert_eq!(
            val,
            Address {
                name: NString(None),
                adl: NString(Some(IString::Literal(
                    b"xxx".as_slice().try_into().unwrap()
                ))),
                mailbox: NString(Some(IString::Quoted("xxx".try_into().unwrap()))),
                host: NString(None),
            }
        );
        assert_eq!(rem, b"");
    }
}
