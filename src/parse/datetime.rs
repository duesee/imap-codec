use abnf_core::streaming::{is_DIGIT, DQUOTE, SP};
use chrono::{DateTime, FixedOffset, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case, take_while_m_n},
    character::streaming::char,
    combinator::{map_res, recognize, value},
    error::ErrorKind,
    sequence::{delimited, tuple},
    IResult,
};
use std::str::from_utf8;

/// date = date-text / DQUOTE date-text DQUOTE
pub(crate) fn date(input: &[u8]) -> IResult<&[u8], Option<NaiveDate>> {
    alt((date_text, delimited(DQUOTE, date_text, DQUOTE)))(input)
}

/// date-text = date-day "-" date-month "-" date-year
fn date_text(input: &[u8]) -> IResult<&[u8], Option<NaiveDate>> {
    let parser = tuple((date_day, tag(b"-"), date_month, tag(b"-"), date_year));

    let (remaining, (d, _, m, _, y)) = parser(input)?;

    Ok((
        remaining,
        NaiveDate::from_ymd_opt(y.into(), m.into(), d.into()),
    ))
}

/// Day of month
///
/// date-day = 1*2DIGIT
fn date_day(input: &[u8]) -> IResult<&[u8], u8> {
    map_res(
        map_res(take_while_m_n(1, 2, is_DIGIT), from_utf8),
        str::parse::<u8>,
    )(input)
}

/// date-month = "Jan" / "Feb" / "Mar" / "Apr" /
///              "May" / "Jun" / "Jul" / "Aug" /
///              "Sep" / "Oct" / "Nov" / "Dec"
fn date_month(input: &[u8]) -> IResult<&[u8], u8> {
    alt((
        value(1, tag_no_case(b"Jan")),
        value(2, tag_no_case(b"Feb")),
        value(3, tag_no_case(b"Mar")),
        value(4, tag_no_case(b"Apr")),
        value(5, tag_no_case(b"May")),
        value(6, tag_no_case(b"Jun")),
        value(7, tag_no_case(b"Jul")),
        value(8, tag_no_case(b"Aug")),
        value(9, tag_no_case(b"Sep")),
        value(10, tag_no_case(b"Oct")),
        value(11, tag_no_case(b"Nov")),
        value(12, tag_no_case(b"Dec")),
    ))(input)
}

/// date-year = 4DIGIT
fn date_year(input: &[u8]) -> IResult<&[u8], u16> {
    map_res(
        map_res(take_while_m_n(4, 4, is_DIGIT), from_utf8),
        str::parse::<u16>,
    )(input)
}

/// Hours minutes seconds
///
/// time = 2DIGIT ":" 2DIGIT ":" 2DIGIT
fn time(input: &[u8]) -> IResult<&[u8], Option<NaiveTime>> {
    let parser = tuple((
        map_res(
            map_res(take_while_m_n(2, 2, is_DIGIT), from_utf8),
            str::parse::<u8>,
        ),
        tag(b":"),
        map_res(
            map_res(take_while_m_n(2, 2, is_DIGIT), from_utf8),
            str::parse::<u8>,
        ),
        tag(b":"),
        map_res(
            map_res(take_while_m_n(2, 2, is_DIGIT), from_utf8),
            str::parse::<u8>,
        ),
    ));

    let (remaining, (h, _, m, _, s)) = parser(input)?;

    Ok((
        remaining,
        NaiveTime::from_hms_opt(h.into(), m.into(), s.into()),
    ))
}

/// date-time = DQUOTE date-day-fixed "-" date-month "-" date-year SP time SP zone DQUOTE
pub(crate) fn date_time(input: &[u8]) -> IResult<&[u8], DateTime<FixedOffset>> {
    let parser = delimited(
        DQUOTE,
        tuple((
            date_day_fixed,
            tag(b"-"),
            date_month,
            tag(b"-"),
            date_year,
            SP,
            time,
            SP,
            zone,
        )),
        DQUOTE,
    );

    let (remaining, (d, _, m, _, y, _, time, _, zone)) = parser(input)?;

    let date = NaiveDate::from_ymd_opt(y.into(), m.into(), d.into());

    match (date, time, zone) {
        (Some(date), Some(time), Some(zone)) => {
            let local_datetime = NaiveDateTime::new(date, time);

            // Not sure about that...
            if let LocalResult::Single(datetime) = zone.from_local_datetime(&local_datetime) {
                Ok((remaining, datetime))
            } else {
                Err(nom::Err::Failure((remaining, ErrorKind::Verify))) // TODO(verify): use `Failure` or `Error`?
            }
        }
        _ => Err(nom::Err::Failure((remaining, ErrorKind::Verify))), // TODO(verify): use `Failure` or `Error`?
    }
}

