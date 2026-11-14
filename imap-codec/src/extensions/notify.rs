//! The IMAP NOTIFY Extension

use alloc::vec::Vec;
use core::fmt;

use abnf_core::streaming::sp;
use imap_types::{
    command::CommandBody,
    core::Vec1,
    extensions::notify::{
        Event, EventGroups, EventName, EventOther, FilterMailboxes, MailboxEvents, MailboxGroup,
        MessageEvents, Notify, SelectedEvents, SelectedGroup, SupportedEvent,
    },
    fetch::MessageDataItemName,
    mailbox::Mailbox,
};
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    combinator::{map, opt, value},
    error::ErrorKind,
    multi::separated_list1,
    sequence::{delimited, preceded, tuple},
};

use crate::{
    core::atom,
    decode::{IMAPErrorKind, IMAPParseError, IMAPResult},
    encode::{EncodeContext, EncodeIntoContext, utils::join_serializable},
    fetch::fetch_att,
    mailbox::mailbox,
};

fn invalid<O>(input: &[u8]) -> IMAPResult<&[u8], O> {
    Err(nom::Err::Failure(IMAPParseError {
        input,
        kind: IMAPErrorKind::Nom(ErrorKind::Verify),
    }))
}

// ----- Command -----

/// ```abnf
/// notify     = "NOTIFY" SP (notify-set / notify-none)
/// notify-set = "SET" [status-indicator] SP event-groups
/// notify-none = "NONE"
/// status-indicator = SP "STATUS"
/// ```
pub(crate) fn notify(input: &[u8]) -> IMAPResult<&[u8], CommandBody> {
    let set = map(
        tuple((
            tag_no_case("SET"),
            opt(preceded(sp, tag_no_case("STATUS"))),
            preceded(sp, event_groups),
        )),
        |(_, status, groups)| Notify::Set {
            status: status.is_some(),
            groups,
        },
    );

    map(
        preceded(
            tag_no_case("NOTIFY "),
            alt((set, value(Notify::None, tag_no_case("NONE")))),
        ),
        CommandBody::Notify,
    )(input)
}

