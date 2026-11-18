//! Date and time-related types.

use core::fmt::{Debug, Formatter};

use bounded_static::{IntoBoundedStatic, ToBoundedStatic};
use chrono::{Datelike, FixedOffset, TimeZone, Timelike};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::datetime::error::{DateTimeError, NaiveDateError};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "chrono::DateTime<FixedOffset>"))]
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct DateTime(chrono::DateTime<FixedOffset>);

impl DateTime {
    pub fn validate(value: &chrono::DateTime<FixedOffset>) -> Result<(), DateTimeError> {
        // Only a subset of `chrono`s `DateTime<FixedOffset>` is valid in IMAP.
        if !(0..=9999).contains(&value.year()) {
            return Err(DateTimeError::YearOutOfRange { got: value.year() });
        }

        if value.timestamp_subsec_nanos() != 0 {
            return Err(DateTimeError::UnalignedNanoSeconds {
                got: value.timestamp_subsec_nanos(),
            });
        }

        if value.offset().local_minus_utc() % 60 != 0 {
            return Err(DateTimeError::UnalignedOffset {
                got: value.offset().local_minus_utc() % 60,
            });
        }

        Ok(())
    }

    /// Validates the time and the offset (in minutes east of UTC).
    fn validate_fields(
        hour: u8,
        minute: u8,
        second: u8,
        offset_minutes: i16,
    ) -> Result<(), DateTimeError> {
        if hour > 23 {
            return Err(DateTimeError::HourOutOfRange { got: hour });
        }

        if minute > 59 {
            return Err(DateTimeError::MinuteOutOfRange { got: minute });
        }

        if second > 59 {
            return Err(DateTimeError::SecondOutOfRange { got: second });
        }

        if !(-MAX_OFFSET_MINUTES..=MAX_OFFSET_MINUTES).contains(&offset_minutes) {
            return Err(DateTimeError::OffsetOutOfRange {
                got: offset_minutes,
            });
        }

        Ok(())
    }

    /// Constructs a date time without validation.
    ///
    /// # Warning: IMAP conformance
    ///
    /// The caller must ensure that `value` is valid according to [`Self::validate`]. Failing to do
    /// so may create invalid/unparsable IMAP messages, or even produce unintended protocol flows.
    /// Do not call this constructor with untrusted data.
    ///
    /// Note: This method will `panic!` on wrong input in debug builds.
    pub fn unvalidated(value: chrono::DateTime<FixedOffset>) -> Self {
        #[cfg(debug_assertions)]
        Self::validate(&value).unwrap();

        Self(value)
    }

    /// Constructs a date time, validating that it fits into IMAP.
    ///
    /// `offset_minutes` is the offset east of UTC, in minutes, e.g., `-480`
    /// for `-0800`.
    pub fn new(
        date: NaiveDate,
        hour: u8,
        minute: u8,
        second: u8,
        offset_minutes: i16,
    ) -> Result<Self, DateTimeError> {
        Self::validate_fields(hour, minute, second, offset_minutes)?;

        let offset = FixedOffset::east_opt(i32::from(offset_minutes) * 60)
            .expect("offset should be in range");
        let local = date
            .0
            .and_hms_opt(hour.into(), minute.into(), second.into())
            .expect("time should be in range");
        let datetime = offset
            .from_local_datetime(&local)
            .single()
            .expect("fixed offsets should be unambiguous");

        Ok(Self(datetime))
    }

    /// Local date.
    pub fn date(&self) -> NaiveDate {
        NaiveDate(self.0.date_naive())
    }

    /// Local hour, in `0..=23`.
    pub fn hour(&self) -> u8 {
        self.0.hour() as u8
    }

    /// Local minute, in `0..=59`.
    pub fn minute(&self) -> u8 {
        self.0.minute() as u8
    }

    /// Local second, in `0..=59`.
    pub fn second(&self) -> u8 {
        self.0.second() as u8
    }

    /// Offset east of UTC, in minutes, in `-1439..=1439`.
    pub fn offset_minutes(&self) -> i16 {
        (self.0.offset().local_minus_utc() / 60) as i16
    }
}

impl TryFrom<chrono::DateTime<FixedOffset>> for DateTime {
    type Error = DateTimeError;

