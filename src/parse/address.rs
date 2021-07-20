use abnf_core::streaming::SP;
use nom::{
    bytes::streaming::tag,
    sequence::{delimited, tuple},
    IResult,
};

use crate::{
    parse::core::nstring,
    types::{address::Address, core::nstr},
};

/// address = "(" addr-name SP
///               addr-adl SP
///               addr-mailbox SP
///               addr-host ")"
pub(crate) fn address(input: &[u8]) -> IResult<&[u8], Address> {
    let mut parser = delimited(
        tag(b"("),
        tuple((addr_name, SP, addr_adl, SP, addr_mailbox, SP, addr_host)),
        tag(b")"),
    );

    let (remaining, (name, _, adl, _, mailbox, _, host)) = parser(input)?;

    Ok((
        remaining,
        Address::new(
            name.to_owned(),
            adl.to_owned(),
            mailbox.to_owned(),
            host.to_owned(),
        ),
    ))
}

#[inline]
/// addr-name = nstring
///
/// If non-NIL, holds phrase from [RFC-2822]
/// mailbox after removing [RFC-2822] quoting
fn addr_name(input: &[u8]) -> IResult<&[u8], nstr> {
    nstring(input)
}

#[inline]
/// addr-adl = nstring
///
/// Holds route from [RFC-2822] route-addr if non-NIL
fn addr_adl(input: &[u8]) -> IResult<&[u8], nstr> {
    nstring(input)
}

#[inline]
/// addr-mailbox = nstring
///
/// NIL indicates end of [RFC-2822] group;
/// if non-NIL and addr-host is NIL, holds [RFC-2822] group name.
/// Otherwise, holds [RFC-2822] local-part after removing [RFC-2822] quoting
fn addr_mailbox(input: &[u8]) -> IResult<&[u8], nstr> {
    nstring(input)
}

#[inline]
/// addr-host = nstring
///
/// NIL indicates [RFC-2822] group syntax.
/// Otherwise, holds [RFC-2822] domain name
fn addr_host(input: &[u8]) -> IResult<&[u8], nstr> {
    nstring(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::core::{IString, NString};

    #[test]
    fn test_address() {
        let (rem, val) = address(b"(nil {3}\r\nxxx \"xxx\" nil)").unwrap();
        assert_eq!(
            val,
            Address::new(
                NString(None),
                NString(Some(IString::Literal(b"xxx".to_vec()))),
                NString(Some(IString::Quoted(String::from("xxx")))),
                NString(None),
            )
        );
        assert_eq!(rem, b"");
    }
}
