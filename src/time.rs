use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{ZsuiError, ZsuiResult};

const MINUTES_PER_DAY: i32 = 24 * 60;

/// A validated wall-clock time without a date or time zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ZsTime {
    hour: u8,
    minute: u8,
}

impl ZsTime {
    pub const MIDNIGHT: Self = Self { hour: 0, minute: 0 };

    pub fn new(hour: u8, minute: u8) -> ZsuiResult<Self> {
        if hour > 23 {
            return Err(ZsuiError::invalid_spec(
                "time.hour",
                "hour must be between 0 and 23",
            ));
        }
        if minute > 59 {
            return Err(ZsuiError::invalid_spec(
                "time.minute",
                "minute must be between 0 and 59",
            ));
        }
        Ok(Self { hour, minute })
    }

    /// Parses the canonical platform-independent `HH:MM` representation used
    /// by UI documents and typed state bindings.
    pub fn parse_24_hour(value: &str) -> ZsuiResult<Self> {
        let bytes = value.as_bytes();
        if bytes.len() != 5
            || bytes[2] != b':'
            || !bytes
                .iter()
                .enumerate()
                .all(|(index, byte)| index == 2 || byte.is_ascii_digit())
        {
            return Err(ZsuiError::invalid_spec(
                "time",
                "time must use the canonical HH:MM 24-hour representation",
            ));
        }
        let hour = value[0..2].parse::<u8>().map_err(|_| {
            ZsuiError::invalid_spec(
                "time.hour",
                "hour must use two ASCII digits between 00 and 23",
            )
        })?;
        let minute = value[3..5].parse::<u8>().map_err(|_| {
            ZsuiError::invalid_spec(
                "time.minute",
                "minute must use two ASCII digits between 00 and 59",
            )
        })?;
        Self::new(hour, minute)
    }

    pub const fn hour(self) -> u8 {
        self.hour
    }

    pub const fn minute(self) -> u8 {
        self.minute
    }

    pub const fn minutes_since_midnight(self) -> u16 {
        self.hour as u16 * 60 + self.minute as u16
    }

    pub fn with_hour(self, hour: u8) -> ZsuiResult<Self> {
        Self::new(hour, self.minute)
    }

    pub fn with_minute(self, minute: u8) -> ZsuiResult<Self> {
        Self::new(self.hour, minute)
    }

    /// Adds minutes and wraps at midnight, matching a time-only picker.
    pub fn add_minutes_wrapping(self, offset: i32) -> Self {
        let total = (i32::from(self.minutes_since_midnight()) + offset).rem_euclid(MINUTES_PER_DAY);
        Self {
            hour: (total / 60) as u8,
            minute: (total % 60) as u8,
        }
    }

    pub fn snap(self, increment: ZsMinuteIncrement) -> Self {
        let step = i32::from(increment.get());
        let total = i32::from(self.minutes_since_midnight());
        let snapped = ((total + step / 2) / step * step).rem_euclid(MINUTES_PER_DAY);
        Self {
            hour: (snapped / 60) as u8,
            minute: (snapped % 60) as u8,
        }
    }

    pub fn format(self, clock: ZsClockFormat) -> String {
        match clock {
            ZsClockFormat::TwentyFourHour => format!("{:02}:{:02}", self.hour, self.minute),
            ZsClockFormat::TwelveHour => {
                let hour = match self.hour % 12 {
                    0 => 12,
                    hour => hour,
                };
                let period = if self.hour < 12 { "AM" } else { "PM" };
                format!("{hour}:{:02} {period}", self.minute)
            }
        }
    }
}

impl fmt::Display for ZsTime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:02}:{:02}", self.hour, self.minute)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZsClockFormat {
    TwelveHour,
    TwentyFourHour,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZsMinuteIncrement(u8);

impl ZsMinuteIncrement {
    pub const ONE: Self = Self(1);
    pub const FIVE: Self = Self(5);
    pub const TEN: Self = Self(10);
    pub const FIFTEEN: Self = Self(15);
    pub const THIRTY: Self = Self(30);

    pub fn new(minutes: u8) -> ZsuiResult<Self> {
        if minutes == 0 || minutes >= 60 || 60 % minutes != 0 {
            return Err(ZsuiError::invalid_spec(
                "time_picker.minute_increment",
                "minute increment must be a non-zero divisor of 60 smaller than 60",
            ));
        }
        Ok(Self(minutes))
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl Default for ZsMinuteIncrement {
    fn default() -> Self {
        Self::ONE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_validation_and_clock_formatting_are_typed() {
        let time = ZsTime::new(18, 5).unwrap();

        assert_eq!(time.format(ZsClockFormat::TwentyFourHour), "18:05");
        assert_eq!(time.format(ZsClockFormat::TwelveHour), "6:05 PM");
        assert_eq!(ZsTime::parse_24_hour("18:05").unwrap(), time);
        assert!(ZsTime::new(24, 0).is_err());
        assert!(ZsTime::new(12, 60).is_err());
        assert!(ZsTime::parse_24_hour("8:05").is_err());
        assert!(ZsTime::parse_24_hour("18:5").is_err());
        assert!(ZsTime::parse_24_hour("24:00").is_err());
        assert!(ZsTime::parse_24_hour("ab:cd").is_err());
    }

    #[test]
    fn time_arithmetic_wraps_and_snaps_to_a_valid_increment() {
        let time = ZsTime::new(23, 58).unwrap();

        assert_eq!(time.add_minutes_wrapping(5), ZsTime::new(0, 3).unwrap());
        assert_eq!(time.snap(ZsMinuteIncrement::FIFTEEN), ZsTime::MIDNIGHT);
        assert!(ZsMinuteIncrement::new(7).is_err());
    }
}
