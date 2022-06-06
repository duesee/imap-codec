use std::borrow::Cow;

#[cfg(feature = "ext_enable")]
use crate::extensions::rfc5161::CapabilityEnable;
use crate::{
    address::Address,
    body::{
        BasicFields, Body, BodyStructure, MultiPartExtensionData, SinglePartExtensionData,
        SpecificFields,
    },
    command::{Command, CommandBody, SearchKey},
    core::{
        AString, Atom, AtomExt, Charset, IString, Literal, NString, NonEmptyVec, Quoted, Tag, Text,
    },
    envelope::Envelope,
    fetch_attributes::{FetchAttribute, FetchAttributeValue, MacroOrFetchAttributes},
    flag::{Flag, FlagNameAttribute},
    mailbox::{ListCharString, ListMailbox, Mailbox, MailboxOther},
    response::{Capability, Code, Continuation, Data, Response, Status},
    section::Section,
    AuthMechanism, AuthMechanismOther,
};

// -------------------------------------------------------------------------------------------------

impl<'a> Command<'a> {
    pub fn into_owned(self) -> Command<'static> {
        IntoOwned::into_owned(self)
    }
}

impl<'a> Response<'a> {
    pub fn into_owned(self) -> Response<'static> {
        IntoOwned::into_owned(self)
    }
}

// -------------------------------------------------------------------------------------------------

pub trait IntoOwned {
    type Item;

    fn into_owned(self) -> Self::Item;
}

#[macro_export]
macro_rules! impl_inner_into_static {
    ($target:ty, $item:ty) => {
        impl<'a> IntoOwned for $target {
            type Item = $item;

            fn into_owned(self) -> Self::Item {
                Self::Item {
                    inner: IntoOwned::into_owned(self.inner),
                }
            }
        }
    };
}

impl_inner_into_static! {Atom<'a>, Atom<'static>}

impl_inner_into_static! {AtomExt<'a>, AtomExt<'static>}

impl<'a> IntoOwned for IString<'a> {
    type Item = IString<'static>;

    fn into_owned(self) -> Self::Item {
        match self {
            IString::Literal(val) => IString::Literal(val.into_owned()),
            IString::Quoted(val) => IString::Quoted(val.into_owned()),
        }
    }
}

impl_inner_into_static! {Literal<'a>, Literal<'static>}

impl_inner_into_static! {Quoted<'a>, Quoted<'static>}

impl_inner_into_static! {NString<'a>, NString<'static>}

impl<'a> IntoOwned for AString<'a> {
    type Item = AString<'static>;

    fn into_owned(self) -> Self::Item {
        use AString::*;

        match self {
            Atom(val) => Atom(val.into_owned()),
            String(val) => String(val.into_owned()),
        }
    }
}

impl_inner_into_static! {Tag<'a>, Tag<'static>}

impl_inner_into_static! {Text<'a>, Text<'static>}
impl<T> IntoOwned for NonEmptyVec<T>
where
    T: IntoOwned,
{
    type Item = NonEmptyVec<<T as IntoOwned>::Item>;

    fn into_owned(self) -> Self::Item {
        NonEmptyVec {
            inner: (self.inner.into_iter().map(IntoOwned::into_owned).collect()),
        }
    }
}

// -------------------------------------------------------------------------------------------------

impl<'a> IntoOwned for Command<'a> {
    type Item = Command<'static>;

    fn into_owned(self) -> Self::Item {
        Command {
            tag: self.tag.into_owned(),
            body: self.body.into_owned(),
        }
    }
}

impl<'a> IntoOwned for CommandBody<'a> {
    type Item = CommandBody<'static>;