enum Group<'a> {
    Selected(SelectedGroup<'a>),
    Mailbox(MailboxGroup<'a>),
}

/// ```abnf
/// event-groups = event-group *(SP event-group)
/// ```
///
/// At most one group may be for the selected mailbox.
pub(crate) fn event_groups(input: &[u8]) -> IMAPResult<&[u8], EventGroups> {
    let (rem, groups) = separated_list1(sp, event_group)(input)?;

    let mut selected = None;
    let mut others = Vec::new();
    for group in groups {
        match group {
            Group::Selected(group) => {
                if selected.replace(group).is_some() {
                    return invalid(input);
                }
            }
            Group::Mailbox(group) => others.push(group),
        }
    }

    let groups = match selected {
        Some(selected) => EventGroups::WithSelected { selected, others },
        None => EventGroups::Others(Vec1::unvalidated(others)),
    };

    Ok((rem, groups))
}

/// ```abnf
/// event-group = "(" filter-mailboxes SP events ")"
/// filter-mailboxes = filter-mailboxes-selected / filter-mailboxes-other
/// filter-mailboxes-selected = "selected" / "selected-delayed"
/// ```
fn event_group(input: &[u8]) -> IMAPResult<&[u8], Group> {
    let selected = map(
        tuple((
            alt((
                value(true, tag_no_case("SELECTED-DELAYED")),
                value(false, tag_no_case("SELECTED")),
            )),
            preceded(sp, events),
        )),
        |(delayed, events)| (Some(delayed), None, events),
    );
    let other = map(
        tuple((filter_mailboxes_other, preceded(sp, events))),
        |(filter, events)| (None, Some(filter), events),
    );

    let (rem, (delayed, filter, events)) =
        delimited(tag("("), alt((selected, other)), tag(")"))(input)?;

    // `events` is None for NONE; a list that breaks the RFC's rules is refused.
    let group = match (delayed, filter, events) {
        (Some(delayed), _, None) => Group::Selected(SelectedGroup {
            delayed,
            events: None,
        }),
        (Some(delayed), _, Some(raw)) => match selected_events(raw) {
            Some(events) => Group::Selected(SelectedGroup {
                delayed,
                events: Some(events),
            }),
            None => return invalid(input),
        },
        (None, Some(filter), None) => Group::Mailbox(MailboxGroup {
            filter,
            events: None,
        }),
        (None, Some(filter), Some(raw)) => match mailbox_events(raw) {
            Some(events) => Group::Mailbox(MailboxGroup {
                filter,
                events: Some(events),
            }),
            None => return invalid(input),
        },
        (None, None, _) => unreachable!(),
    };

    Ok((rem, group))
}

/// ```abnf
/// filter-mailboxes-other = "inboxes" / "personal" / "subscribed" /
///                          ( "subtree" SP one-or-more-mailbox ) /
///                          ( "mailboxes" SP one-or-more-mailbox )
/// ```
fn filter_mailboxes_other(input: &[u8]) -> IMAPResult<&[u8], FilterMailboxes> {
    alt((
        value(FilterMailboxes::Inboxes, tag_no_case("INBOXES")),
        value(FilterMailboxes::Personal, tag_no_case("PERSONAL")),
        value(FilterMailboxes::Subscribed, tag_no_case("SUBSCRIBED")),
        map(
            preceded(tag_no_case("SUBTREE "), one_or_more_mailbox),
            FilterMailboxes::Subtree,
        ),
        map(
            preceded(tag_no_case("MAILBOXES "), one_or_more_mailbox),
            FilterMailboxes::Mailboxes,
        ),
    ))(input)
}

/// ```abnf
/// one-or-more-mailbox = mailbox / many-mailboxes
/// many-mailboxes      = "(" mailbox *(SP mailbox) ")"
/// ```
fn one_or_more_mailbox(input: &[u8]) -> IMAPResult<&[u8], Vec1<Mailbox>> {
    alt((
        map(
            delimited(tag("("), separated_list1(sp, mailbox), tag(")")),
            Vec1::unvalidated,
        ),
        map(mailbox, Vec1::from),
    ))(input)
}

/// One event as written, before the message events are checked as a set.
#[derive(Clone)]
enum RawEvent<'a> {
    Known(EventName, Option<Vec1<MessageDataItemName<'a>>>),
    Other(EventOther<'a>),
}

/// ```abnf
/// events = ( "(" event *(SP event) ")" ) / "NONE"
/// ```
///
/// `None` is `NONE`.
fn events(input: &[u8]) -> IMAPResult<&[u8], Option<Vec<RawEvent>>> {
    alt((
        value(None, tag_no_case("NONE")),
        map(
            delimited(tag("("), separated_list1(sp, event), tag(")")),
            Some,
        ),
    ))(input)
}

/// ```abnf
/// event = message-event / mailbox-event / user-event / event-ext
/// message-event = ( "MessageNew" [SP "(" fetch-att *(SP fetch-att) ")" ] ) /
///                 "MessageExpunge" / "FlagChange" / "AnnotationChange"
/// mailbox-event = "MailboxName" / "SubscriptionChange" / "MailboxMetadataChange"
/// user-event    = "ServerMetadataChange"
/// event-ext     = atom
/// ```
fn event(input: &[u8]) -> IMAPResult<&[u8], RawEvent> {
    let (rem, name) = atom(input)?;

    match EventName::known(name.as_ref()) {
        Some(EventName::MessageNew) => {
            let (rem, fetch) = opt(preceded(
                sp,
                delimited(tag("("), separated_list1(sp, fetch_att), tag(")")),
            ))(rem)?;
            Ok((
                rem,
                RawEvent::Known(EventName::MessageNew, fetch.map(Vec1::unvalidated)),
            ))
        }
        Some(known) => Ok((rem, RawEvent::Known(known, None))),
        None => match EventOther::try_from(name) {
            Ok(other) => Ok((rem, RawEvent::Other(other))),
            Err(_) => invalid(input),
        },
    }
}

/// The message events of `raw`, if they are a set the RFC allows: MessageNew
/// and MessageExpunge together, FlagChange and AnnotationChange only with them.
fn message_events(raw: &[RawEvent]) -> Option<Option<MessageEvents>> {
    let has = |wanted: EventName| {
        raw.iter()
            .any(|e| matches!(e, RawEvent::Known(name, _) if *name == wanted))
    };
    let new = has(EventName::MessageNew);
    let events = MessageEvents {
        flag_change: has(EventName::FlagChange),
        annotation_change: has(EventName::AnnotationChange),
    };
    if new != has(EventName::MessageExpunge) {
        return None;
    }
    if !new && (events.flag_change || events.annotation_change) {
        return None;
    }
    Some(new.then_some(events))
}

/// Events for the selected mailbox: message events only.
fn selected_events(raw: Vec<RawEvent>) -> Option<SelectedEvents> {
    let message = message_events(&raw)??;
    let mut fetch = None;
    for event in raw {
        match event {
            RawEvent::Known(EventName::MessageNew, Some(atts)) => fetch = Some(atts),
            RawEvent::Known(
                EventName::MessageNew
                | EventName::MessageExpunge
                | EventName::FlagChange
                | EventName::AnnotationChange,
                _,
            ) => {}
            _ => return None,
        }
    }
    Some(SelectedEvents { message, fetch })
}

/// Events for other mailboxes: no fetch attributes.
fn mailbox_events(raw: Vec<RawEvent>) -> Option<MailboxEvents> {
    let message = message_events(&raw)?;
    let mut others = Vec::new();
    for event in raw {
        others.push(match event {
            RawEvent::Known(EventName::MessageNew, Some(_)) => return None,
            RawEvent::Known(
                EventName::MessageNew
                | EventName::MessageExpunge
                | EventName::FlagChange
                | EventName::AnnotationChange,
                _,
            ) => continue,
            RawEvent::Known(EventName::MailboxName, _) => Event::MailboxName,
            RawEvent::Known(EventName::SubscriptionChange, _) => Event::SubscriptionChange,
            RawEvent::Known(EventName::MailboxMetadataChange, _) => Event::MailboxMetadataChange,
            RawEvent::Known(EventName::ServerMetadataChange, _) => Event::ServerMetadataChange,
            RawEvent::Other(other) => Event::Other(other),
        });
    }
    match message {
        Some(message) => Some(MailboxEvents::WithMessage { message, others }),
        None => Vec1::try_from(others).ok().map(MailboxEvents::Only),
    }
}

impl EncodeIntoContext for Notify<'_> {
    fn encode_ctx(&self, ctx: &mut EncodeContext) -> fmt::Result {
        match self {
            Notify::Set { status, groups } => {
                ctx.write_all(b"NOTIFY SET")?;
                if *status {
                    ctx.write_all(b" STATUS")?;
                }
                ctx.write_all(b" ")?;
                groups.encode_ctx(ctx)
            }
            Notify::None => ctx.write_all(b"NOTIFY NONE"),
        }
    }
}

impl EncodeIntoContext for EventGroups<'_> {
    fn encode_ctx(&self, ctx: &mut EncodeContext) -> fmt::Result {
        match self {
            EventGroups::WithSelected { selected, others } => {
                selected.encode_ctx(ctx)?;
                for other in others {
                    ctx.write_all(b" ")?;
                    other.encode_ctx(ctx)?;
                }
                Ok(())
            }
            EventGroups::Others(others) => join_serializable(others.as_ref(), b" ", ctx),
        }
    }
}