/// Fixed-format version of date-day
///
/// date-day-fixed = (SP DIGIT) / 2DIGIT
fn date_day_fixed(input: &[u8]) -> IResult<&[u8], u8> {
    map_res(
        map_res(
            alt((
                recognize(tuple((SP, take_while_m_n(1, 1, is_DIGIT)))),
                take_while_m_n(2, 2, is_DIGIT),
            )),
            |bytes| from_utf8(bytes).map(|bytes| bytes.trim_start()),
        ),
        str::parse::<u8>,
    )(input)
}

/// Signed four-digit value of hhmm representing
/// hours and minutes east of Greenwich (that is,
/// the amount that the given time differs from
/// Universal Time).  Subtracting the timezone
/// from the given time will give the UT form.
/// The Universal Time zone is "+0000".
///
/// zone = ("+" / "-") 4DIGIT
fn zone(input: &[u8]) -> IResult<&[u8], Option<FixedOffset>> {
    let parser = tuple((
        alt((char('+'), char('-'))),
        map_res(
            map_res(take_while_m_n(2, 2, is_DIGIT), from_utf8),
            str::parse::<i32>,
        ),
        map_res(
            map_res(take_while_m_n(2, 2, is_DIGIT), from_utf8),
            str::parse::<i32>,
        ),
    ));

    let (remaining, (sign, hh, mm)) = parser(input)?;

    let offset = 3600 * hh + 60 * mm;

    let zone = match sign {
        '+' => FixedOffset::east_opt(offset),
        '-' => FixedOffset::west_opt(offset),
        _ => unreachable!(),
    };

    Ok((remaining, zone))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_date() {
        let (rem, val) = date(b"1-Feb-2020xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, NaiveDate::from_ymd_opt(2020, 2, 1));

        let (rem, val) = date(b"\"1-Feb-2020\"xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, NaiveDate::from_ymd_opt(2020, 2, 1));

        let (rem, val) = date(b"\"01-Feb-2020\"xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, NaiveDate::from_ymd_opt(2020, 2, 1));
    }

    #[test]
    fn test_date_text() {
        let (rem, val) = date_text(b"1-Feb-2020").unwrap();
        assert_eq!(rem, b"");
        assert_eq!(val, NaiveDate::from_ymd_opt(2020, 2, 1));
    }

    #[test]
    fn test_date_day() {
        let (rem, val) = date_day(b"1xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, 1);

        let (rem, val) = date_day(b"01xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, 1);

        let (rem, val) = date_day(b"999xxx").unwrap();
        assert_eq!(rem, b"9xxx");
        assert_eq!(val, 99);
    }

    #[test]
    fn test_date_month() {
        let (rem, val) = date_month(b"jAn").unwrap();
        assert_eq!(rem, b"");
        assert_eq!(val, 1);

        let (rem, val) = date_month(b"DeCxxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, 12);
    }

    #[test]
    fn test_date_year() {
        let (rem, val) = date_year(b"1985xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, 1985);

        let (rem, val) = date_year(b"1991xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, 1991);
    }

    #[test]
    fn test_date_day_fixed() {
        let (rem, val) = date_day_fixed(b"00").unwrap();
        assert_eq!(rem, b"");
        assert_eq!(val, 0);

        let (rem, val) = date_day_fixed(b" 0").unwrap();
        assert_eq!(rem, b"");
        assert_eq!(val, 0);

        let (rem, val) = date_day_fixed(b"99").unwrap();
        assert_eq!(rem, b"");
        assert_eq!(val, 99);

        let (rem, val) = date_day_fixed(b" 9").unwrap();
        assert_eq!(rem, b"");
        assert_eq!(val, 9);
    }

    #[test]
    fn test_time() {
        assert!(time(b"1:34:56xxx").is_err());
        assert!(time(b"12:3:56xxx").is_err());
        assert!(time(b"12:34:5xxx").is_err());

        let (rem, val) = time(b"12:34:56xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, NaiveTime::from_hms_opt(12, 34, 56));

        let (rem, val) = time(b"99:99:99 ").unwrap();
        assert_eq!(rem, b" ");
        assert_eq!(val, NaiveTime::from_hms_opt(99, 99, 99));

        let (rem, val) = time(b"12:34:56").unwrap();
        assert_eq!(rem, b"");
        assert_eq!(val, NaiveTime::from_hms_opt(12, 34, 56));

        let (rem, val) = time(b"99:99:99").unwrap();
        assert_eq!(rem, b"");
        assert_eq!(val, NaiveTime::from_hms_opt(99, 99, 99));
    }

    #[test]
    fn test_date_time() {
        let (rem, val) = date_time(b"\" 1-Feb-1985 12:34:56 +0100\"xxx").unwrap();
        assert_eq!(rem, b"xxx");

        let local_datetime = NaiveDateTime::new(
            NaiveDate::from_ymd(1985, 2, 1),
            NaiveTime::from_hms(12, 34, 56),
        );

        let datetime = FixedOffset::east(3600)
            .from_local_datetime(&local_datetime)
            .unwrap();

        println!("{} == \n{}", val, datetime);

        assert_eq!(val, datetime);
    }

    #[test]
    fn test_date_time_invalid() {
        let tests = [
            b"\" 1-Feb-0000 12:34:56 +0000\"xxx".as_ref(), // ok
            b"\" 1-Feb-9999 12:34:56 +0000\"xxx",          // ok
            b"\" 1-Feb-0000 12:34:56 -0000\"xxx",          // ok
            b"\" 1-Feb-9999 12:34:56 -0000\"xxx",          // ok
            b"\" 1-Feb-2020 00:00:00 +0100\"xxx",          // ok
            b"\" 1-Feb-0000 12:34:56 +9999\"xxx",
            b"\" 1-Feb-9999 12:34:56 +9999\"xxx",
            b"\" 1-Feb-0000 12:34:56 -9999\"xxx",
            b"\" 1-Feb-9999 12:34:56 -9999\"xxx",
            b"\" 1-Feb-2020 99:99:99 +0100\"xxx",
            b"\"31-Feb-2020 00:00:00 +0100\"xxx",
            b"\"99-Feb-2020 99:99:99 +0100\"xxx",
        ];

        for test in &tests[..5] {
            let (rem, datetime) = date_time(test).unwrap();
            assert_eq!(rem, b"xxx");
            println!("{} -> {:?}", std::str::from_utf8(test).unwrap(), datetime);
        }

        for test in &tests[5..] {
            assert!(date_time(test).is_err());
        }
    }

    #[test]
    fn test_zone() {
        let (rem, val) = zone(b"+0000xxx").unwrap();
        eprintln!("{:?}", val);
        assert_eq!(rem, b"xxx");
        assert_eq!(val, FixedOffset::east_opt(0));

        let (rem, val) = zone(b"+0000").unwrap();
        eprintln!("{:?}", val);
        assert_eq!(rem, b"");
        assert_eq!(val, FixedOffset::east_opt(0));

        let (rem, val) = zone(b"-0205xxx").unwrap();
        eprintln!("{:?}", val);
        assert_eq!(rem, b"xxx");
        assert_eq!(val, FixedOffset::west_opt(2 * 3600 + 5 * 60));

        let (rem, val) = zone(b"-1159").unwrap();
        eprintln!("{:?}", val);
        assert_eq!(rem, b"");
        assert_eq!(val, FixedOffset::west_opt(11 * 3600 + 59 * 60));

        let (rem, val) = zone(b"-1159").unwrap();
        eprintln!("{:?}", val);
        assert_eq!(rem, b"");
        assert_eq!(val, FixedOffset::west_opt(11 * 3600 + 59 * 60));
    }
}
