use abnf_core::{
    is_digit,
    streaming::{dquote, sp},
};
use imap_types::datetime::{DateTime, NaiveDate};
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case, take_while_m_n},
    character::streaming::char,
    combinator::{map, map_res, value},
    sequence::{delimited, preceded, tuple},
};

use crate::decode::{IMAPErrorKind, IMAPParseError, IMAPResult};

/// ```abnf
/// date = date-text / DQUOTE date-text DQUOTE
/// ```
pub(crate) fn date(input: &[u8]) -> IMAPResult<&[u8], Option<NaiveDate>> {
    alt((date_text, delimited(dquote, date_text, dquote)))(input)
}

/// ```abnf
/// date-text = date-day "-" date-month "-" date-year
/// ```
pub(crate) fn date_text(input: &[u8]) -> IMAPResult<&[u8], Option<NaiveDate>> {
    let mut parser = tuple((date_day, tag(b"-"), date_month, tag(b"-"), date_year));

    let (remaining, (d, _, m, _, y)) = parser(input)?;

    Ok((remaining, NaiveDate::new(y, m, d).ok()))
}

/// Day of month.
///
/// ```abnf
/// date-day = 1*2DIGIT
/// ```
pub(crate) fn date_day(input: &[u8]) -> IMAPResult<&[u8], u8> {
    digit_1_2(input)
}