    fn into_owned(self) -> Self::Item {
        use CommandBody::*;

        match self {
            Capability => Capability,
            Noop => Noop,
            Logout => Logout,
            StartTLS => StartTLS,
            Authenticate {
                mechanism,
                initial_response,
            } => Authenticate {
                mechanism: mechanism.into_owned(),
                initial_response: initial_response.into_owned(),
            },
            Login { username, password } => Login {
                username: username.into_owned(),
                password: password.into_owned(),
            },
            Select { mailbox } => Select {
                mailbox: mailbox.into_owned(),
            },
            Examine { mailbox } => Examine {
                mailbox: mailbox.into_owned(),
            },
            Create { mailbox } => Create {
                mailbox: mailbox.into_owned(),
            },
            Delete { mailbox } => Delete {
                mailbox: mailbox.into_owned(),
            },
            Rename {
                mailbox,
                new_mailbox,
            } => Rename {
                mailbox: mailbox.into_owned(),
                new_mailbox: new_mailbox.into_owned(),
            },
            Subscribe { mailbox } => Subscribe {
                mailbox: mailbox.into_owned(),
            },
            Unsubscribe { mailbox } => Unsubscribe {
                mailbox: mailbox.into_owned(),
            },
            List {
                reference,
                mailbox_wildcard,
            } => List {
                reference: reference.into_owned(),
                mailbox_wildcard: mailbox_wildcard.into_owned(),
            },
            Lsub {
                reference,
                mailbox_wildcard,
            } => Lsub {
                reference: reference.into_owned(),
                mailbox_wildcard: mailbox_wildcard.into_owned(),
            },
            Status {
                mailbox,
                attributes,
            } => Status {
                mailbox: mailbox.into_owned(),
                attributes,
            },
            Append {
                mailbox,
                flags,
                date,
                message,
            } => Append {
                mailbox: mailbox.into_owned(),
                flags: flags.into_owned(),
                date,
                message: message.into_owned(),
            },
            Check => Check,
            Close => Close,
            Expunge => Expunge,
            Search {
                charset,
                criteria,
                uid,
            } => Search {
                charset: charset.into_owned(),
                criteria: criteria.into_owned(),
                uid,
            },
            Fetch {
                sequence_set,
                attributes,
                uid,
            } => Fetch {
                sequence_set,
                attributes: attributes.into_owned(),
                uid,
            },
            Store {
                sequence_set,
                kind,
                response,
                flags,
                uid,
            } => Store {
                sequence_set,
                kind,
                response,
                flags: flags.into_owned(),
                uid,
            },
            Copy {
                sequence_set,
                mailbox,
                uid,
            } => Copy {
                sequence_set,
                mailbox: mailbox.into_owned(),
                uid,
            },
            #[cfg(feature = "ext_idle")]
            Idle => Idle,
            #[cfg(feature = "ext_enable")]
            Enable { capabilities } => Enable {
                capabilities: capabilities.into_owned(),
            },
            #[cfg(feature = "ext_compress")]
            Compress { algorithm } => Compress { algorithm },
        }
    }
}

impl<'a> IntoOwned for AuthMechanism<'a> {
    type Item = AuthMechanism<'static>;

    fn into_owned(self) -> Self::Item {
        use AuthMechanism::*;

        match self {
            Plain => Plain,
            Login => Login,
            Other(val) => Other(val.into_owned()),
        }
    }
}

impl_inner_into_static! {AuthMechanismOther<'a>, AuthMechanismOther<'static>}

impl<'a> IntoOwned for Mailbox<'a> {
    type Item = Mailbox<'static>;

    fn into_owned(self) -> Self::Item {
        use Mailbox::*;

        match self {
            Inbox => Inbox,
            Other(val) => Other(val.into_owned()),
        }
    }
}

impl_inner_into_static! {MailboxOther<'a>, MailboxOther<'static>}

impl<'a> IntoOwned for ListMailbox<'a> {
    type Item = ListMailbox<'static>;

    fn into_owned(self) -> Self::Item {
        match self {
            ListMailbox::Token(val) => ListMailbox::Token(val.into_owned()),
            ListMailbox::String(val) => ListMailbox::String(val.into_owned()),
        }
    }
}

impl<'a> IntoOwned for Charset<'a> {
    type Item = Charset<'static>;

