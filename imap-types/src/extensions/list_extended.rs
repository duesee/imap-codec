//! IMAP LIST Command Extensions: response data items (RFC 5258).
//!
//! See [RFC 5258](https://www.rfc-editor.org/rfc/rfc5258).
//!
//! This module provides the extended data items (`CHILDINFO`, and generic
//! vendor/standard items) returned in `LIST` responses.

#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Unstructured};
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "arbitrary")]
use crate::arbitrary::impl_arbitrary_try_from;
use crate::{
    core::{AString, Atom, Vec1, Vec2},
    extensions::list_extended::error::{MboxListExtendedItemTagError, OptionExtensionTagError},
    sequence::SequenceSet,
};

/// A generic extended data item value component.
///
/// ```abnf
/// tagged-ext-comp = astring /
///                   tagged-ext-comp *(SP tagged-ext-comp) /
///                   "(" tagged-ext-comp ")"
/// ```
///
/// See [RFC 5258, section 6](https://www.rfc-editor.org/rfc/rfc5258#section-6).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum TaggedExtComp<'a> {
    /// An `astring`.
    ///
    /// Represents exactly 1 component.
    Single(AString<'a>),

    /// A space-separated sequence.
    ///
    /// Represents 2 components or more. The model diverges on purpose from the
    /// RFC, because specs are ambiguous: a single component can be represented
    /// by multiple variants. Exactly one component = Single,
    Multi(Vec2<TaggedExtComp<'a>>),

    /// A parenthesized group of components.
    Group(Box<TaggedExtComp<'a>>),
}

/// The value of a generic `mbox-list-extended-item` (`tagged-ext-val`).
///
/// ```abnf
/// tagged-ext-val = tagged-ext-simple / "(" [tagged-ext-comp] ")"
/// ```
///
/// See [RFC 5258, section 6](https://www.rfc-editor.org/rfc/rfc5258#section-6)
/// and [RFC 4466, section 2.1](https://www.rfc-editor.org/rfc/rfc4466#section-2.1).
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum TaggedExtVal<'a> {
    /// A `tagged-ext-simple` value (a `sequence-set` or `number`).
    Simple(TaggedExtSimple),

    /// A parenthesized `tagged-ext-comp` value (`"(" [tagged-ext-comp] ")"`).
    Comp(Option<TaggedExtComp<'a>>),
}

/// A simple value of a [`TaggedExtVal`] (`tagged-ext-simple`).
///
/// ```abnf
/// tagged-ext-simple = sequence-set / number
/// ```
///
/// Note: A `sequence-set` subsumes every non-zero `number`, and its values must
/// be non-zero. So a bare non-zero number is represented as a single-element
/// [`SequenceSet`](Self::SequenceSet), and the [`Number`](Self::Number) variant
/// is only needed for values a `sequence-set` can't hold (notably `0`).
///
/// See [RFC 4466, section 2.1](https://www.rfc-editor.org/rfc/rfc4466#section-2.1).
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum TaggedExtSimple {
    /// A `sequence-set`.
    SequenceSet(SequenceSet),

    /// A `number`.
    Number(u32),
}

/// A base selection option.
///
/// Used (quoted, as `list-select-base-opt-quoted`) inside a `CHILDINFO`
/// extended data item.
///
/// ```abnf
/// list-select-base-opt = "SUBSCRIBED" / option-extension
/// ```
///
/// See [RFC 5258, section 3](https://www.rfc-editor.org/rfc/rfc5258#section-3).
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum ListSelectBaseOpt<'a> {
    /// `SUBSCRIBED`
    ///
    /// At least one subscribed submailbox is present below the returned mailbox
    /// (when reported via `CHILDINFO`).
    Subscribed,

    /// An `option-extension` (a standard or vendor-specific base option).
    Extension(ListSelectBaseOptExtension<'a>),
}

/// A base selection.
///
/// ```abnf
/// option-extension = (option-standard-tag / option-vendor-tag) [SP option-value]
/// option-value = "(" option-val-comp ")"
/// ```
///
/// See [RFC 5258, section 3](https://www.rfc-editor.org/rfc/rfc5258#section-3).
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct ListSelectBaseOptExtension<'a> {
    /// The option tag (`option-standard-tag` / `option-vendor-tag`).
    pub tag: ListSelectBaseOptExtensionTag<'a>,
    /// The optional `option-value` (`"(" option-val-comp ")"`).
    ///
    /// `None` means the option carries no value.
    pub value: Option<TaggedExtComp<'a>>,
}

/// The tag of a [`ListSelectBaseOptExtension`].
///
/// It's guaranteed that this type can't represent the standardized base option
/// name `SUBSCRIBED` (which has its own [`ListSelectBaseOpt::Subscribed`]
/// variant).
///
/// Note: `option-standard-tag` and `option-vendor-tag` are not distinguished.
/// Both are syntactically an `atom` (`option-vendor-tag = vendor-token "-" atom`
/// is still a single run of atom-chars), so a single [`Atom`] captures both; the
/// standard-vs-vendor distinction is a semantic (registry) concern the grammar
/// can't express, not a parseable one.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "Atom"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct ListSelectBaseOptExtensionTag<'a>(Atom<'a>);

impl<'a> ListSelectBaseOptExtensionTag<'a> {
    pub fn validate(atom: &Atom) -> Result<(), OptionExtensionTagError> {
        if atom.as_ref().eq_ignore_ascii_case("subscribed") {
            return Err(OptionExtensionTagError::Reserved);
        }

        Ok(())
    }

    pub fn inner(&self) -> &Atom<'a> {
        &self.0
    }

    /// Constructs an option-extension tag without validation.
    ///
    /// # Warning: IMAP conformance
    ///
    /// The caller must ensure that `atom` is valid according to
    /// [`Self::validate`]. Failing to do so may create invalid/unparsable IMAP
    /// messages, or even produce unintended protocol flows.
    ///
    /// Note: This method will `panic!` on wrong input in debug builds.
    pub fn unvalidated(atom: Atom<'a>) -> Self {
        #[cfg(debug_assertions)]
        Self::validate(&atom).unwrap();

        Self(atom)
    }
}