impl EncodeIntoContext for SelectedGroup<'_> {
    fn encode_ctx(&self, ctx: &mut EncodeContext) -> fmt::Result {
        ctx.write_all(if self.delayed {
            b"(SELECTED-DELAYED "
        } else {
            b"(SELECTED "
        })?;
        match &self.events {
            Some(events) => {
                ctx.write_all(b"(MessageNew")?;
                if let Some(fetch) = &events.fetch {
                    ctx.write_all(b" (")?;
                    join_serializable(fetch.as_ref(), b" ", ctx)?;
                    ctx.write_all(b")")?;
                }
                ctx.write_all(b" MessageExpunge")?;
                encode_flag_events(&events.message, ctx)?;
                ctx.write_all(b")")?;
            }
            None => ctx.write_all(b"NONE")?,
        }
        ctx.write_all(b")")
    }
}

fn encode_flag_events(events: &MessageEvents, ctx: &mut EncodeContext) -> fmt::Result {
    if events.flag_change {
        ctx.write_all(b" FlagChange")?;
    }
    if events.annotation_change {
        ctx.write_all(b" AnnotationChange")?;
    }
    Ok(())
}

impl EncodeIntoContext for MailboxGroup<'_> {
    fn encode_ctx(&self, ctx: &mut EncodeContext) -> fmt::Result {
        ctx.write_all(b"(")?;
        self.filter.encode_ctx(ctx)?;
        ctx.write_all(b" ")?;
        match &self.events {
            Some(events) => {
                ctx.write_all(b"(")?;
                events.encode_ctx(ctx)?;
                ctx.write_all(b")")?;
            }
            None => ctx.write_all(b"NONE")?,
        }
        ctx.write_all(b")")
    }
}

