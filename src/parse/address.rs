use crate::{
    parse::{core::nstring, sp},
    types::{core::NString, response::Address},
};
use nom::{
    bytes::streaming::tag,
    sequence::{delimited, tuple},
    IResult,
};

/// address = "(" addr-name SP addr-adl SP addr-mailbox SP addr-host ")"
pub fn address(input: &[u8]) -> IResult<&[u8], Address> {
    let parser = delimited(
        tag(b"("),
        tuple((addr_name, sp, addr_adl, sp, addr_mailbox, sp, addr_host)),
        tag(b")"),
    );

    let (remaining, (name, _, adl, _, mailbox, _, host)) = parser(input)?;

    Ok((remaining, Address::new(name, adl, mailbox, host)))
}

/// addr-name = nstring
///               ; If non-NIL, holds phrase from [RFC-2822]
///               ; mailbox after removing [RFC-2822] quoting
pub fn addr_name(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

/// addr-adl = nstring
///              ; Holds route from [RFC-2822] route-addr if
///              ; non-NIL
pub fn addr_adl(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

/// addr-mailbox = nstring
///                  ; NIL indicates end of [RFC-2822] group; if
///                  ; non-NIL and addr-host is NIL, holds
///                  ; [RFC-2822] group name.
///                  ; Otherwise, holds [RFC-2822] local-part
///                  ; after removing [RFC-2822] quoting
pub fn addr_mailbox(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

/// addr-host = nstring
///               ; NIL indicates [RFC-2822] group syntax.
///               ; Otherwise, holds [RFC-2822] domain name
pub fn addr_host(input: &[u8]) -> IResult<&[u8], NString> {
    nstring(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::core::String as IMAPString;

    #[test]
    fn test_address() {
        let (rem, val) = address(b"(nil {3}\r\nxxx \"xxx\" nil)").unwrap();
        assert_eq!(
            val,
            Address::new(
                NString::Nil,
                NString::String(IMAPString::Literal(b"xxx".to_vec())),
                NString::String(IMAPString::Quoted(String::from("xxx"))),
                NString::Nil
            )
        );
        assert_eq!(rem, b"");
    }
}