    fn into_owned(self) -> Self::Item {
        use Charset::*;

        match self {
            Atom(val) => Atom(val.into_owned()),
            Quoted(val) => Quoted(val.into_owned()),
        }
    }
}

impl<'a> IntoOwned for SearchKey<'a> {
    type Item = SearchKey<'static>;

    fn into_owned(self) -> Self::Item {
        use SearchKey::*;

        match self {
            And(val) => And(val.into_owned()),
            SequenceSet(val) => SequenceSet(val),
            All => All,
            Answered => Answered,
            Bcc(val) => Bcc(val.into_owned()),
            Before(val) => Before(val),
            Body(val) => Body(val.into_owned()),
            Cc(val) => Cc(val.into_owned()),
            Deleted => Deleted,
            Draft => Draft,
            Flagged => Flagged,
            From(val) => From(val.into_owned()),
            Header(val1, val2) => Header(val1.into_owned(), val2.into_owned()),
            Keyword(val) => Keyword(val.into_owned()),
            Larger(val) => Larger(val),
            New => New,
            Not(val) => Not(val.into_owned()),
            Old => Old,
            On(val) => On(val),
            Or(val1, val2) => Or(val1.into_owned(), val2.into_owned()),
            Recent => Recent,
            Seen => Seen,
            SentBefore(val) => SentBefore(val),
            SentOn(val) => SentOn(val),
            SentSince(val) => SentSince(val),
            Since(val) => Since(val),
            Smaller(val) => Smaller(val),
            Subject(val) => Subject(val.into_owned()),
            Text(val) => Text(val.into_owned()),
            To(val) => To(val.into_owned()),
            Uid(val) => Uid(val),
            Unanswered => Unanswered,
            Undeleted => Undeleted,
            Undraft => Undraft,
            Unflagged => Unflagged,
            Unkeyword(val) => Unkeyword(val.into_owned()),
            Unseen => Unseen,
        }
    }
}

impl<'a> IntoOwned for MacroOrFetchAttributes<'a> {
    type Item = MacroOrFetchAttributes<'static>;

    fn into_owned(self) -> Self::Item {
        use MacroOrFetchAttributes::*;

        match self {
            Macro(val) => Macro(val),
            FetchAttributes(val) => FetchAttributes(val.into_owned()),
        }
    }
}

impl<'a> IntoOwned for FetchAttribute<'a> {
    type Item = FetchAttribute<'static>;

    fn into_owned(self) -> Self::Item {
        use FetchAttribute::*;

        match self {
            Body => Body,
            BodyExt {
                section,
                partial,
                peek,
            } => BodyExt {
                section: section.into_owned(),
                partial,
                peek,
            },
            BodyStructure => BodyStructure,
            Envelope => Envelope,
            Flags => Flags,
            InternalDate => InternalDate,
            Rfc822 => Rfc822,
            Rfc822Header => Rfc822Header,
            Rfc822Size => Rfc822Size,
            Rfc822Text => Rfc822Text,
            Uid => Uid,
        }
    }
}

impl<'a> IntoOwned for FetchAttributeValue<'a> {
    type Item = FetchAttributeValue<'static>;

    fn into_owned(self) -> Self::Item {
        use FetchAttributeValue::*;

        match self {
            Body(val) => Body(val.into_owned()),
            BodyExt {
                section,
                origin,
                data,
            } => BodyExt {
                section: section.into_owned(),
                origin,
                data: data.into_owned(),
            },
            BodyStructure(val) => BodyStructure(val.into_owned()),
            Envelope(val) => Envelope(val.into_owned()),
            Flags(val) => Flags(val.into_owned()),
            InternalDate(val) => InternalDate(val),
            Rfc822(val) => Rfc822(val.into_owned()),
            Rfc822Header(val) => Rfc822Header(val.into_owned()),
            Rfc822Size(val) => Rfc822Size(val),
            Rfc822Text(val) => Rfc822Text(val.into_owned()),
            Uid(val) => Uid(val),
        }
    }
}