/// Month names as encoded in `date-month`, indexed by `month - 1`.
pub(crate) const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// ```abnf
/// date-month = "Jan" / "Feb" / "Mar" / "Apr" /
///              "May" / "Jun" / "Jul" / "Aug" /
///              "Sep" / "Oct" / "Nov" / "Dec"
/// ```
pub(crate) fn date_month(input: &[u8]) -> IMAPResult<&[u8], u8> {
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

/// ```abnf
/// date-year = 4DIGIT
/// ```
pub(crate) fn date_year(input: &[u8]) -> IMAPResult<&[u8], u16> {
    digit_4(input)
}

/// Hours minutes seconds.
///
/// ```abnf
/// time = 2DIGIT ":" 2DIGIT ":" 2DIGIT
/// ```
///
/// Values are not range-checked here, see [`DateTime::new`].
pub(crate) fn time(input: &[u8]) -> IMAPResult<&[u8], (u8, u8, u8)> {
    let mut parser = tuple((digit_2, tag(b":"), digit_2, tag(b":"), digit_2));

    let (remaining, (h, _, m, _, s)) = parser(input)?;

    Ok((remaining, (h, m, s)))
}

/// ```abnf
/// date-time = DQUOTE
///              date-day-fixed "-" date-month "-" date-year SP
///              time SP
///              zone
///             DQUOTE
/// ```
pub(crate) fn date_time(input: &[u8]) -> IMAPResult<&[u8], DateTime> {
    let mut parser = delimited(
        dquote,
        tuple((
            date_day_fixed,
            tag(b"-"),
            date_month,
            tag(b"-"),
            date_year,
            sp,
            time,
            sp,
            zone,
        )),
        dquote,
    );

    let (remaining, (d, _, m, _, y, _, (hour, minute, second), _, zone)) = parser(input)?;

    let datetime = zone.and_then(|offset| {
        let date = NaiveDate::new(y, m, d).ok()?;
        DateTime::new(date, hour, minute, second, offset).ok()
    });

    match datetime {
        Some(datetime) => Ok((remaining, datetime)),
        None => Err(nom::Err::Failure(IMAPParseError {
            input,
            kind: IMAPErrorKind::BadDateTime,
        })),
    }
}

/// Fixed-format version of date-day.
///
/// ```abnf
/// date-day-fixed = (SP DIGIT) / 2DIGIT
/// ```
pub(crate) fn date_day_fixed(input: &[u8]) -> IMAPResult<&[u8], u8> {
    alt((
        map(
            preceded(sp, take_while_m_n(1, 1, is_digit)),
            |bytes: &[u8]| bytes[0] - b'0',
        ),
        digit_2,
    ))(input)
}

/// Signed four-digit value of hhmm representing hours and minutes east of Greenwich (that is, the
/// amount that the given time differs from Universal Time).
///
/// Subtracting the timezone from the given time will give the UT form. The Universal Time zone is
/// "+0000".
///
/// ```abnf
/// zone = ("+" / "-") 4DIGIT
/// ```
///
/// Returns the offset in minutes east of UTC, or `None` when `mm` is not a
/// valid minute. Hours are range-checked by [`DateTime::new`].
pub(crate) fn zone(input: &[u8]) -> IMAPResult<&[u8], Option<i16>> {
    let mut parser = tuple((alt((char('+'), char('-'))), digit_2, digit_2));

    let (remaining, (sign, hh, mm)) = parser(input)?;

    if mm > 59 {
        return Ok((remaining, None));
    }

    let offset = i16::from(hh) * 60 + i16::from(mm);

    let zone = match sign {
        '+' => offset,
        '-' => -offset,
        _ => unreachable!(),
    };

    Ok((remaining, Some(zone)))
}

fn digit_1_2(input: &[u8]) -> IMAPResult<&[u8], u8> {
    map_res(
        map(take_while_m_n(1, 2, is_digit), |bytes| {
            // # Safety
            //
            // `bytes` is always UTF-8.
            core::str::from_utf8(bytes).unwrap()
        }),
        str::parse::<u8>,
    )(input)
}

fn digit_2(input: &[u8]) -> IMAPResult<&[u8], u8> {
    map_res(
        map(take_while_m_n(2, 2, is_digit), |bytes| {
            // # Safety
            //
            // `bytes` is always UTF-8.
            core::str::from_utf8(bytes).unwrap()
        }),
        str::parse::<u8>,
    )(input)
}

fn digit_4(input: &[u8]) -> IMAPResult<&[u8], u16> {
    map_res(
        map(take_while_m_n(4, 4, is_digit), |bytes| {
            // # Safety
            //
            // `bytes` is always UTF-8.
            core::str::from_utf8(bytes).unwrap()
        }),
        str::parse::<u16>,
    )(input)
}

#[cfg(test)]
mod tests {
    use core::str::from_utf8;
    use std::{eprintln, println};

    use super::*;
    use crate::testing::known_answer_test_encode;

    fn date_time_of(y: u16, mo: u8, d: u8, h: u8, mi: u8, s: u8, offset: i16) -> DateTime {
        DateTime::new(NaiveDate::new(y, mo, d).unwrap(), h, mi, s, offset).unwrap()
    }

    #[test]
    fn test_encode_naive_date() {
        let tests = [
            (
                NaiveDate::new(2023, 4, 12).unwrap(),
                b"\"12-Apr-2023\"".as_ref(),
            ),
            (
                NaiveDate::new(7, 1, 1).unwrap(),
                b"\"01-Jan-0007\"".as_ref(),
            ),
            (
                NaiveDate::new(9999, 12, 31).unwrap(),
                b"\"31-Dec-9999\"".as_ref(),
            ),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }

    #[test]
    fn test_date_leap_day() {
        let (_, val) = date(b"29-Feb-2020").unwrap();
        assert_eq!(val, NaiveDate::new(2020, 2, 29).ok());

        let (_, val) = date(b"29-Feb-2019").unwrap();
        assert_eq!(val, None);
    }

    #[test]
    fn test_date_time_zone_bounds() {
        assert!(date_time(b"\" 1-Feb-2020 12:34:56 +2400\"xxx").is_err());
        assert_eq!(
            date_time(b"\" 1-Feb-2020 12:34:56 -2359\"xxx").unwrap().1,
            date_time_of(2020, 2, 1, 12, 34, 56, -1439)
        );
    }

    #[test]
    fn test_encode_date_time() {
        let tests = [
            (
                date_time_of(1994, 2, 7, 21, 52, 25, -480),
                b"\"07-Feb-1994 21:52:25 -0800\"".as_ref(),
            ),
            (
                date_time_of(0, 2, 7, 21, 52, 25, -480),
                b"\"07-Feb-0000 21:52:25 -0800\"".as_ref(),
            ),
            (
                date_time_of(2020, 12, 31, 0, 0, 0, 0),
                b"\"31-Dec-2020 00:00:00 +0000\"".as_ref(),
            ),
            (
                date_time_of(2020, 1, 1, 23, 59, 59, 1439),
                b"\"01-Jan-2020 23:59:59 +2359\"".as_ref(),
            ),
            (
                date_time_of(2020, 1, 1, 0, 0, 0, -1),
                b"\"01-Jan-2020 00:00:00 -0001\"".as_ref(),
            ),
        ];

        for test in tests {
            known_answer_test_encode(test);
        }
    }

    #[test]
    fn test_date() {
        let (rem, val) = date(b"1-Feb-2020xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, NaiveDate::new(2020, 2, 1).ok());

        let (rem, val) = date(b"\"1-Feb-2020\"xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, NaiveDate::new(2020, 2, 1).ok());

        let (rem, val) = date(b"\"01-Feb-2020\"xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, NaiveDate::new(2020, 2, 1).ok());
    }

    #[test]
    fn test_date_text() {
        let (rem, val) = date_text(b"1-Feb-2020").unwrap();
        assert_eq!(rem, b"");
        assert_eq!(val, NaiveDate::new(2020, 2, 1).ok());
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
        assert_eq!(val, (12, 34, 56));

        let (rem, val) = time(b"99:99:99 ").unwrap();
        assert_eq!(rem, b" ");
        assert_eq!(val, (99, 99, 99));

        let (rem, val) = time(b"12:34:56").unwrap();
        assert_eq!(rem, b"");
        assert_eq!(val, (12, 34, 56));

        let (rem, val) = time(b"99:99:99").unwrap();
        assert_eq!(rem, b"");
        assert_eq!(val, (99, 99, 99));
    }

    #[test]
    fn test_date_time() {
        let (rem, val) = date_time(b"\" 1-Feb-1985 12:34:56 +0100\"xxx").unwrap();
        assert_eq!(rem, b"xxx");

        let datetime = date_time_of(1985, 2, 1, 12, 34, 56, 60);

        println!("{val:?} == \n{datetime:?}");

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
            b"\" 1-Feb-2020 12:34:56 +0060\"xxx",
            b"\" 1-Feb-2020 12:34:56 +0099\"xxx",
            b"\" 1-Feb-2020 12:34:56 -0099\"xxx",
        ];

        for test in &tests[..5] {
            let (rem, datetime) = date_time(test).unwrap();
            assert_eq!(rem, b"xxx");
            println!("{} -> {:?}", from_utf8(test).unwrap(), datetime);
        }

        for test in &tests[5..] {
            assert!(date_time(test).is_err());
        }
    }

    #[test]
    fn test_zone() {
        assert_eq!(zone(b"+0060").unwrap().1, None);
        assert_eq!(zone(b"-0099").unwrap().1, None);

        let (rem, val) = zone(b"+0000xxx").unwrap();
        eprintln!("{val:?}");
        assert_eq!(rem, b"xxx");
        assert_eq!(val, Some(0));

        let (rem, val) = zone(b"+0000").unwrap();
        eprintln!("{val:?}");
        assert_eq!(rem, b"");
        assert_eq!(val, Some(0));

        let (rem, val) = zone(b"-0205xxx").unwrap();
        eprintln!("{val:?}");
        assert_eq!(rem, b"xxx");
        assert_eq!(val, Some(-(2 * 60 + 5)));

        let (rem, val) = zone(b"-1159").unwrap();
        eprintln!("{val:?}");
        assert_eq!(rem, b"");
        assert_eq!(val, Some(-(11 * 60 + 59)));

        let (rem, val) = zone(b"-1159").unwrap();
        eprintln!("{val:?}");
        assert_eq!(rem, b"");
        assert_eq!(val, Some(-(11 * 60 + 59)));
    }
}
