use std::fmt::{Debug, Formatter};

#[cfg(feature = "bounded-static")]
use bounded_static::{IntoBoundedStatic, ToBoundedStatic};
use chrono::{Datelike, FixedOffset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct DateTime(chrono::DateTime<FixedOffset>);

impl DateTime {
    #[cfg(feature = "unchecked")]
    pub fn unchecked(value: chrono::DateTime<FixedOffset>) -> Self {
        Self(value)
    }
}

impl TryFrom<chrono::DateTime<FixedOffset>> for DateTime {
    type Error = DateTimeError;

    fn try_from(value: chrono::DateTime<FixedOffset>) -> Result<Self, Self::Error> {
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

        Ok(Self(value))
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum DateTimeError {
    #[error("expected `0 <= year <= 9999`, got {got}")]
    YearOutOfRange { got: i32 },
    #[error("expected `nanos == 0`, got {got}")]
    UnalignedNanoSeconds { got: u32 },
    #[error("expected `offset % 60 == 0`, got {got}")]
    UnalignedOffset { got: i32 },
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

#[cfg(feature = "bounded-static")]
impl IntoBoundedStatic for DateTime {
    type Static = Self;

    fn into_static(self) -> Self::Static {
        self
    }
}

#[cfg(feature = "bounded-static")]
impl ToBoundedStatic for DateTime {
    type Static = Self;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct NaiveDate(chrono::NaiveDate);

impl NaiveDate {
    #[cfg(feature = "unchecked")]
    pub fn unchecked(value: chrono::NaiveDate) -> Self {
        Self(value)
    }
}

impl TryFrom<chrono::NaiveDate> for NaiveDate {
    type Error = NaiveDateError;

    fn try_from(value: chrono::NaiveDate) -> Result<Self, Self::Error> {
        // Only a subset of `chrono`s `NaiveDate` is valid in IMAP.
        if !(0..=9999).contains(&value.year()) {
            return Err(NaiveDateError::YearOutOfRange { got: value.year() });
        }

        Ok(Self(value))
    }
}

#[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
pub enum NaiveDateError {
    #[error("expected `0 <= year <= 9999`, got {got}")]
    YearOutOfRange { got: i32 },
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

#[cfg(feature = "bounded-static")]
impl IntoBoundedStatic for NaiveDate {
    type Static = Self;

    fn into_static(self) -> Self::Static {
        self
    }
}

#[cfg(feature = "bounded-static")]
impl ToBoundedStatic for NaiveDate {
    type Static = Self;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Timelike};

    use crate::{imap4rev1::datetime::DateTimeError, message::DateTime};

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
}