impl<'a> IntoOwned for Section<'a> {
    type Item = Section<'static>;

    fn into_owned(self) -> Self::Item {
        use Section::*;

        match self {
            Part(vl) => Part(vl),
            Header(vl) => Header(vl),
            HeaderFields(val1, val2) => HeaderFields(val1, val2.into_owned()),
            HeaderFieldsNot(val1, val2) => HeaderFieldsNot(val1, val2.into_owned()),
            Text(val) => Text(val),
            Mime(val) => Mime(val),
        }
    }
}

impl<'a> IntoOwned for Flag<'a> {
    type Item = Flag<'static>;

    fn into_owned(self) -> Self::Item {
        use Flag::*;

        match self {
            Seen => Seen,
            Answered => Answered,
            Flagged => Flagged,
            Deleted => Deleted,
            Draft => Draft,
            Recent => Recent,
            NameAttribute(val) => NameAttribute(val.into_owned()),
            Permanent => Permanent,
            Keyword(val) => Keyword(val.into_owned()),
            Extension(val) => Extension(val.into_owned()),
        }
    }
}

impl<'a> IntoOwned for FlagNameAttribute<'a> {
    type Item = FlagNameAttribute<'static>;

    fn into_owned(self) -> Self::Item {
        use FlagNameAttribute::*;

        match self {
            Noinferiors => Noinferiors,
            Noselect => Noselect,
            Marked => Marked,
            Unmarked => Unmarked,
            Extension(val) => Extension(val.into_owned()),
        }
    }
}

impl_inner_into_static! {ListCharString<'a>, ListCharString<'static>}

// -------------------------------------------------------------------------------------------------

impl<'a> IntoOwned for Response<'a> {
    type Item = Response<'static>;

    fn into_owned(self) -> Self::Item {
        use Response::*;

        match self {
            Status(val) => Status(val.into_owned()),
            Data(val) => Data(val.into_owned()),
            Continuation(val) => Continuation(val.into_owned()),
        }
    }
}

impl<'a> IntoOwned for Status<'a> {
    type Item = Status<'static>;

    fn into_owned(self) -> Self::Item {
        use Status::*;

        match self {
            Ok { tag, code, text } => Ok {
                tag: tag.into_owned(),
                code: code.into_owned(),
                text: text.into_owned(),
            },
            No { tag, code, text } => No {
                tag: tag.into_owned(),
                code: code.into_owned(),
                text: text.into_owned(),
            },
            Bad { tag, code, text } => Bad {
                tag: tag.into_owned(),
                code: code.into_owned(),
                text: text.into_owned(),
            },
            PreAuth { code, text } => PreAuth {
                code: code.into_owned(),
                text: text.into_owned(),
            },
            Bye { code, text } => Bye {
                code: code.into_owned(),
                text: text.into_owned(),
            },
        }
    }
}

impl<'a> IntoOwned for Code<'a> {
    type Item = Code<'static>;

    fn into_owned(self) -> Self::Item {
        use Code::*;

        match self {
            Alert => Alert,
            BadCharset(val) => BadCharset(val.into_owned()),
            Capability(val) => Capability(val.into_owned()),
            Parse => Parse,
            PermanentFlags(val) => PermanentFlags(val.into_owned()),
            ReadOnly => ReadOnly,
            ReadWrite => ReadWrite,
            TryCreate => TryCreate,
            UidNext(val) => UidNext(val),
            UidValidity(val) => UidValidity(val),
            Unseen(val) => Unseen(val),
            Other(val1, val2) => Other(val1.into_owned(), val2.into_owned()),
            Referral(val) => Referral(IntoOwned::into_owned(val)),
            #[cfg(feature = "ext_compress")]
            CompressionActive => CompressionActive,
        }
    }
}

impl<'a> IntoOwned for Capability<'a> {
    type Item = Capability<'static>;