impl EncodeIntoContext for MailboxEvents<'_> {
    fn encode_ctx(&self, ctx: &mut EncodeContext) -> fmt::Result {
        match self {
            MailboxEvents::WithMessage { message, others } => {
                ctx.write_all(b"MessageNew MessageExpunge")?;
                encode_flag_events(message, ctx)?;
                for other in others {
                    ctx.write_all(b" ")?;
                    other.encode_ctx(ctx)?;
                }
                Ok(())
            }
            MailboxEvents::Only(events) => join_serializable(events.as_ref(), b" ", ctx),
        }
    }
}

impl EncodeIntoContext for FilterMailboxes<'_> {
    fn encode_ctx(&self, ctx: &mut EncodeContext) -> fmt::Result {
        let (keyword, mailboxes): (&[u8], _) = match self {
            FilterMailboxes::Inboxes => return ctx.write_all(b"INBOXES"),
            FilterMailboxes::Personal => return ctx.write_all(b"PERSONAL"),
            FilterMailboxes::Subscribed => return ctx.write_all(b"SUBSCRIBED"),
            FilterMailboxes::Subtree(mailboxes) => (b"SUBTREE ", mailboxes),
            FilterMailboxes::Mailboxes(mailboxes) => (b"MAILBOXES ", mailboxes),
        };
        ctx.write_all(keyword)?;
        match mailboxes.as_ref() {
            [one] => one.encode_ctx(ctx),
            many => {
                ctx.write_all(b"(")?;
                join_serializable(many, b" ", ctx)?;
                ctx.write_all(b")")
            }
        }
    }
}

impl EncodeIntoContext for Event<'_> {
    fn encode_ctx(&self, ctx: &mut EncodeContext) -> fmt::Result {
        match self {
            Event::MailboxName => ctx.write_all(b"MailboxName"),
            Event::SubscriptionChange => ctx.write_all(b"SubscriptionChange"),
            Event::MailboxMetadataChange => ctx.write_all(b"MailboxMetadataChange"),
            Event::ServerMetadataChange => ctx.write_all(b"ServerMetadataChange"),
            Event::Other(other) => other.inner().encode_ctx(ctx),
        }
    }
}

// ----- Response codes -----

/// ```abnf
/// resp-text-code =/ "NOTIFICATIONOVERFLOW" / unsupported-events-code
/// unsupported-events-code = "BADEVENT" SP "(" event-name *(SP event-name) ")"
/// ```
pub(crate) fn bad_event(input: &[u8]) -> IMAPResult<&[u8], Vec1<SupportedEvent>> {
    map(
        preceded(
            tag_no_case("BADEVENT ("),
            tuple((separated_list1(sp, event_name), tag(")"))),
        ),
        |(names, _)| Vec1::unvalidated(names),
    )(input)
}

/// ```abnf
/// event-name = message-event-name / mailbox-event / user-event
/// ```
///
/// Also any other atom: the server "MUST list names of all events supported
/// by the server", extensions' included.
fn event_name(input: &[u8]) -> IMAPResult<&[u8], SupportedEvent> {
    let (rem, name) = atom(input)?;
    match EventName::known(name.as_ref()) {
        Some(known) => Ok((rem, SupportedEvent::Known(known))),
        None => match EventOther::try_from(name) {
            Ok(other) => Ok((rem, SupportedEvent::Other(other))),
            Err(_) => invalid(input),
        },
    }
}

impl EncodeIntoContext for SupportedEvent<'_> {
    fn encode_ctx(&self, ctx: &mut EncodeContext) -> fmt::Result {
        match self {
            SupportedEvent::Known(name) => name.encode_ctx(ctx),
            SupportedEvent::Other(other) => other.inner().encode_ctx(ctx),
        }
    }
}

