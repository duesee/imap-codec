#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    core::{AString, NString8, Vec1},
    error::ValidationError,
};

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, ToStatic)]
pub struct EntryValue<'a> {
    pub entry: Entry<'a>,
    pub value: NString8<'a>,
}

/// Slash-separated path to entry.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, ToStatic)]
pub struct Entry<'a>(AString<'a>);

impl<'a> Entry<'a> {
    pub fn inner(&self) -> &AString<'a> {
        &self.0
    }
}

impl<'a> TryFrom<AString<'a>> for Entry<'a> {
    type Error = ValidationError;

    fn try_from(value: AString<'a>) -> Result<Self, Self::Error> {
        // TODO(#449): Currently, we do no validation. Let's gather more
        //             practical experience before settling on a representation.
        Ok(Self(value))
    }
}

impl AsRef<[u8]> for Entry<'_> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, ToStatic)]
pub enum GetMetadataOption {
    /// Only return values that are less than or equal in octet size to the specified limit.
    ///
    /// If there are any entries with values  larger than the MAXSIZE limit, the server MUST include
    /// the METADATA LONGENTRIES response code in the tagged OK response for the GETMETADATA command.
    MaxSize(u32),
    /// Extends the list of entry values returned by the server.
    ///
    /// For each entry name specified in the GETMETADATA command, the server returns the value of the
    /// specified entry name (if it exists), plus all entries below the entry name up to the specified DEPTH.
    Depth(Depth),
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, ToStatic)]
pub enum Depth {
    /// No entries below the specified entry are returned
    Null,
    /// Only entries immediately below the specified entry are returned
    One,
    /// All entries below the specified entry are returned
    Infinity,
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, ToStatic)]
pub enum MetadataCode {
    LongEntries(u32),
    MaxSize(u32),
    TooMany,
    NoPrivate,
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, ToStatic)]
pub enum MetadataResponse<'a> {
    WithValues(Vec1<EntryValue<'a>>),
    WithoutValues(Vec1<Entry<'a>>),
}
