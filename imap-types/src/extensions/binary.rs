//! IMAP4 Binary Content Extension

use std::{
    borrow::Cow,
    fmt::{Debug, Formatter},
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::core::LiteralMode;

/// String that might contain NULs.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Literal8<'a> {
    pub data: Cow<'a, [u8]>,
    /// Specifies whether this is a synchronizing or non-synchronizing literal.
    ///
    /// `true` (default) denotes a synchronizing literal, e.g., `~{3}\r\nfoo`.
    /// `false` denotes a non-synchronizing literal, e.g., `~{3+}\r\nfoo`.
    ///
    /// Note: In the special case that a server advertised a `LITERAL-` capability, AND the literal
    /// has more than 4096 bytes a non-synchronizing literal must still be treated as synchronizing.
    pub mode: LiteralMode,
}

// We want a more readable `Debug` implementation.
impl<'a> Debug for Literal8<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        struct BStr<'a>(&'a Cow<'a, [u8]>);

        impl<'a> Debug for BStr<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "b\"{}\"",
                    crate::utils::escape_byte_string(self.0.as_ref())
                )
            }
        }

        f.debug_struct("Literal8")
            .field("data", &BStr(&self.data))
            .field("mode", &self.mode)
            .finish()
    }
}
