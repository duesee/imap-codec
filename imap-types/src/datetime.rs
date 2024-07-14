//! Date and time-related types.

use std::fmt::{Debug, Formatter};

use bounded_static::{IntoBoundedStatic, ToBoundedStatic};
use chrono::{Datelike, FixedOffset};
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
}

impl TryFrom<chrono::DateTime<FixedOffset>> for DateTime {
    type Error = DateTimeError;

    fn try_from(value: chrono::DateTime<FixedOffset>) -> Result<Self, Self::Error> {
        Self::validate(&value)?;

        Ok(Self(value))
    }
}

impl Debug for DateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
}

impl TryFrom<chrono::NaiveDate> for NaiveDate {
    type Error = NaiveDateError;

    fn try_from(value: chrono::NaiveDate) -> Result<Self, Self::Error> {
        Self::validate(&value)?;

        Ok(Self(value))
    }
}

impl Debug for NaiveDate {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
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
    }

    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum NaiveDateError {
        #[error("expected `0 <= year <= 9999`, got {got}")]
        YearOutOfRange { got: i32 },
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Timelike};

    use super::*;

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
            DateTime(
                chrono::FixedOffset::east_opt(3600)
                    .unwrap()
                    .with_ymd_and_hms(2015, 5, 15, 11, 22, 33)
                    .unwrap(),
            ),
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
        assert_eq!(
            naive_date,
            NaiveDate(chrono::NaiveDate::from_ymd_opt(2015, 5, 15).unwrap()),
        );

        let err = serde_json::from_str::<NaiveDate>(invalid_input)
            .expect_err("invalid input should not deserialize successfully");
        assert_eq!(err.to_string(), r"expected `0 <= year <= 9999`, got 12015");
    }
}