impl<'a> TryFrom<Atom<'a>> for ListSelectBaseOptExtensionTag<'a> {
    type Error = OptionExtensionTagError;

    fn try_from(atom: Atom<'a>) -> Result<Self, Self::Error> {
        Self::validate(&atom)?;

        Ok(Self(atom))
    }
}

/// An extended data item of a `LIST` response.
///
/// ```abnf
/// mbox-list-extended-item = mbox-list-extended-item-tag SP tagged-ext-val
/// childinfo-extended-item = "CHILDINFO" SP "("
///     list-select-base-opt-quoted *(SP list-select-base-opt-quoted) ")"
/// ```
///
/// See [RFC 5258, section 3.5](https://www.rfc-editor.org/rfc/rfc5258#section-3.5).
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub enum MboxListExtendedItem<'a> {
    /// The `CHILDINFO` extended data item.
    ChildInfo(Vec1<ListSelectBaseOpt<'a>>),

    /// A generic (vendor or standard) extended data item.
    Other {
        /// The item tag (`mbox-list-extended-item-tag`).
        tag: MboxListExtendedItemTag<'a>,
        /// The item value (`tagged-ext-val`).
        value: TaggedExtVal<'a>,
    },
}

/// The tag of a generic [`MboxListExtendedItem::Other`].
///
/// It's guaranteed that this type can't represent the `CHILDINFO` item (which
/// has its own [`MboxListExtendedItem::ChildInfo`] variant).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "AString"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct MboxListExtendedItemTag<'a>(AString<'a>);

impl<'a> MboxListExtendedItemTag<'a> {
    pub fn validate(value: impl AsRef<[u8]>) -> Result<(), MboxListExtendedItemTagError> {
        if value.as_ref().eq_ignore_ascii_case(b"childinfo") {
            return Err(MboxListExtendedItemTagError::Reserved);
        }

        Ok(())
    }

    pub fn inner(&self) -> &AString<'a> {
        &self.0
    }

    /// Constructs an extended-item tag without validation.
    ///
    /// # Warning: IMAP conformance
    ///
    /// The caller must ensure that `value` is valid according to
    /// [`Self::validate`]. Failing to do so may create invalid/unparsable IMAP
    /// messages, or even produce unintended protocol flows.
    ///
    /// Note: This method will `panic!` on wrong input in debug builds.
    pub fn unvalidated(value: AString<'a>) -> Self {
        #[cfg(debug_assertions)]
        Self::validate(&value).unwrap();

        Self(value)
    }
}

impl<'a> TryFrom<AString<'a>> for MboxListExtendedItemTag<'a> {
    type Error = MboxListExtendedItemTagError;

    fn try_from(value: AString<'a>) -> Result<Self, Self::Error> {
        Self::validate(&value)?;

        Ok(Self(value))
    }
}

#[cfg(feature = "arbitrary")]
impl_arbitrary_try_from! { ListSelectBaseOptExtensionTag<'a>, Atom<'a> }
#[cfg(feature = "arbitrary")]
impl_arbitrary_try_from! { MboxListExtendedItemTag<'a>, AString<'a> }

/// Error-related types.
pub mod error {
    use thiserror::Error;

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum OptionExtensionTagError {
        #[error("Reserved: Please use one of the typed variants")]
        Reserved,
    }

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum MboxListExtendedItemTagError {
        #[error("Reserved: Please use one of the typed variants")]
        Reserved,
    }
}

#[cfg(test)]
mod tests {
    use super::{error::*, *};

    #[test]
    fn test_mbox_list_extended_item_tag_rejects_childinfo() {
        let reserved = MboxListExtendedItemTag::try_from(AString::try_from("ChildInfo").unwrap());
        assert_eq!(reserved, Err(MboxListExtendedItemTagError::Reserved));
        assert!(MboxListExtendedItemTag::try_from(AString::try_from("TAG").unwrap()).is_ok());
    }

    #[test]
    fn test_list_select_base_opt_ext_tag_rejects_subscribed() {
        let reserved =
            ListSelectBaseOptExtensionTag::try_from(Atom::try_from("SubScribed").unwrap());

        assert_eq!(reserved, Err(OptionExtensionTagError::Reserved));
        assert!(ListSelectBaseOptExtensionTag::try_from(Atom::try_from("REMOTE").unwrap()).is_ok());
        assert!(
            ListSelectBaseOptExtensionTag::try_from(Atom::try_from("VENDOR.X-FOO").unwrap())
                .is_ok()
        );
    }
}
