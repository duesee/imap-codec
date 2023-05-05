use abnf_core::streaming::SP;
use imap_types::{core::NString, response::data::Address};
use nom::{
    bytes::streaming::tag,
    sequence::{delimited, tuple},
    IResult,
};

use crate::imap4rev1::core::nstring;

/// `address = "("
///             addr-name SP
///             addr-adl SP
///             addr-mailbox SP
///             addr-host
///             ")"`
pub fn address(input: &[u8]) -> IResult<&[u8], Address> {
    let mut parser = delimited(
        tag(b"("),
        tuple((addr_name, SP, addr_adl, SP, addr_mailbox, SP, addr_host)),
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
pub fn addr_name(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

#[inline]
/// `addr-adl = nstring`
///
/// Holds route from [RFC-2822] route-addr if non-NIL
/// TODO(misuse): use `Route`?
pub fn addr_adl(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

#[inline]
/// `addr-mailbox = nstring`
///
/// NIL indicates end of [RFC-2822] group;
/// if non-NIL and addr-host is NIL, holds [RFC-2822] group name.
/// Otherwise, holds [RFC-2822] local-part after removing [RFC-2822] quoting
/// TODO(misuse): use `GroupName` or `LocalPart`?
pub fn addr_mailbox(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

#[inline]
/// `addr-host = nstring`
///
/// NIL indicates [RFC-2822] group syntax.
/// Otherwise, holds [RFC-2822] domain name
/// TODO(misuse): use `DomainName`?
pub fn addr_host(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use imap_types::core::{IString, NString};

    use super::*;

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
