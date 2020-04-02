use crate::parse::{digit, dquote, sp};
use chrono::NaiveDate;
use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    combinator::{map, value},
    multi::{count, many_m_n},
    sequence::tuple,
    IResult,
};
use std::str::FromStr;

/// date = date-text / DQUOTE date-text DQUOTE
pub fn date(input: &[u8]) -> IResult<&[u8], NaiveDate> {
    let parser = alt((
        date_text,
        map(tuple((dquote, date_text, dquote)), |(_, date_text, _)| {
            date_text
        }),
    ));

    let (remaining, parsed_date) = parser(input)?;

    Ok((remaining, parsed_date))
}

/// date-text = date-day "-" date-month "-" date-year
pub fn date_text(input: &[u8]) -> IResult<&[u8], NaiveDate> {
    let parser = tuple((
        date_day,
        tag_no_case(b"-"),
        date_month,
        tag_no_case(b"-"),
        date_year,
    ));

    let (remaining, (d, _, m, _, y)) = parser(input)?;

    Ok((remaining, NaiveDate::from_ymd(y.into(), m.into(), d.into())))
}

/// date-day = 1*2DIGIT ; Day of month
pub fn date_day(input: &[u8]) -> IResult<&[u8], u8> {
    let parser = many_m_n(1, 2, digit);

    let (remaining, parsed_date_day) = parser(input)?;

    Ok((
        remaining,
        u8::from_str(&String::from_utf8(parsed_date_day).unwrap()).unwrap(),
    ))
}

/// date-month = "Jan" / "Feb" / "Mar" / "Apr" / "May" / "Jun" / "Jul" / "Aug" / "Sep" / "Oct" / "Nov" / "Dec"
pub fn date_month(input: &[u8]) -> IResult<&[u8], u8> {
    let parser = alt((
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
    ));

    let (remaining, parsed_date_month) = parser(input)?;

    Ok((remaining, parsed_date_month))
}

/// date-year = 4DIGIT
pub fn date_year(input: &[u8]) -> IResult<&[u8], u16> {
    let parser = count(digit, 4);

    let (remaining, parsed_date_year) = parser(input)?;

    Ok((
        remaining,
        u16::from_str(&String::from_utf8(parsed_date_year).unwrap()).unwrap(),
    ))
}

/// time = 2DIGIT ":" 2DIGIT ":" 2DIGIT ; Hours minutes seconds
pub fn time(input: &[u8]) -> IResult<&[u8], (u8, u8, u8)> {
    let parser = tuple((
        map(count(digit, 2), |h| {
            u8::from_str(&String::from_utf8(h).unwrap()).unwrap()
        }),
        tag(b":"),
        map(count(digit, 2), |m| {
            u8::from_str(&String::from_utf8(m).unwrap()).unwrap()
        }),
        tag(b":"),
        map(count(digit, 2), |s| {
            u8::from_str(&String::from_utf8(s).unwrap()).unwrap()
        }),
    ));

    let (remaining, (h, _, m, _, s)) = parser(input)?;

    Ok((remaining, (h, m, s)))
}

pub type DateTime = ((u8, u8, u16), (u8, u8, u8), String);

/// date-time = DQUOTE date-day-fixed "-" date-month "-" date-year SP time SP zone DQUOTE
pub fn date_time(input: &[u8]) -> IResult<&[u8], ((u8, u8, u16), (u8, u8, u8), String)> {
    let parser = tuple((
        dquote,
        date_day_fixed,
        tag(b"-"),
        date_month,
        tag(b"-"),
        date_year,
        sp,
        time,
        sp,
        zone,
        dquote,
    ));

    let (remaining, (_, d, _, m, _, y, _, time, _, zone, _)) = parser(input)?;

    Ok((remaining, ((d, m, y), time, zone)))
}

/// date-day-fixed = (SP DIGIT) / 2DIGIT ; Fixed-format version of date-day
pub fn date_day_fixed(input: &[u8]) -> IResult<&[u8], u8> {
    let parser = alt((
        map(tuple((sp, digit)), |(_, day)| {
            u8::from_str(&String::from_utf8(vec![day]).unwrap()).unwrap()
        }),
        map(count(digit, 2), |raw| {
            u8::from_str(&String::from_utf8(raw).unwrap()).unwrap()
        }),
    ));

    let (remaining, parsed_date_day_fixed) = parser(input)?;

    Ok((remaining, parsed_date_day_fixed))
}

/// zone = ("+" / "-") 4DIGIT
///          ; Signed four-digit value of hhmm representing
///          ; hours and minutes east of Greenwich (that is,
///          ; the amount that the given time differs from
///          ; Universal Time).  Subtracting the timezone
///          ; from the given time will give the UT form.
///          ; The Universal Time zone is "+0000".
pub fn zone(input: &[u8]) -> IResult<&[u8], String> {
    let parser = tuple((
        alt((
            value(String::from("+"), tag(b"+")),
            value(String::from("-"), tag(b"-")),
        )),
        map(count(digit, 4), |raw| String::from_utf8(raw).unwrap()),
    ));

    let (remaining, (sign, hhmm)) = parser(input)?;

    Ok((remaining, sign + &hhmm))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_date() {
        let (rem, val) = date(b"1-Feb-2020xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, NaiveDate::from_ymd(2020, 2, 1));

        let (rem, val) = date(b"\"1-Feb-2020\"xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, NaiveDate::from_ymd(2020, 2, 1));

        let (rem, val) = date(b"\"01-Feb-2020\"xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, NaiveDate::from_ymd(2020, 2, 1));
    }

    #[test]
    fn test_date_text() {
        let (rem, val) = date_text(b"1-Feb-2020").unwrap();
        assert_eq!(rem, b"");
        assert_eq!(val, NaiveDate::from_ymd(2020, 2, 1));
    }

    #[test]
    fn test_date_day() {
        let (rem, val) = date_day(b"1xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, 1);

        let (rem, val) = date_day(b"01xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, 1);
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
        let (rem, val) = time(b"00:00:00xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, (0, 0, 0));

        let (rem, val) = time(b"99:99:99 ").unwrap();
        assert_eq!(rem, b" ");
        assert_eq!(val, (99, 99, 99));

        let (rem, val) = time(b"00:00:00").unwrap();
        assert_eq!(rem, b"");
        assert_eq!(val, (0, 0, 0));

        let (rem, val) = time(b"99:99:99").unwrap();
        assert_eq!(rem, b"");
        assert_eq!(val, (99, 99, 99));
    }

    #[test]
    fn test_date_time() {
        let (rem, val) = date_time(b"\" 1-Feb-1985 12:34:56 +0100\"xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, ((1, 2, 1985), (12, 34, 56), String::from("+0100")));
    }

    #[test]
    fn test_zone() {
        let (rem, val) = zone(b"+0000xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, "+0000");

        let (rem, val) = zone(b"+0000").unwrap();
        assert_eq!(rem, b"");
        assert_eq!(val, "+0000");

        let (rem, val) = zone(b"-9999xxx").unwrap();
        assert_eq!(rem, b"xxx");
        assert_eq!(val, "-9999");

        let (rem, val) = zone(b"-9999").unwrap();
        assert_eq!(rem, b"");
        assert_eq!(val, "-9999");
    }
}
