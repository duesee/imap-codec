//! IMAP QUOTA Extension

use std::convert::TryFrom;

use abnf_core::streaming::SP;
use imap_types::{
    command::CommandBody,
    core::{AString, NonEmptyVec},
    extensions::rfc9208::{QuotaGet, QuotaSet, Resource},
    response::Data,
};
use nom::{
    bytes::{complete::tag, streaming::tag_no_case},
    combinator::map,
    multi::{many0, separated_list0, separated_list1},
    sequence::{delimited, preceded, tuple},
    IResult,
};

use crate::rfc3501::{
    core::{astring, atom, number64},
    mailbox::mailbox,
};

/// ```abnf
/// quota-root-name = astring
/// ```
#[inline]
pub fn quota_root_name(input: &[u8]) -> IResult<&[u8], AString> {
    astring(input)
}

/// ```abnf
/// getquota = "GETQUOTA" SP quota-root-name
/// ```
#[inline]
pub fn getquota(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case("GETQUOTA "), quota_root_name));

    let (remaining, (_, root)) = parser(input)?;

    Ok((remaining, CommandBody::GetQuota { root }))
}

/// ```abnf
/// getquotaroot = "GETQUOTAROOT" SP mailbox
/// ```
pub fn getquotaroot(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((tag_no_case("GETQUOTAROOT "), mailbox));

    let (remaining, (_, mailbox)) = parser(input)?;

    Ok((remaining, CommandBody::GetQuotaRoot { mailbox }))
}

/// ```abnf
/// quota-resource = resource-name SP
///                  resource-usage SP
///                  resource-limit
///
/// resource-usage = number64
///
/// resource-limit = number64
/// ```
pub fn quota_resource(input: &[u8]) -> IResult<&[u8], QuotaGet> {
    let mut parser = tuple((resource_name, SP, number64, SP, number64));

    let (remaining, (resource, _, usage, _, limit)) = parser(input)?;

    Ok((
        remaining,
        QuotaGet {
            resource,
            usage,
            limit,
        },
    ))
}

/// ```abnf
/// resource-name = "STORAGE" /
///                 "MESSAGE" /
///                 "MAILBOX" /
///                 "ANNOTATION-STORAGE" /
///                 resource-name-ext
///
/// resource-name-ext = atom
/// ```
pub fn resource_name(input: &[u8]) -> IResult<&[u8], Resource> {
    map(atom, Resource::from)(input)
}

/// ```abnf
/// quota-response = "QUOTA" SP quota-root-name SP quota-list
///
/// quota-list = "(" quota-resource *(SP quota-resource) ")"
/// ```
pub fn quota_response(input: &[u8]) -> IResult<&[u8], Data> {
    let mut parser = tuple((
        tag_no_case("QUOTA "),
        quota_root_name,
        SP,
        delimited(tag("("), separated_list1(SP, quota_resource), tag(")")),
    ));

    let (remaining, (_, root, _, quotas)) = parser(input)?;

    Ok((
        remaining,
        Data::Quota {
            root,
            // Safety: Safe because we use `separated_list1` above.
            quotas: NonEmptyVec::try_from(quotas).unwrap(),
        },
    ))
}

/// ```abnf
/// quotaroot-response = "QUOTAROOT" SP mailbox *(SP quota-root-name)
/// ```
pub fn quotaroot_response(input: &[u8]) -> IResult<&[u8], Data> {
    let mut parser = tuple((
        tag_no_case("QUOTAROOT "),
        mailbox,
        many0(preceded(SP, quota_root_name)),
    ));

    let (remaining, (_, mailbox, roots)) = parser(input)?;

    Ok((remaining, Data::QuotaRoot { mailbox, roots }))
}

/// ```abnf
/// setquota = "SETQUOTA" SP quota-root-name SP setquota-list
///
/// setquota-list = "(" [setquota-resource *(SP setquota-resource)] ")"
/// ```
pub fn setquota(input: &[u8]) -> IResult<&[u8], CommandBody> {
    let mut parser = tuple((
        tag_no_case("SETQUOTA "),
        quota_root_name,
        SP,
        delimited(tag("("), separated_list0(SP, setquota_resource), tag(")")),
    ));

    let (remaining, (_, root, _, quotas)) = parser(input)?;

    Ok((remaining, CommandBody::SetQuota { root, quotas }))
}

/// ```abnf
/// setquota-resource = resource-name SP resource-limit
/// ```
pub fn setquota_resource(input: &[u8]) -> IResult<&[u8], QuotaSet> {
    let mut parser = tuple((resource_name, SP, number64));

    let (remaining, (resource, _, limit)) = parser(input)?;

    Ok((remaining, QuotaSet { resource, limit }))
}

