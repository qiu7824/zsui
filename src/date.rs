use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{ZsuiError, ZsuiResult};

/// A validated proleptic-Gregorian calendar date without a time zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ZsDate {
    year: i32,
    month: u8,
    day: u8,
}

impl ZsDate {
    pub const MIN_YEAR: i32 = 1;
    pub const MAX_YEAR: i32 = 9999;

    pub fn new(year: i32, month: u8, day: u8) -> ZsuiResult<Self> {
        if !(Self::MIN_YEAR..=Self::MAX_YEAR).contains(&year) {
            return Err(ZsuiError::invalid_spec(
                "date.year",
                format!(
                    "year must be between {} and {}",
                    Self::MIN_YEAR,
                    Self::MAX_YEAR
                ),
            ));
        }
        if !(1..=12).contains(&month) {
            return Err(ZsuiError::invalid_spec(
                "date.month",
                "month must be between 1 and 12",
            ));
        }
        let maximum = days_in_month(year, month);
        if day == 0 || day > maximum {
            return Err(ZsuiError::invalid_spec(
                "date.day",
                format!("day must be between 1 and {maximum} for {year:04}-{month:02}"),
            ));
        }
        Ok(Self { year, month, day })
    }

    pub const fn year(self) -> i32 {
        self.year
    }

    pub const fn month(self) -> u8 {
        self.month
    }

    pub const fn day(self) -> u8 {
        self.day
    }

    pub const fn days_in_month(self) -> u8 {
        days_in_month(self.year, self.month)
    }

    pub fn first_day_of_month(self) -> Self {
        Self {
            year: self.year,
            month: self.month,
            day: 1,
        }
    }

    /// Returns Sunday as 0 through Saturday as 6.
    pub fn weekday_from_sunday(self) -> u8 {
        (self.unix_days() + 4).rem_euclid(7) as u8
    }

    pub fn add_months(self, offset: i32) -> Self {
        let month_index = i64::from(self.year - 1)
            .saturating_mul(12)
            .saturating_add(i64::from(self.month - 1))
            .saturating_add(i64::from(offset))
            .clamp(0, i64::from(Self::MAX_YEAR) * 12 - 1);
        let year = (month_index / 12 + 1) as i32;
        let month = (month_index % 12 + 1) as u8;
        Self {
            year,
            month,
            day: self.day.min(days_in_month(year, month)),
        }
    }

    pub fn add_days(self, offset: i32) -> Self {
        let minimum = days_from_civil(Self::MIN_YEAR, 1, 1);
        let maximum = days_from_civil(Self::MAX_YEAR, 12, 31);
        let days = self
            .unix_days()
            .saturating_add(i64::from(offset))
            .clamp(minimum, maximum);
        Self::from_unix_days(days)
    }

    pub fn clamp(self, minimum: Self, maximum: Self) -> Self {
        let (minimum, maximum) = if minimum <= maximum {
            (minimum, maximum)
        } else {
            (maximum, minimum)
        };
        self.max(minimum).min(maximum)
    }

    pub(crate) const fn unix_days(self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
    }

    pub(crate) fn from_unix_days(days: i64) -> Self {
        let (year, month, day) = civil_from_days(days);
        Self { year, month, day }
    }

    pub fn iso_string(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Returns today's date in the operating system's local time zone.
    #[cfg(feature = "date-picker")]
    pub fn today_local() -> ZsuiResult<Self> {
        use chrono::Datelike;

        let today = chrono::Local::now().date_naive();
        Self::new(today.year(), today.month() as u8, today.day() as u8)
    }
}

impl fmt::Display for ZsDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

pub const fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

pub const fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

const fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let year = year as i64 - if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month = month as i64;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day as i64 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(days: i64) -> (i32, u8, u8) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year as i32, month as u8, day as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_validation_and_leap_years_are_gregorian() {
        assert!(ZsDate::new(2024, 2, 29).is_ok());
        assert!(ZsDate::new(2023, 2, 29).is_err());
        assert!(ZsDate::new(2024, 13, 1).is_err());
        assert!(ZsDate::new(0, 1, 1).is_err());
    }

    #[test]
    fn date_day_arithmetic_crosses_month_and_year_boundaries() {
        let leap_day = ZsDate::new(2024, 2, 28).unwrap().add_days(1);
        assert_eq!(leap_day, ZsDate::new(2024, 2, 29).unwrap());
        assert_eq!(leap_day.add_days(1), ZsDate::new(2024, 3, 1).unwrap());
        assert_eq!(
            ZsDate::new(2024, 1, 1).unwrap().add_days(-1),
            ZsDate::new(2023, 12, 31).unwrap()
        );
    }

    #[test]
    fn month_arithmetic_clamps_the_day_and_supported_range() {
        assert_eq!(
            ZsDate::new(2024, 1, 31).unwrap().add_months(1),
            ZsDate::new(2024, 2, 29).unwrap()
        );
        assert_eq!(
            ZsDate::new(1, 1, 1).unwrap().add_months(-1),
            ZsDate::new(1, 1, 1).unwrap()
        );
        assert_eq!(
            ZsDate::new(9999, 12, 31).unwrap().add_months(1),
            ZsDate::new(9999, 12, 31).unwrap()
        );
    }

    #[test]
    fn weekday_uses_sunday_as_zero() {
        assert_eq!(ZsDate::new(1970, 1, 1).unwrap().weekday_from_sunday(), 4);
        assert_eq!(ZsDate::new(2026, 7, 1).unwrap().weekday_from_sunday(), 3);
    }

    #[test]
    fn unix_day_round_trip_is_stable_across_supported_range() {
        for date in [
            ZsDate::new(1, 1, 1).unwrap(),
            ZsDate::new(1970, 1, 1).unwrap(),
            ZsDate::new(2000, 2, 29).unwrap(),
            ZsDate::new(9999, 12, 31).unwrap(),
        ] {
            assert_eq!(ZsDate::from_unix_days(date.unix_days()), date);
        }
    }

    #[cfg(feature = "date-picker")]
    #[test]
    fn local_today_is_a_valid_supported_date() {
        let today = ZsDate::today_local().expect("local clock date should be representable");

        assert!((ZsDate::MIN_YEAR..=ZsDate::MAX_YEAR).contains(&today.year()));
        assert!((1..=12).contains(&today.month()));
        assert!((1..=today.days_in_month()).contains(&today.day()));
    }
}
