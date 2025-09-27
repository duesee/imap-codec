//! The IMAP NAMESPACE Extension

use imap_types::{
    core::{AString, Quoted},
    extensions::namespace::{Namespace, NamespaceResponseExtension, Namespaces},
    response::Data,
};
use nom::{
    branch::alt,
    bytes::{complete::tag, complete::tag_no_case},
    combinator::{map, value},
    multi::{many0, many1},
    sequence::{delimited, preceded, tuple},
};
use std::io::Write;

use crate::{
    core::{astring, quoted, quoted_char, string},
    decode::IMAPResult,
    encode::{EncodeContext, EncodeIntoContext},
};

/// Parses the full NAMESPACE data response.
///
/// ``` abnf
/// Namespace_Response = "*"` SP `"NAMESPACE"` SP `Namespace` SP `Namespace` SP `Namespace`
/// ```
pub(crate) fn namespace_response(input: &[u8]) -> IMAPResult<&[u8], Data> {
    let mut parser = tuple((
        tag_no_case("NAMESPACE "),
        namespaces,
        preceded(tag(" "), namespaces),
        preceded(tag(" "), namespaces),
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

/// Parses a list of namespaces.
///
/// ```abnf
/// Namespace = nil / "(" 1*( "(" string SP  (<"> QUOTED_CHAR <"> / nil) *(Namespace_Response_Extension) ")" ) ")"
/// ```
fn namespaces(input: &[u8]) -> IMAPResult<&[u8], Namespaces> {
    alt((
        delimited(tag("("), many1(namespace), tag(")")),
        map(tag_no_case("NIL"), |_| Vec::new()),
    ))(input)
}

/// Parses a single namespace description.
fn namespace(input: &[u8]) -> IMAPResult<&[u8], Namespace> {
    let delimiter_parser = alt((
        map(delimited(tag("\""), quoted_char, tag("\"")), Some),
        value(None, tag_no_case("NIL")),
    ));

    map(
        delimited(
            tag("("),
            tuple((
                astring,
                tag(" "),
                delimiter_parser,
                many0(namespace_response_extension),
            )),
            tag(")"),
        ),
        |(prefix, _, delimiter, extensions)| Namespace {
            prefix,
            delimiter,
            extensions,
        },
    )(input)
}

/// Parses a namespace response extension.
///
/// ```abnf
/// Namespace_Response_Extension = SP string SP "(" string *(SP string) ")"
/// ```
fn namespace_response_extension(input: &[u8]) -> IMAPResult<&[u8], NamespaceResponseExtension> {
    map(
        preceded(
            tag(" "),
            tuple((
                astring,
                tag(" "),
                delimited(tag("("), many0(preceded(tag(" "), astring)), tag(")")),
            )),
        ),
        |(key, _, values)| NamespaceResponseExtension { key, values },
    )(input)
}

impl EncodeIntoContext for Namespace<'_> {
    fn encode_ctx(&self, ctx: &mut EncodeContext) -> std::io::Result<()> {
        write!(ctx, "(")?;
        self.prefix.encode_ctx(ctx)?;
        write!(ctx, " ")?;

        match &self.delimiter {
            Some(delimiter_char) => {
                write!(ctx, "\"{}\"", delimiter_char.inner())?;
            }
            None => {
                ctx.write_all(b"NIL")?;
            }
        }

        for ext in &self.extensions {
            ext.encode_ctx(ctx)?;
        }

        write!(ctx, ")")
    }
}

impl EncodeIntoContext for NamespaceResponseExtension<'_> {
    fn encode_ctx(&self, ctx: &mut EncodeContext) -> std::io::Result<()> {
        write!(ctx, " ")?;
        self.key.encode_ctx(ctx)?;
        write!(ctx, " (")?;
        for (i, value) in self.values.iter().enumerate() {
            if i > 0 {
                write!(ctx, " ")?;
            }
            value.encode_ctx(ctx)?;
        }
        write!(ctx, ")")
    }
}

pub fn encode_namespaces(ctx: &mut EncodeContext, list: &Namespaces<'_>) -> std::io::Result<()> {
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