// This had to be inlined into the `capability` parser because `CapabilityOther("QUOTAFOO")` would
// be parsed as `Capability::Quota` plus an erroneous remainder. The `capability` parser eagerly consumes
// an `atom` and tries to detect the variants later.
// /// ```abnf
// /// capability-quota = "QUOTASET" / capa-quota-res
// /// ```
// ///
// /// Note: Extended to ...
// ///
// /// ```abnf
// /// capability-quota = "QUOTASET" / capa-quota-res / "QUOTA"
// /// ```
// pub fn capability_quota(input: &[u8]) -> IResult<&[u8], Capability> {
//     alt((
//         value(Capability::QuotaSet, tag_no_case("QUOTASET")),
//         capa_quota_res,
//         value(Capability::Quota, tag_no_case("QUOTA")),
//     ))(input)
// }

// /// ```abnf
// /// capa-quota-res = "QUOTA=RES-" resource-name
// /// ```
// pub fn capa_quota_res(input: &[u8]) -> IResult<&[u8], Capability> {
//     let mut parser = preceded(tag_no_case("QUOTA=RES-"), resource_name);
//
//     let (remaining, resource) = parser(input)?;
//
//     Ok((remaining, Capability::QuotaRes(resource)))
// }

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use imap_types::{
        core::IString,
        message::Tag,
        response::{
            data::{Capability, StatusAttributeValue},
            Code, Response, Status,
        },
    };

    use super::*;
    use crate::{
        codec::Decode,
        command::{status::StatusAttribute, Command, CommandBody},
        imap_types::extensions::rfc9208::ResourceOther,
        message::Mailbox,
    };

    #[test]
    fn test_trace_command() {
        let tests = [
            (
                "A001 SETQUOTA \"\" (STORAGE 512)\r\n",
                CommandBody::set_quota("", vec![QuotaSet::new(Resource::Storage, 512)])
                    .unwrap()
                    .tag("A001")
                    .unwrap(),
            ),
            (
                "A003 GETQUOTA \"\"\r\n",
                CommandBody::get_quota("").unwrap().tag("A003").unwrap(),
            ),
            (
                "A003 GETQUOTAROOT INBOX\r\n",
                CommandBody::get_quota_root(Mailbox::Inbox)
                    .unwrap()
                    .tag("A003")
                    .unwrap(),
            ),
            (
                "G0001 GETQUOTA \"!partition/sda4\"\r\n",
                CommandBody::get_quota(AString::String(
                    IString::try_from("!partition/sda4").unwrap(),
                ))
                .unwrap()
                .tag("G0001")
                .unwrap(),
            ),
            (
                "G0002 GETQUOTAROOT INBOX\r\n",
                CommandBody::get_quota_root("inbox")
                    .unwrap()
                    .tag("G0002")
                    .unwrap(),
            ),
            (
                "S0000 GETQUOTA \"#user/alice\"\r\n",
                CommandBody::get_quota(AString::String(IString::try_from("#user/alice").unwrap()))
                    .unwrap()
                    .tag("S0000")
                    .unwrap(),
            ),
            (
                "S0001 SETQUOTA \"#user/alice\" (STORAGE 510)\r\n",
                CommandBody::set_quota(
                    AString::String(IString::try_from("#user/alice").unwrap()),
                    vec![QuotaSet::new(Resource::Storage, 510)],
                )
                .unwrap()
                .tag("S0001")
                .unwrap(),
            ),
            (
                "S0002 SETQUOTA \"!partition/sda4\" (STORAGE 99999999)\r\n",
                CommandBody::set_quota(
                    AString::String(IString::try_from("!partition/sda4").unwrap()),
                    vec![QuotaSet::new(Resource::Storage, 99999999)],
                )
                .unwrap()
                .tag("S0002")
                .unwrap(),
            ),
            (
                "S0003 STATUS INBOX (MESSAGES DELETED DELETED-STORAGE)\r\n",
                CommandBody::status(
                    "inbox",
                    vec![
                        StatusAttribute::Messages,
                        StatusAttribute::Deleted,
                        StatusAttribute::DeletedStorage,
                    ],
                )
                .unwrap()
                .tag("S0003")
                .unwrap(),
            ),
        ];

        for (test, expected) in tests {
            let (got_rem, got_cmd) = Command::decode(test.as_bytes()).unwrap();
            assert!(got_rem.is_empty());
            assert_eq!(expected, got_cmd);
        }
    }

    #[test]
    fn test_trace_response() {
        let tests = [
            (
                "A003 NO [OVERQUOTA] APPEND Failed\r\n",
                Response::Status(
                    Status::no(
                        Some(Tag::try_from("A003").unwrap()),
                        Some(Code::OverQuota),
                        "APPEND Failed",
                    )
                    .unwrap(),
                ),
            ),
            (
                "* CAPABILITY QUOTA QUOTA=RES-STORAGE\r\n",
                Response::Data(
                    Data::capability(vec![
                        Capability::Quota,
                        Capability::QuotaRes(Resource::Storage),
                    ])
                    .unwrap(),
                ),
            ),
            (
                "* CAPABILITY QUOTA QUOTA=RES-STORAGE QUOTA=RES-MESSAGE\r\n",
                Response::Data(
                    Data::capability(vec![
                        Capability::Quota,
                        Capability::QuotaRes(Resource::Storage),
                        Capability::QuotaRes(Resource::Message),
                    ])
                    .unwrap(),
                ),
            ),
            (
                "* CAPABILITY QUOTA QUOTASET QUOTA=RES-STORAGE QUOTA=RES-MESSAGE\r\n",
                Response::Data(
                    Data::capability(vec![
                        Capability::Quota,
                        Capability::QuotaSet,
                        Capability::QuotaRes(Resource::Storage),
                        Capability::QuotaRes(Resource::Message),
                    ])
                    .unwrap(),
                ),
            ),
            (
                "* NO [OVERQUOTA] Soft quota has been exceeded\r\n",
                Response::Status(
                    Status::no(None, Some(Code::OverQuota), "Soft quota has been exceeded")
                        .unwrap(),
                ),
            ),
            (
                "* QUOTA \"!partition/sda4\" (STORAGE 104 10923847)\r\n",
                Response::Data(
                    Data::quota(
                        AString::String(IString::try_from("!partition/sda4").unwrap()),
                        vec![QuotaGet::new(Resource::Storage, 104, 10923847)],
                    )
                    .unwrap(),
                ),
            ),
            (
                "* QUOTAROOT comp.mail.mime\r\n",
                Response::Data(Data::QuotaRoot {
                    mailbox: Mailbox::try_from("comp.mail.mime").unwrap(),
                    roots: vec![],
                }),
            ),
            (
                "* QUOTAROOT INBOX \"\"\r\n",
                Response::Data(Data::QuotaRoot {
                    mailbox: Mailbox::Inbox,
                    roots: vec!["".try_into().unwrap()],
                }),
            ),
            (
                "* QUOTAROOT INBOX \"#user/alice\" \"!partition/sda4\"\r\n",
                Response::Data(Data::QuotaRoot {
                    mailbox: Mailbox::try_from("inbox").unwrap(),
                    roots: vec![
                        AString::String(IString::try_from("#user/alice").unwrap()),
                        AString::String(IString::try_from("!partition/sda4").unwrap()),
                    ],
                }),
            ),
            (
                "* QUOTA \"\" (STORAGE 10 512)\r\n",
                Response::Data(Data::Quota {
                    root: "".try_into().unwrap(),
                    quotas: vec![QuotaGet::new(Resource::Storage, 10, 512)]
                        .try_into()
                        .unwrap(),
                }),
            ),
            (
                "* QUOTA \"#user/alice\" (MESSAGE 42 1000)\r\n",
                Response::Data(Data::Quota {
                    root: AString::String(IString::try_from("#user/alice").unwrap()),
                    quotas: vec![QuotaGet::new(Resource::Message, 42, 1000)]
                        .try_into()
                        .unwrap(),
                }),
            ),
            (
                "* QUOTA \"#user/alice\" (STORAGE 54 111 MESSAGE 42 1000)\r\n",
                Response::Data(
                    Data::quota(
                        AString::String(IString::try_from("#user/alice").unwrap()),
                        vec![
                            QuotaGet::new(Resource::Storage, 54, 111),
                            QuotaGet::new(Resource::Message, 42, 1000),
                        ],
                    )
                    .unwrap(),
                ),
            ),
            (
                "* QUOTA \"#user/alice\" (STORAGE 58 512)\r\n",
                Response::Data(
                    Data::quota(
                        AString::String(IString::try_from("#user/alice").unwrap()),
                        vec![QuotaGet::new(Resource::Storage, 58, 512)],
                    )
                    .unwrap(),
                ),
            ),
            (
                "* STATUS INBOX (MESSAGES 12 DELETED 4 DELETED-STORAGE 8)\r\n",
                Response::Data(Data::Status {
                    mailbox: Mailbox::Inbox,
                    attributes: vec![
                        StatusAttributeValue::Messages(12),
                        StatusAttributeValue::Deleted(4),
                        StatusAttributeValue::DeletedStorage(8),
                    ],
                }),
            ),
        ];

        for (test, expected) in tests {
            let (got_rem, got_rsp) = Response::decode(test.as_bytes()).unwrap();
            println!("{expected:?} == {got_rsp:?}");
            assert!(got_rem.is_empty());
            assert_eq!(expected, got_rsp);
        }
    }

    #[test]
    fn test_resource() {
        let tests = [
            (b"stOragE ".as_ref(), Resource::Storage),
            (b"mesSaGe ".as_ref(), Resource::Message),
            (b"maIlbOx ".as_ref(), Resource::Mailbox),
            (b"anNotatIon-stoRage ".as_ref(), Resource::AnnotationStorage),
            (
                b"anNotatIon-stoRageX ".as_ref(),
                Resource::Other(ResourceOther::try_from(b"anNotatIon-stoRageX".as_ref()).unwrap()),
            ),
            (
                b"anNotatIon-stoRagee ".as_ref(),
                Resource::Other(ResourceOther::try_from(b"anNotatIon-stoRagee".as_ref()).unwrap()),
            ),
        ];

        for (test, expected) in tests.iter() {
            let (rem, got) = resource_name(test).unwrap();
            assert_eq!(*expected, got);
            assert_eq!(rem, b" ");
        }
    }
}