    fn try_from(value: chrono::DateTime<FixedOffset>) -> Result<Self, Self::Error> {
        Self::validate(&value)?;

        Ok(Self(value))
    }
}

impl Debug for DateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl AsRef<chrono::DateTime<FixedOffset>> for DateTime {
    fn as_ref(&self) -> &chrono::DateTime<FixedOffset> {
        &self.0
    }
}

impl IntoBoundedStatic for DateTime {
    type Static = Self;

    fn into_static(self) -> Self::Static {
        self
    }
}

impl ToBoundedStatic for DateTime {
    type Static = Self;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "chrono::NaiveDate"))]
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct NaiveDate(chrono::NaiveDate);

impl NaiveDate {
    pub fn validate(value: &chrono::NaiveDate) -> Result<(), NaiveDateError> {
        // Only a subset of `chrono`s `NaiveDate` is valid in IMAP.
        if !(0..=9999).contains(&value.year()) {
            return Err(NaiveDateError::YearOutOfRange { got: value.year() });
        }

        Ok(())
    }

    /// Validates that the date exists and that its year has four digits.
    fn validate_fields(year: u16, month: u8, day: u8) -> Result<(), NaiveDateError> {
        if year > 9999 {
            return Err(NaiveDateError::YearOutOfRange { got: year.into() });
        }

        if !(1..=12).contains(&month) {
            return Err(NaiveDateError::MonthOutOfRange { got: month });
        }

        let max = days_in_month(year, month);

        if !(1..=max).contains(&day) {
            return Err(NaiveDateError::DayOutOfRange { got: day, max });
        }

        Ok(())
    }

    /// Constructs a naive date without validation.
    ///
    /// # Warning: IMAP conformance
    ///
    /// The caller must ensure that `value` is valid according to [`Self::validate`]. Failing to do
    /// so may create invalid/unparsable IMAP messages, or even produce unintended protocol flows.
    /// Do not call this constructor with untrusted data.
    ///
    /// Note: This method will `panic!` on wrong input in debug builds.
    pub fn unvalidated(value: chrono::NaiveDate) -> Self {
        #[cfg(debug_assertions)]
        Self::validate(&value).unwrap();

        Self(value)
    }

    /// Constructs a date, validating that it exists and fits into IMAP.
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self, NaiveDateError> {
        Self::validate_fields(year, month, day)?;

        let date = chrono::NaiveDate::from_ymd_opt(year.into(), month.into(), day.into())
            .expect("date should be valid");

        Ok(Self(date))
    }

    /// Year, in `0..=9999`.
    pub fn year(&self) -> u16 {
        self.0.year() as u16
    }

    /// Month, in `1..=12`.
    pub fn month(&self) -> u8 {
        self.0.month() as u8
    }

    /// Day of month, in `1..=31`.
    pub fn day(&self) -> u8 {
        self.0.day() as u8
    }
}

impl TryFrom<chrono::NaiveDate> for NaiveDate {
    type Error = NaiveDateError;

    fn try_from(value: chrono::NaiveDate) -> Result<Self, Self::Error> {
        Self::validate(&value)?;

        Ok(Self(value))
    }
}

impl Debug for NaiveDate {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl AsRef<chrono::NaiveDate> for NaiveDate {
    fn as_ref(&self) -> &chrono::NaiveDate {
        &self.0
    }
}

impl IntoBoundedStatic for NaiveDate {
    type Static = Self;

