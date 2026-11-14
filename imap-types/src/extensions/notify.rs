//! The IMAP NOTIFY Extension ([RFC 5465])
//!
//! The types make the command's rules unrepresentable rather than checked:
//! MessageNew and MessageExpunge always go together, FlagChange and
//! AnnotationChange only with them, fetch attributes only for the selected
//! mailbox, at most one `SELECTED`/`SELECTED-DELAYED` group per command,
//! no empty event list, and an extension event can't spell a known one.
//! Where the order of groups or events carries no meaning, the selected
//! group and the message events are encoded first.
//!
//! [RFC 5465]: https://www.rfc-editor.org/rfc/rfc5465

use alloc::vec::Vec;
use core::fmt::{Display, Formatter};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    core::{Atom, Vec1},
    error::{ValidationError, ValidationErrorKind},
    fetch::MessageDataItemName,
    mailbox::Mailbox,
};

/// The NOTIFY command.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, ToStatic)]
pub enum Notify<'a> {
    /// `NOTIFY SET`: replace the registered events.
    Set {
        /// `STATUS`: send a STATUS response for every watched mailbox first.
        status: bool,
        groups: EventGroups<'a>,
    },
    /// `NOTIFY NONE`: cancel all registered events.
    None,
}

/// The event groups of a `NOTIFY SET`, at most one of them for the selected mailbox.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, ToStatic)]
pub enum EventGroups<'a> {
    /// A group for the selected mailbox, and any others.
    ///
    /// Encoded with the selected group first.
    WithSelected {
        selected: SelectedGroup<'a>,
        others: Vec<MailboxGroup<'a>>,
    },
    /// Groups for other (non-selected) mailboxes only.
    Others(Vec1<MailboxGroup<'a>>),
}

/// `(SELECTED ...)` or `(SELECTED-DELAYED ...)`.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, ToStatic)]
pub struct SelectedGroup<'a> {
    /// `SELECTED-DELAYED`: expunges are reported only when it is safe.
    pub delayed: bool,
    /// `None` means `NONE`: no events for the selected mailbox.
    pub events: Option<SelectedEvents<'a>>,
}

/// Message events for the selected mailbox, which may ask for fetch attributes.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, ToStatic)]
pub struct SelectedEvents<'a> {
    pub message: MessageEvents,
    /// `MessageNew (fetch-att ...)`: what the server sends for each new message.
    pub fetch: Option<Vec1<MessageDataItemName<'a>>>,
}

/// `MessageNew` and `MessageExpunge`, which are always requested together,
/// optionally with `FlagChange` and `AnnotationChange`.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, ToStatic)]
pub struct MessageEvents {
    pub flag_change: bool,
    pub annotation_change: bool,
}

/// `(filter-mailboxes events)` for mailboxes other than the selected one.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, ToStatic)]
pub struct MailboxGroup<'a> {
    pub filter: FilterMailboxes<'a>,
    /// `None` means `NONE`: no events for these mailboxes.
    pub events: Option<MailboxEvents<'a>>,
}

/// The events of a [`MailboxGroup`]: never empty, with the message events at most once.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, ToStatic)]
pub enum MailboxEvents<'a> {
    /// Message events, and any others.
    ///
    /// Encoded with the message events first.
    WithMessage {
        message: MessageEvents,
        others: Vec<Event<'a>>,
    },
    /// No message events.
    Only(Vec1<Event<'a>>),
}

/// Which (non-selected) mailboxes a group applies to.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, ToStatic)]
pub enum FilterMailboxes<'a> {
    /// `INBOXES`: mailboxes the user's mail is delivered to.
    Inboxes,
    /// `PERSONAL`: the user's personal mailboxes.
    Personal,
    /// `SUBSCRIBED`: the subscribed mailboxes.
    Subscribed,
    /// `SUBTREE`: these mailboxes and all below them.
    Subtree(Vec1<Mailbox<'a>>),
    /// `MAILBOXES`: these mailboxes only.
    Mailboxes(Vec1<Mailbox<'a>>),
}

/// A mailbox, user or extension event; message events are [`MessageEvents`].
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, ToStatic)]
pub enum Event<'a> {
    MailboxName,
    SubscriptionChange,
    MailboxMetadataChange,
    ServerMetadataChange,
    /// An event defined by a later extension (`event-ext`).
    Other(EventOther<'a>),
}

/// An extension event; never one of the names RFC 5465 defines.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "Atom"))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, ToStatic)]
pub struct EventOther<'a>(Atom<'a>);

impl<'a> EventOther<'a> {
    pub fn inner(&self) -> &Atom<'a> {
        &self.0
    }
}

impl<'a> TryFrom<Atom<'a>> for EventOther<'a> {
    type Error = ValidationError;

    fn try_from(atom: Atom<'a>) -> Result<Self, Self::Error> {
        match EventName::known(atom.as_ref()) {
            Some(_) => Err(ValidationError::new(ValidationErrorKind::Invalid)),
            None => Ok(Self(atom)),
        }
    }
}

/// An event the server supports, as the `BADEVENT` response code lists them:
/// one RFC 5465 defines, or an extension's.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, ToStatic)]
pub enum SupportedEvent<'a> {
    Known(EventName),
    Other(EventOther<'a>),
}

/// An event name RFC 5465 defines.
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, ToStatic)]
pub enum EventName {
    MessageNew,
    MessageExpunge,
    FlagChange,
    AnnotationChange,
    MailboxName,
    SubscriptionChange,
    MailboxMetadataChange,
    ServerMetadataChange,
}

impl EventName {
    /// The event named `name`, compared case-insensitively.
    pub fn known(name: &str) -> Option<Self> {
        [
            Self::MessageNew,
            Self::MessageExpunge,
            Self::FlagChange,
            Self::AnnotationChange,
            Self::MailboxName,
            Self::SubscriptionChange,
            Self::MailboxMetadataChange,
            Self::ServerMetadataChange,
        ]
        .into_iter()
        .find(|e| e.as_str().eq_ignore_ascii_case(name))
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MessageNew => "MessageNew",
            Self::MessageExpunge => "MessageExpunge",
            Self::FlagChange => "FlagChange",
            Self::AnnotationChange => "AnnotationChange",
            Self::MailboxName => "MailboxName",
            Self::SubscriptionChange => "SubscriptionChange",
            Self::MailboxMetadataChange => "MailboxMetadataChange",
            Self::ServerMetadataChange => "ServerMetadataChange",
        }
    }
}

impl Display for EventName {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_other_refuses_known_names() {
        for name in ["MessageNew", "messageexpunge", "FLAGCHANGE", "MailboxName"] {
            assert!(EventOther::try_from(Atom::try_from(name).unwrap()).is_err());
        }
        assert!(EventOther::try_from(Atom::try_from("MessageMoved").unwrap()).is_ok());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn event_other_deserializes_checked() {
        assert!(serde_json::from_str::<EventOther>(r#""MessageNew""#).is_err());
        let other: EventOther = serde_json::from_str(r#""MessageMoved""#).unwrap();
        assert_eq!(other.inner().as_ref(), "MessageMoved");
    }
}