    fn into_owned(self) -> Self::Item {
        use Capability::*;

        match self {
            Imap4Rev1 => Imap4Rev1,
            Auth(val) => Auth(val.into_owned()),
            LoginDisabled => LoginDisabled,
            StartTls => StartTls,
            #[cfg(feature = "ext_idle")]
            Idle => Idle,
            MailboxReferrals => MailboxReferrals,
            LoginReferrals => LoginReferrals,
            SaslIr => SaslIr,
            #[cfg(feature = "ext_enable")]
            Enable => Enable,
            #[cfg(feature = "ext_compress")]
            Compress { algorithm } => Compress { algorithm },
            Other(val) => Other(val.into_owned()),
        }
    }
}

#[cfg(feature = "ext_enable")]
impl<'a> IntoOwned for CapabilityEnable<'a> {
    type Item = CapabilityEnable<'static>;

    fn into_owned(self) -> Self::Item {
        use CapabilityEnable::*;

        match self {
            Utf8(val) => Utf8(val),
            Other(val) => Other(val.into_owned()),
        }
    }
}

impl<'a> IntoOwned for Data<'a> {
    type Item = Data<'static>;

    fn into_owned(self) -> Self::Item {
        use Data::*;

        match self {
            Capability(val) => Capability(val.into_owned()),
            List {
                items,
                delimiter,
                mailbox,
            } => List {
                items: items.into_owned(),
                delimiter,
                mailbox: mailbox.into_owned(),
            },
            Lsub {
                items,
                delimiter,
                mailbox,
            } => Lsub {
                items: items.into_owned(),
                delimiter,
                mailbox: mailbox.into_owned(),
            },
            Status {
                mailbox,
                attributes,
            } => Status {
                mailbox: mailbox.into_owned(),
                attributes,
            },
            Search(val) => Search(val),
            Flags(val) => Flags(val.into_owned()),
            Exists(val) => Exists(val),
            Recent(val) => Recent(val),
            Expunge(val) => Expunge(val),
            Fetch {
                seq_or_uid,
                attributes,
            } => Fetch {
                seq_or_uid,
                attributes: attributes.into_owned(),
            },
            #[cfg(feature = "ext_enable")]
            Enabled { capabilities } => Enabled {
                capabilities: capabilities.into_owned(),
            },
        }
    }
}

impl<'a> IntoOwned for Continuation<'a> {
    type Item = Continuation<'static>;

    fn into_owned(self) -> Self::Item {
        use Continuation::*;

        match self {
            Basic { code, text } => Basic {
                code: code.into_owned(),
                text: text.into_owned(),
            },
            Base64(val) => Base64(IntoOwned::into_owned(val)),
        }
    }
}

impl<'a> IntoOwned for BodyStructure<'a> {
    type Item = BodyStructure<'static>;

    fn into_owned(self) -> Self::Item {
        use BodyStructure::*;

        match self {
            Single { body, extension } => Single {
                body: body.into_owned(),
                extension: extension.into_owned(),
            },
            Multi {
                bodies,
                subtype,
                extension_data,
            } => Multi {
                bodies: bodies.into_owned(),
                subtype: subtype.into_owned(),
                extension_data: extension_data.into_owned(),
            },
        }
    }
}

impl<'a> IntoOwned for Body<'a> {
    type Item = Body<'static>;

    fn into_owned(self) -> Self::Item {
        Body {
            basic: self.basic.into_owned(),
            specific: self.specific.into_owned(),
        }
    }
}

impl<'a> IntoOwned for BasicFields<'a> {
    type Item = BasicFields<'static>;

    fn into_owned(self) -> Self::Item {
        BasicFields {
            parameter_list: self.parameter_list.into_owned(),
            id: self.id.into_owned(),
            description: self.description.into_owned(),
            content_transfer_encoding: self.content_transfer_encoding.into_owned(),
            size: self.size,
        }
    }
}

impl<'a> IntoOwned for SpecificFields<'a> {
    type Item = SpecificFields<'static>;