    fn into_static(self) -> Self::Static {
        self
    }
}

impl ToBoundedStatic for NaiveDate {
    type Static = Self;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}

/// Largest UTC offset (in minutes) representable in IMAP's `zone`.
///
/// IMAP encodes the zone as `("+" / "-") hhmm`. Hours are limited to `23`
/// and minutes to `59`, i.e., the offset is strictly less than one day.
const MAX_OFFSET_MINUTES: i16 = 23 * 60 + 59;

fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Error-related types.
pub mod error {
    use thiserror::Error;

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum DateTimeError {
        #[error("expected `0 <= year <= 9999`, got {got}")]
        YearOutOfRange { got: i32 },
        #[error("expected `nanos == 0`, got {got}")]
        UnalignedNanoSeconds { got: u32 },
        #[error("expected `offset % 60 == 0`, got {got}")]
        UnalignedOffset { got: i32 },
        #[error(transparent)]
        Date(#[from] NaiveDateError),
        #[error("expected `hour <= 23`, got {got}")]
        HourOutOfRange { got: u8 },
        #[error("expected `minute <= 59`, got {got}")]
        MinuteOutOfRange { got: u8 },
        #[error("expected `second <= 59`, got {got}")]
        SecondOutOfRange { got: u8 },
        #[error("expected `-1439 <= offset <= 1439` (minutes), got {got}")]
        OffsetOutOfRange { got: i16 },
    }

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum NaiveDateError {
        #[error("expected `0 <= year <= 9999`, got {got}")]
        YearOutOfRange { got: i32 },
        #[error("expected `1 <= month <= 12`, got {got}")]
        MonthOutOfRange { got: u8 },
        #[error("expected `1 <= day <= {max}`, got {got}")]
        DayOutOfRange { got: u8, max: u8 },
    }
}

#[cfg(test)]
mod tests {
    use alloc::format;
    #[cfg(feature = "serde")]
    use alloc::string::ToString;
    use std::println;

    use chrono::{TimeZone, Timelike};

    use super::*;

    fn date(year: u16, month: u8, day: u8) -> NaiveDate {
        NaiveDate::new(year, month, day).unwrap()
    }

    #[test]
    fn test_new_date_time_failing() {
        let tests = [
            (
                DateTime::new(date(2020, 2, 1), 24, 0, 0, 0),
                DateTimeError::HourOutOfRange { got: 24 },
            ),
            (
                DateTime::new(date(2020, 2, 1), 0, 60, 0, 0),
                DateTimeError::MinuteOutOfRange { got: 60 },
            ),
            (
                DateTime::new(date(2020, 2, 1), 0, 0, 60, 0),
                DateTimeError::SecondOutOfRange { got: 60 },
            ),
            (
                DateTime::new(date(2020, 2, 1), 0, 0, 0, 1440),
                DateTimeError::OffsetOutOfRange { got: 1440 },
            ),
            (
                DateTime::new(date(2020, 2, 1), 0, 0, 0, -1440),
                DateTimeError::OffsetOutOfRange { got: -1440 },
            ),
        ];

        for (got, expected) in tests {
            println!("{}", got.clone().unwrap_err());
            println!("{:?}", got.clone().unwrap_err());
            assert_eq!(expected, got.unwrap_err());
        }

        assert!(DateTime::new(date(2020, 2, 1), 23, 59, 59, 1439).is_ok());
        assert!(DateTime::new(date(2020, 2, 1), 0, 0, 0, -1439).is_ok());
    }

    #[test]
    fn test_new_naive_date_failing() {
        let tests = [
            (
                NaiveDate::new(10000, 2, 1),
                NaiveDateError::YearOutOfRange { got: 10000 },
            ),
            (
                NaiveDate::new(2020, 0, 1),
                NaiveDateError::MonthOutOfRange { got: 0 },
            ),
            (
                NaiveDate::new(2020, 13, 1),
                NaiveDateError::MonthOutOfRange { got: 13 },
            ),
            (
                NaiveDate::new(2020, 1, 0),
                NaiveDateError::DayOutOfRange { got: 0, max: 31 },
            ),
            (
                NaiveDate::new(2020, 4, 31),
                NaiveDateError::DayOutOfRange { got: 31, max: 30 },
            ),
            (
                NaiveDate::new(2019, 2, 29),
                NaiveDateError::DayOutOfRange { got: 29, max: 28 },
            ),
            (
                NaiveDate::new(1900, 2, 29),
                NaiveDateError::DayOutOfRange { got: 29, max: 28 },
            ),
        ];

        for (got, expected) in tests {
            println!("{}", got.clone().unwrap_err());
            println!("{:?}", got.clone().unwrap_err());
            assert_eq!(expected, got.unwrap_err());
        }

        for year in [0, 2000, 2020, 9996] {
            assert!(NaiveDate::new(year, 2, 29).is_ok(), "{year}");
        }
    }

    #[test]
    fn test_debug() {
        assert_eq!(format!("{:?}", date(7, 2, 1)), "0007-02-01");
        assert_eq!(
            format!(
                "{:?}",
                DateTime::new(date(1994, 2, 7), 21, 52, 25, -480).unwrap()
            ),
            "1994-02-07T21:52:25-08:00"
        );
        assert_eq!(
            format!(
                "{:?}",
                DateTime::new(date(1994, 2, 7), 21, 52, 25, 0).unwrap()
            ),
            "1994-02-07T21:52:25+00:00"
        );
    }

