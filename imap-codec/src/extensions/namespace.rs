//! The IMAP NAMESPACE Extension

use imap_types::{core::Quoted, extensions::namespace::NamespaceDescription, response::Data};
use nom::{
    branch::alt,
    bytes::{complete::tag_no_case, streaming::tag},
    combinator::{map, value},
    multi::many0,
    sequence::{delimited, preceded, tuple},
};
use std::io::Write;

use crate::{
    core::{astring, quoted_char},
    decode::IMAPResult,
    encode::{EncodeContext, EncodeIntoContext},
};

/// ```abnf
/// namespace-response = "NAMESPACE" SP namespace-list SP namespace-list SP namespace-list
/// ```
pub(crate) fn namespace_response(input: &[u8]) -> IMAPResult<&[u8], Data> {
    let mut parser = tuple((
        tag_no_case("NAMESPACE "),
        namespace_list,
        preceded(tag(" "), namespace_list),
        preceded(tag(" "), namespace_list),
    ));

    let (remaining, (_, personal, other, shared)) = parser(input)?;

    Ok((
        remaining,
        Data::Namespace {
            personal,
            other,
            shared,
        },
    ))
}

/// ```abnf
/// namespace-list = "(" *(namespace-descriptor) ")" / nil
///
/// namespace-descriptor = DQUOTE <namespace-prefix: string> DQUOTE SP <delimiter: nstring>
/// ```
fn namespace_list(input: &[u8]) -> IMAPResult<&[u8], Vec<NamespaceDescription>> {
    alt((
        delimited(tag("("), many0(namespace_descriptor), tag(")")),
        map(tag_no_case("NIL"), |_| Vec::new()),
    ))(input)
}

fn namespace_descriptor(input: &[u8]) -> IMAPResult<&[u8], NamespaceDescription> {
    let delimiter_parser = alt((map(quoted_char, Some), value(None, tag_no_case(b"NIL"))));

    map(
        tuple((astring, tag(b" "), delimiter_parser)),
        |(prefix, _, delimiter)| NamespaceDescription { prefix, delimiter },
    )(input)
}

impl EncodeIntoContext for NamespaceDescription<'_> {
    fn encode_ctx(&self, ctx: &mut EncodeContext) -> std::io::Result<()> {
        write!(ctx, "(")?;
        self.prefix.encode_ctx(ctx)?;
        write!(ctx, " ")?;
        match &self.delimiter {
            Some(delimiter_char) => {
                let as_string = String::from(delimiter_char.inner());
                let quoted = Quoted::try_from(as_string)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                quoted.encode_ctx(ctx)?;
            }
            None => {
                ctx.write_all(b"NIL")?;
            }
        }
        write!(ctx, ")")
    }
}

pub fn encode_namespace_list(
    ctx: &mut EncodeContext,
    list: &Vec<NamespaceDescription<'_>>,
) -> std::io::Result<()> {
    if list.is_empty() {
        ctx.write_all(b"NIL")
    } else {
        ctx.write_all(b"(")?;
        for desc in list {
            desc.encode_ctx(ctx)?;
        }
        ctx.write_all(b")")
    }
}
