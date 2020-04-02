use crate::{
    parse::{
        core::{number, nz_number},
        sp,
    },
    types::{command::StatusItem, response::StatusItemResp},
};
use nom::{
    branch::alt,
    bytes::streaming::tag_no_case,
    combinator::{map, value},
    multi::separated_nonempty_list,
    sequence::tuple,
    IResult,
};

/// status-att = "MESSAGES" / "RECENT" / "UIDNEXT" / "UIDVALIDITY" / "UNSEEN"
pub fn status_att(input: &[u8]) -> IResult<&[u8], StatusItem> {
    let parser = alt((
        value(StatusItem::Messages, tag_no_case(b"MESSAGES")),
        value(StatusItem::Recent, tag_no_case(b"RECENT")),
        value(StatusItem::UidNext, tag_no_case(b"UIDNEXT")),
        value(StatusItem::UidValidity, tag_no_case(b"UIDVALIDITY")),
        value(StatusItem::Unseen, tag_no_case(b"UNSEEN")),
    ));

    let (remaining, parsed_status_att) = parser(input)?;

    Ok((remaining, parsed_status_att))
}

/// ; errata id: 261
/// status-att-list = status-att-val *(SP status-att-val)
pub fn status_att_list(input: &[u8]) -> IResult<&[u8], Vec<StatusItemResp>> {
    let parser = separated_nonempty_list(sp, status_att_val);

    let (remaining, parsed_status_att_list) = parser(input)?;

    Ok((remaining, parsed_status_att_list))
}

/// ; errata id: 261
/// status-att-val  = ("MESSAGES" SP number) / ("RECENT" SP number) /
///                   ("UIDNEXT" SP nz-number) / ("UIDVALIDITY" SP nz-number) /
///                   ("UNSEEN" SP number)
pub fn status_att_val(input: &[u8]) -> IResult<&[u8], StatusItemResp> {
    let parser = alt((
        map(
            tuple((tag_no_case(b"MESSAGES"), sp, number)),
            |(_, _, num)| StatusItemResp::Messages(num),
        ),
        map(
            tuple((tag_no_case(b"RECENT"), sp, number)),
            |(_, _, num)| StatusItemResp::Recent(num),
        ),
        map(
            tuple((tag_no_case(b"UIDNEXT"), sp, nz_number)),
            |(_, _, next)| StatusItemResp::UidNext(next),
        ),
        map(
            tuple((tag_no_case(b"UIDVALIDITY"), sp, nz_number)),
            |(_, _, val)| StatusItemResp::UidValidity(val),
        ),
        map(
            tuple((tag_no_case(b"UNSEEN"), sp, number)),
            |(_, _, num)| StatusItemResp::Unseen(num),
        ),
    ));

    let (remaining, parsed_status_att_val) = parser(input)?;

    Ok((remaining, parsed_status_att_val))
}