    #[test]
    fn test_conversion_date_time_failing() {
        let tests = [
            (
                DateTime::try_from(
                    chrono::FixedOffset::east_opt(3600)
                        .unwrap()
                        .from_local_datetime(&chrono::NaiveDateTime::new(
                            chrono::NaiveDate::from_ymd_opt(-1, 2, 1).unwrap(),
                            chrono::NaiveTime::from_hms_opt(12, 34, 56).unwrap(),
                        ))
                        .unwrap(),
                ),
                DateTimeError::YearOutOfRange { got: -1 },
            ),
            (
                DateTime::try_from(
                    chrono::FixedOffset::east_opt(3600)
                        .unwrap()
                        .from_local_datetime(&chrono::NaiveDateTime::new(
                            chrono::NaiveDate::from_ymd_opt(10000, 2, 1).unwrap(),
                            chrono::NaiveTime::from_hms_opt(12, 34, 56).unwrap(),
                        ))
                        .unwrap(),
                ),
                DateTimeError::YearOutOfRange { got: 10000 },
            ),
            (
                DateTime::try_from(
                    chrono::FixedOffset::east_opt(1)
                        .unwrap()
                        .from_local_datetime(&chrono::NaiveDateTime::new(
                            chrono::NaiveDate::from_ymd_opt(0, 2, 1).unwrap(),
                            chrono::NaiveTime::from_hms_opt(12, 34, 56).unwrap(),
                        ))
                        .unwrap(),
                ),
                DateTimeError::UnalignedOffset { got: 1 },
            ),
            (
                DateTime::try_from(
                    chrono::FixedOffset::east_opt(59)
                        .unwrap()
                        .from_local_datetime(&chrono::NaiveDateTime::new(
                            chrono::NaiveDate::from_ymd_opt(9999, 2, 1).unwrap(),
                            chrono::NaiveTime::from_hms_opt(12, 34, 56).unwrap(),
                        ))
                        .unwrap(),
                ),
                DateTimeError::UnalignedOffset { got: 59 },
            ),
            (
                DateTime::try_from(
                    chrono::FixedOffset::east_opt(60)
                        .unwrap()
                        .from_local_datetime(&chrono::NaiveDateTime::new(
                            chrono::NaiveDate::from_ymd_opt(0, 2, 1).unwrap(),
                            chrono::NaiveTime::from_hms_opt(12, 34, 56).unwrap(),
                        ))
                        .unwrap()
                        .with_nanosecond(1)
                        .unwrap(),
                ),
                DateTimeError::UnalignedNanoSeconds { got: 1 },
            ),
        ];

        for (got, expected) in tests {
            println!("{}", got.clone().unwrap_err());
            println!("{:?}", got.clone().unwrap_err());
            assert_eq!(expected, got.unwrap_err());
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialization_date_time() {
        let valid_input = r#""2015-05-15T11:22:33+01:00""#;
        let invalid_input = r#""+12015-05-15T11:22:33+01:00""#;

        let date_time = serde_json::from_str::<DateTime>(valid_input)
            .expect("valid input should deserialize successfully");
        assert_eq!(
            date_time,
            DateTime::new(date(2015, 5, 15), 11, 22, 33, 60).unwrap(),
        );

        let err = serde_json::from_str::<DateTime>(invalid_input)
            .expect_err("invalid input should not deserialize successfully");
        assert_eq!(err.to_string(), r"expected `0 <= year <= 9999`, got 12015");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialization_naive_date() {
        let valid_input = r#""2015-05-15""#;
        let invalid_input = r#""+12015-05-15""#;

        let naive_date = serde_json::from_str::<NaiveDate>(valid_input)
            .expect("valid input should deserialize successfully");
        assert_eq!(naive_date, date(2015, 5, 15));

        let err = serde_json::from_str::<NaiveDate>(invalid_input)
            .expect_err("invalid input should not deserialize successfully");
        assert_eq!(err.to_string(), r"expected `0 <= year <= 9999`, got 12015");
    }
}