impl EncodeIntoContext for EventName {
    fn encode_ctx(&self, ctx: &mut EncodeContext) -> fmt::Result {
        ctx.write_all(self.as_str().as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use imap_types::{
        command::{Command, CommandBody},
        core::{Atom, Text, Vec1},
        extensions::notify::{
            Event, EventGroups, EventName, EventOther, FilterMailboxes, MailboxEvents,
            MailboxGroup, MessageEvents, Notify, SelectedEvents, SelectedGroup, SupportedEvent,
        },
        fetch::MessageDataItemName,
        mailbox::Mailbox,
        response::{Code, Response, Status, StatusBody, StatusKind},
    };

    use crate::{
        CommandCodec,
        decode::Decoder,
        testing::{kat_inverse_command, kat_inverse_response},
    };

    fn mailbox(name: &str) -> Mailbox<'_> {
        Mailbox::try_from(name).unwrap()
    }

    fn set(status: bool, groups: EventGroups<'static>) -> CommandBody<'static> {
        CommandBody::Notify(Notify::Set { status, groups })
    }

    #[test]
    fn test_kat_inverse_command_notify() {
        let new_expunge = MessageEvents::default();
        kat_inverse_command(&[
            (
                b"A NOTIFY NONE\r\n".as_ref(),
                b"".as_ref(),
                Command::new("A", CommandBody::Notify(Notify::None)).unwrap(),
            ),
            // RFC 5465, section 6.
            (
                b"A NOTIFY SET (MAILBOXES INBOX (MessageNew MessageExpunge)) (PERSONAL (SubscriptionChange))\r\n",
                b"",
                Command::new(
                    "A",
                    set(
                        false,
                        EventGroups::Others(Vec1::unvalidated(vec![
                            MailboxGroup {
                                filter: FilterMailboxes::Mailboxes(Vec1::from(mailbox("INBOX"))),
                                events: Some(MailboxEvents::WithMessage {
                                    message: new_expunge,
                                    others: vec![],
                                }),
                            },
                            MailboxGroup {
                                filter: FilterMailboxes::Personal,
                                events: Some(MailboxEvents::Only(Vec1::from(
                                    Event::SubscriptionChange,
                                ))),
                            },
                        ])),
                    ),
                )
                .unwrap(),
            ),
            // RFC 5465, section 6, second example, in canonical order.
            (
                b"A NOTIFY SET (SELECTED (MessageNew (UID FLAGS) MessageExpunge)) (PERSONAL (MessageNew MessageExpunge FlagChange))\r\n",
                b"",
                Command::new(
                    "A",
                    set(
                        false,
                        EventGroups::WithSelected {
                            selected: SelectedGroup {
                                delayed: false,
                                events: Some(SelectedEvents {
                                    message: new_expunge,
                                    fetch: Some(Vec1::unvalidated(vec![
                                        MessageDataItemName::Uid,
                                        MessageDataItemName::Flags,
                                    ])),
                                }),
                            },
                            others: vec![MailboxGroup {
                                filter: FilterMailboxes::Personal,
                                events: Some(MailboxEvents::WithMessage {
                                    message: MessageEvents {
                                        flag_change: true,
                                        annotation_change: false,
                                    },
                                    others: vec![],
                                }),
                            }],
                        },
                    ),
                )
                .unwrap(),
            ),
            (
                b"A NOTIFY SET STATUS (SELECTED-DELAYED NONE) (SUBTREE (Lists Archive) (MessageNew MessageExpunge MailboxName)) (INBOXES NONE)\r\n",
                b"",
                Command::new(
                    "A",
                    set(
                        true,
                        EventGroups::WithSelected {
                            selected: SelectedGroup {
                                delayed: true,
                                events: None,
                            },
                            others: vec![
                                MailboxGroup {
                                    filter: FilterMailboxes::Subtree(Vec1::unvalidated(vec![
                                        mailbox("Lists"),
                                        mailbox("Archive"),
                                    ])),
                                    events: Some(MailboxEvents::WithMessage {
                                        message: new_expunge,
                                        others: vec![Event::MailboxName],
                                    }),
                                },
                                MailboxGroup {
                                    filter: FilterMailboxes::Inboxes,
                                    events: None,
                                },
                            ],
                        },
                    ),
                )
                .unwrap(),
            ),
        ]);
    }

    #[test]
    fn test_decode_command_notify_canonicalizes() {
        // RFC 5465, section 6: lower case, FlagChange between the pair, selected not first.
        let (_, cmd) = CommandCodec::default()
            .decode(b"a notify set (personal (MessageNew FlagChange MessageExpunge)) (selected (Messagenew (uid flags) messageExpunge))\r\n")
            .unwrap();
        let CommandBody::Notify(Notify::Set {
            groups: EventGroups::WithSelected { selected, others },
            ..
        }) = cmd.body
        else {
            panic!("{cmd:?}");
        };
        assert!(selected.events.unwrap().fetch.is_some());
        assert_eq!(
            others[0].events,
            Some(MailboxEvents::WithMessage {
                message: MessageEvents {
                    flag_change: true,
                    annotation_change: false
                },
                others: vec![]
            })
        );
    }

    #[test]
    fn test_decode_command_notify_refuses_what_the_rfc_forbids() {
        for bad in [
            // MessageNew without MessageExpunge, and the reverse.
            b"a NOTIFY SET (PERSONAL (MessageNew))\r\n".as_ref(),
            b"a NOTIFY SET (PERSONAL (MessageExpunge))\r\n",
            // FlagChange or AnnotationChange without the pair.
            b"a NOTIFY SET (PERSONAL (FlagChange))\r\n",
            b"a NOTIFY SET (PERSONAL (AnnotationChange MailboxName))\r\n",
            // Fetch attributes outside the selected mailbox.
            b"a NOTIFY SET (PERSONAL (MessageNew (UID) MessageExpunge))\r\n",
            // Mailbox events for the selected mailbox.
            b"a NOTIFY SET (SELECTED (MessageNew MessageExpunge MailboxName))\r\n",
            // Two groups for the selected mailbox.
            b"a NOTIFY SET (SELECTED NONE) (SELECTED-DELAYED NONE)\r\n",
            // The RFC's own section 3 example, which omits the parentheses its ABNF requires.
            b"a NOTIFY SET (SUBTREE Lists MessageNew)\r\n",
        ] {
            assert!(
                CommandCodec::default().decode(bad).is_err(),
                "{}",
                core::str::from_utf8(bad).unwrap()
            );
        }
    }

    #[test]
    fn test_kat_inverse_response_notify_codes() {
        kat_inverse_response(&[
            (
                b"* OK [NOTIFICATIONOVERFLOW] ...A comment can go here...\r\n".as_ref(),
                b"".as_ref(),
                Response::Status(
                    Status::new(
                        None,
                        StatusKind::Ok,
                        Some(Code::NotificationOverflow),
                        "...A comment can go here...",
                    )
                    .unwrap(),
                ),
            ),
            (
                b"A NO [BADEVENT (MessageNew MessageExpunge FlagChange MailboxName)] unsupported\r\n",
                b"",
                Response::Status(Status::Tagged(imap_types::response::Tagged {
                    tag: imap_types::core::Tag::try_from("A").unwrap(),
                    body: StatusBody {
                        kind: StatusKind::No,
                        code: Some(Code::BadEvent(Vec1::unvalidated(vec![
                            SupportedEvent::Known(EventName::MessageNew),
                            SupportedEvent::Known(EventName::MessageExpunge),
                            SupportedEvent::Known(EventName::FlagChange),
                            SupportedEvent::Known(EventName::MailboxName),
                        ]))),
                        text: Text::try_from("unsupported").unwrap(),
                    },
                })),
            ),
            (
                b"A NO [BADEVENT (MessageNew MessageMoved)] x\r\n",
                b"",
                Response::Status(Status::Tagged(imap_types::response::Tagged {
                    tag: imap_types::core::Tag::try_from("A").unwrap(),
                    body: StatusBody {
                        kind: StatusKind::No,
                        code: Some(Code::BadEvent(Vec1::unvalidated(vec![
                            SupportedEvent::Known(EventName::MessageNew),
                            SupportedEvent::Other(
                                EventOther::try_from(Atom::try_from("MessageMoved").unwrap())
                                    .unwrap(),
                            ),
                        ]))),
                        text: Text::try_from("x").unwrap(),
                    },
                })),
            ),
        ]);
    }
}