    fn into_owned(self) -> Self::Item {
        use SpecificFields::*;

        match self {
            Basic { type_, subtype } => Basic {
                type_: type_.into_owned(),
                subtype: subtype.into_owned(),
            },
            Message {
                envelope,
                body_structure,
                number_of_lines,
            } => Message {
                envelope: envelope.into_owned(),
                body_structure: body_structure.into_owned(),
                number_of_lines,
            },
            Text {
                subtype,
                number_of_lines,
            } => Text {
                subtype: subtype.into_owned(),
                number_of_lines,
            },
        }
    }
}

impl<'a> IntoOwned for Envelope<'a> {
    type Item = Envelope<'static>;

    fn into_owned(self) -> Self::Item {
        Envelope {
            date: self.date.into_owned(),
            subject: self.subject.into_owned(),
            from: self.from.into_owned(),
            sender: self.sender.into_owned(),
            reply_to: self.reply_to.into_owned(),
            to: self.to.into_owned(),
            cc: self.cc.into_owned(),
            bcc: self.bcc.into_owned(),
            in_reply_to: self.in_reply_to.into_owned(),
            message_id: self.message_id.into_owned(),
        }
    }
}

impl<'a> IntoOwned for Address<'a> {
    type Item = Address<'static>;

    fn into_owned(self) -> Self::Item {
        Address {
            name: self.name.into_owned(),
            adl: self.adl.into_owned(),
            mailbox: self.mailbox.into_owned(),
            host: self.host.into_owned(),
        }
    }
}

impl<'a> IntoOwned for SinglePartExtensionData<'a> {
    type Item = SinglePartExtensionData<'static>;

    fn into_owned(self) -> Self::Item {
        SinglePartExtensionData {
            md5: self.md5.into_owned(),
            disposition: self.disposition.into_owned(),
            language: self.language.into_owned(),
            location: self.location.into_owned(),
            extension: IntoOwned::into_owned(self.extension),
        }
    }
}

impl<'a> IntoOwned for MultiPartExtensionData<'a> {
    type Item = MultiPartExtensionData<'static>;

    fn into_owned(self) -> Self::Item {
        MultiPartExtensionData {
            parameter_list: self.parameter_list.into_owned(),
            disposition: self.disposition.into_owned(),
            language: self.language.into_owned(),
            location: self.location.into_owned(),
            extension: IntoOwned::into_owned(self.extension),
        }
    }
}

// -------------------------------------------------------------------------------------------------

impl<T> IntoOwned for Option<T>
where
    T: IntoOwned,
{
    type Item = Option<<T as IntoOwned>::Item>;

    fn into_owned(self) -> Self::Item {
        self.map(IntoOwned::into_owned)
    }
}

impl<'a, L, R> IntoOwned for (L, R)
where
    L: IntoOwned,
    R: IntoOwned,
{
    type Item = (<L as IntoOwned>::Item, <R as IntoOwned>::Item);

    fn into_owned(self) -> Self::Item {
        (self.0.into_owned(), self.1.into_owned())
    }
}

impl<T> IntoOwned for Vec<T>
where
    T: IntoOwned,
{
    type Item = Vec<<T as IntoOwned>::Item>;

    fn into_owned(self) -> Self::Item {
        self.into_iter().map(IntoOwned::into_owned).collect()
    }
}

impl<T> IntoOwned for Box<T>
where
    T: IntoOwned,
{
    type Item = Box<<T as IntoOwned>::Item>;

    fn into_owned(self) -> Self::Item {
        Box::new((*self).into_owned())
    }
}

impl<'a> IntoOwned for Cow<'a, str> {
    type Item = Cow<'static, str>;

    fn into_owned(self) -> Self::Item {
        Cow::Owned(self.into_owned())
    }
}

impl<'a> IntoOwned for Cow<'a, [u8]> {
    type Item = Cow<'static, [u8]>;

    fn into_owned(self) -> Self::Item {
        Cow::Owned(self.into_owned())
    }
}
