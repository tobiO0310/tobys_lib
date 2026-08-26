#[cfg(feature = "std")]
use std::str::FromStr;

use ::core::fmt;
use jiff::{
    Span,
    civil::{Date, DateTime, datetime, time},
};
use thiserror::Error;

use crate::alias::Vec;

/// A single section of a cron string as an enum type.
// TODO; Add n/n syntax (every nth starting at), add multiple At values (using Vec?)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CronSection {
    /// Represents `*` in this section
    EveryTime,
    /// Represents a single value in this section
    At(u8),
    /// Represents `*/n` in this section
    EveryNth(u8),
}

/// An error returned during parsing of the cron string.
///
/// Please see the variants for their own explanations :)
#[derive(Error, Debug, Clone, Copy)]
pub enum CronParsingError {
    /// The cron string supplied does not contain 5 different sections split by a single space
    ///
    /// A correct format is like so;
    /// - `* * * * *`
    /// - `1 1 * * 4`
    /// - etc.
    #[error(
        "The cron string supplied does not contain 5 different sections split by a single space"
    )]
    NotCorrectLength,

    /// The first part in the cron string (minute section) is not in a correct format
    ///
    /// Correct formats are;
    /// - `*`: Every minute
    /// - `n`: At `n` minute
    /// - `*/n` Every `n`th minute
    #[error(
        "The first part in the cron string (minute section) is not in a correct format"
    )]
    MinuteIsNotCorrectFormat,
    /// Minute is above 59
    #[error("Minute {0} is above 59")]
    MinuteIsTooBig(u8),

    /// The second part in the cron string (hour section) is not in a correct format
    ///
    /// Correct formats are;
    /// - `*`: Every hour
    /// - `n`: At `n` hour
    /// - `*/n` Every `n`th hour
    #[error(
        "The second part in the cron string (hour section) is not in a correct format"
    )]
    HourIsNotCorrectFormat,
    /// Hour is above 59
    #[error("Hour {0} is above 23")]
    HourIsTooBig(u8),

    /// The third part in the cron string (day of month section) is not in a correct format
    ///
    /// Correct formats are;
    /// - `*`: Every day of the month
    /// - `n`: At `n` day of the month
    /// - `*/n` Every `n`th day of the month
    #[error(
        "The third part in the cron string (day of month section) is not in a correct format"
    )]
    DayOfMonthIsNotCorrectFormat,
    /// Day of month must be a valid `u8` between 1 and 31, inclusive on both sides.
    #[error("Day {0} is not a valid month day (1-31)")]
    DayOfMonthIsTooBig(u8),

    /// The fourth part in the cron string (month section) is not in a correct format
    ///
    /// Correct formats are;
    /// - `*`: Every month
    /// - `n`: At `n` month
    /// - `*/n` Every `n`th month
    #[error(
        "The fourth part in the cron string (month section) is not in a correct format"
    )]
    MonthIsNotCorrectFormat,
    /// Month must be a valid `u8` between 1 and 12, inclusive on both sides.
    #[error("Month {0} is not a valid month (1-12)")]
    MonthIsTooBig(u8),

    /// The fifth part in the cron string (week day section) is not in a correct format
    ///
    /// Correct formats are;
    /// - `*`: Every day of the week
    /// - `n`: At `n` day of the week
    /// - `*/n` Every `n`th day of the week
    #[error(
        "The fifth part in the cron string (week day section) is not in a correct format"
    )]
    DayOfWeekIsNotCorrectFormat,
    /// Day of week must be a valid `u8` between 0 and 7, inclusive on both sides.
    ///
    /// The meaning of the value is;
    /// - 0 and 7 means Sunday
    /// - 1 means Monday
    /// - 2 means Tuesday
    /// - 3 means Wednesday
    /// - 4 means Thursday
    /// - 5 means Friday
    /// - 6 means Saturday
    #[error("Day {0} is not a valid week day (0-7)")]
    DayOfWeekIsTooBig(u8),
}

/// A parsed cron string.
///
/// # Examples
///
/// Parse a every-minute cron string
/// ```
/// # use tobys_lib_core::cron::CronTime;
/// # assert!(
/// CronTime::parse("* * * * *")
/// # .is_ok());
/// ```
#[must_use = "CronTime requires validation that is dead code if this is not used"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CronTime {
    minute: CronSection,
    hour: CronSection,
    day_of_month: CronSection,
    month: CronSection,
    day_of_week: CronSection,

    valid_days: [bool; 31],
    valid_months: [bool; 12],
    valid_week_days: [bool; 7],
}

impl fmt::Display for CronSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CronSection::EveryTime => f.write_str("*"),
            CronSection::At(v) => f.write_fmt(format_args!("{v}")),
            CronSection::EveryNth(v) => f.write_fmt(format_args!("*/{v}")),
        }
    }
}
impl fmt::Display for CronTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{} {} {} {} {}",
            self.minute,
            self.hour,
            self.day_of_month,
            self.month,
            self.day_of_week
        ))
    }
}
impl From<CronParsingError> for () {
    // allows for converting ? to an empty error in no_std environments
    fn from(_: CronParsingError) -> Self {}
}
#[cfg(feature = "std")]
impl FromStr for CronTime {
    type Err = CronParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl CronSection {
    fn new(str: &str) -> Option<Self> {
        match str {
            "*" => Some(CronSection::EveryTime),
            x if let Ok(y) = x.parse() => Some(CronSection::At(y)),
            x if x.starts_with("*/") => {
                let (x, _) = x.split_at(2);
                if let Ok(y) = x.parse() {
                    Some(CronSection::EveryNth(y))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    const fn within(self, min: u8, max: u8) -> Result<(), u8> {
        match self {
            Self::EveryTime => Ok(()),
            Self::At(x) | Self::EveryNth(x) if min <= x && x <= max => Ok(()),
            Self::At(x) | Self::EveryNth(x) => Err(x),
        }
    }
}

impl CronParsingError {
    /// Returns `true` if the cron parsing error is [`NotCorrectLength`].
    ///
    /// [`NotCorrectLength`]: CronParsingError::NotCorrectLength
    #[must_use]
    pub fn is_not_correct_length(&self) -> bool {
        matches!(self, Self::NotCorrectLength)
    }

    /// Returns `true` if the cron parsing error is [`MinuteIsNotCorrectFormat`].
    ///
    /// [`MinuteIsNotCorrectFormat`]: CronParsingError::MinuteIsNotCorrectFormat
    #[must_use]
    pub fn is_minute_is_not_correct_format(&self) -> bool {
        matches!(self, Self::MinuteIsNotCorrectFormat)
    }

    /// Returns `true` if the cron parsing error is [`MinuteIsTooBig`].
    ///
    /// [`MinuteIsTooBig`]: CronParsingError::MinuteIsTooBig
    #[must_use]
    pub fn is_minute_is_too_big(&self) -> bool {
        matches!(self, Self::MinuteIsTooBig(..))
    }

    /// Returns `true` if the cron parsing error is [`HourIsNotCorrectFormat`].
    ///
    /// [`HourIsNotCorrectFormat`]: CronParsingError::HourIsNotCorrectFormat
    #[must_use]
    pub fn is_hour_is_not_correct_format(&self) -> bool {
        matches!(self, Self::HourIsNotCorrectFormat)
    }

    /// Returns `true` if the cron parsing error is [`HourIsTooBig`].
    ///
    /// [`HourIsTooBig`]: CronParsingError::HourIsTooBig
    #[must_use]
    pub fn is_hour_is_too_big(&self) -> bool {
        matches!(self, Self::HourIsTooBig(..))
    }

    /// Returns `true` if the cron parsing error is [`DayOfMonthIsNotCorrectFormat`].
    ///
    /// [`DayOfMonthIsNotCorrectFormat`]: CronParsingError::DayOfMonthIsNotCorrectFormat
    #[must_use]
    pub fn is_day_of_month_is_not_correct_format(&self) -> bool {
        matches!(self, Self::DayOfMonthIsNotCorrectFormat)
    }

    /// Returns `true` if the cron parsing error is [`DayOfMonthIsTooBig`].
    ///
    /// [`DayOfMonthIsTooBig`]: CronParsingError::DayOfMonthIsTooBig
    #[must_use]
    pub fn is_day_of_month_is_too_big(&self) -> bool {
        matches!(self, Self::DayOfMonthIsTooBig(..))
    }

    /// Returns `true` if the cron parsing error is [`MonthIsNotCorrectFormat`].
    ///
    /// [`MonthIsNotCorrectFormat`]: CronParsingError::MonthIsNotCorrectFormat
    #[must_use]
    pub fn is_month_is_not_correct_format(&self) -> bool {
        matches!(self, Self::MonthIsNotCorrectFormat)
    }

    /// Returns `true` if the cron parsing error is [`MonthIsTooBig`].
    ///
    /// [`MonthIsTooBig`]: CronParsingError::MonthIsTooBig
    #[must_use]
    pub fn is_month_is_too_big(&self) -> bool {
        matches!(self, Self::MonthIsTooBig(..))
    }

    /// Returns `true` if the cron parsing error is [`DayOfWeekIsNotCorrectFormat`].
    ///
    /// [`DayOfWeekIsNotCorrectFormat`]: CronParsingError::DayOfWeekIsNotCorrectFormat
    #[must_use]
    pub fn is_day_of_week_is_not_correct_format(&self) -> bool {
        matches!(self, Self::DayOfWeekIsNotCorrectFormat)
    }

    /// Returns `true` if the cron parsing error is [`DayOfWeekIsTooBig`].
    ///
    /// [`DayOfWeekIsTooBig`]: CronParsingError::DayOfWeekIsTooBig
    #[must_use]
    pub fn is_day_of_week_is_too_big(&self) -> bool {
        matches!(self, Self::DayOfWeekIsTooBig(..))
    }
}

macro_rules! unwrap_cron {
    ($vec:ident, $pos:literal, $min:literal, $max:literal, $err1:expr, $err2:expr) => {{
        let section = CronSection::new(
            $vec.get($pos).ok_or(CronParsingError::NotCorrectLength)?,
        )
        .ok_or($err1)?;
        if let Err(val) = section.within($min, $max) {
            return Err($err2(val));
        }

        section
    }};
}

impl CronTime {
    #[expect(
        clippy::indexing_slicing,
        clippy::as_conversions,
        reason = "some trait equivalents are unstable in const, all values are guaranteed to be valid conversions and all indexing is within bounds"
    )]
    const fn get_valid_times(
        day_of_month: CronSection,
        month: CronSection,
        day_of_week: CronSection,
    ) -> ([bool; 31], [bool; 12], [bool; 7]) {
        let valid_days = match day_of_month {
            CronSection::EveryTime => [true; 31],
            CronSection::At(v) => {
                let mut arr = [false; 31];
                arr[(v as usize).saturating_sub(1)] = true;
                arr
            }
            CronSection::EveryNth(v) => {
                let mut arr = [false; 31];

                let v = v as usize;
                let mut i = 0;
                while i < 31 {
                    arr[i] = i.is_multiple_of(v);
                    i = i.saturating_add(1);
                }

                arr
            }
        };
        let valid_months = match month {
            CronSection::EveryTime => [true; 12],
            CronSection::At(v) => {
                let mut arr = [false; 12];
                arr[(v as usize).saturating_sub(1)] = true;
                arr
            }
            CronSection::EveryNth(v) => {
                let mut arr = [false; 12];

                let v = v as usize;
                let mut i = 0;
                while i < 12 {
                    arr[i] = i.is_multiple_of(v);
                    i = i.saturating_add(1);
                }

                arr
            }
        };
        let valid_week_days = match day_of_week {
            CronSection::EveryTime => [true; 7],
            CronSection::At(v) => {
                let mut arr = [false; 7];
                arr[v as usize % 7] = true;
                arr
            }
            CronSection::EveryNth(v) => {
                let mut arr = [false; 7];

                let v = v as usize;
                let mut i = 0;
                while i < 7 {
                    arr[i] = i.is_multiple_of(v);
                    i = i.saturating_add(1);
                }

                arr
            }
        };

        (valid_days, valid_months, valid_week_days)
    }

    /// Parses a cron string into a [`CronTime`].
    ///
    /// # Errors
    ///
    /// This function errors if the cron string is not a valid cron string.
    pub fn new(cron: &str) -> Result<Self, CronParsingError> {
        let split = cron.split(' ');
        let vec: Vec<_> = split.collect();
        if vec.len() != 5 {
            return Err(CronParsingError::NotCorrectLength);
        }

        let minute = CronSection::new(
            vec.first().ok_or(CronParsingError::NotCorrectLength)?,
        )
        .ok_or(CronParsingError::MinuteIsNotCorrectFormat)?;
        if let Err(val) = minute.within(0, 59) {
            return Err(CronParsingError::MinuteIsTooBig(val));
        }

        let minute = unwrap_cron!(
            vec,
            0,  // vec pos
            0,  // min
            59, // max
            CronParsingError::MinuteIsNotCorrectFormat,
            CronParsingError::MinuteIsTooBig
        );
        let hour = unwrap_cron!(
            vec,
            1,  // vec pos
            0,  // min
            23, // max
            CronParsingError::HourIsNotCorrectFormat,
            CronParsingError::HourIsTooBig
        );
        let day_of_month = unwrap_cron!(
            vec,
            2,  // vec pos
            1,  // min
            31, // max
            CronParsingError::DayOfMonthIsNotCorrectFormat,
            CronParsingError::DayOfMonthIsTooBig
        );
        let month = unwrap_cron!(
            vec,
            3,  // vec pos
            1,  // min
            12, // max
            CronParsingError::MonthIsNotCorrectFormat,
            CronParsingError::MonthIsTooBig
        );
        let day_of_week = unwrap_cron!(
            vec,
            4, // vec pos
            0, // min
            7, // max
            CronParsingError::DayOfWeekIsNotCorrectFormat,
            CronParsingError::DayOfWeekIsTooBig
        );

        let (valid_days, valid_months, valid_week_days) =
            CronTime::get_valid_times(day_of_month, month, day_of_week);

        Ok(Self {
            minute,
            hour,
            day_of_month,
            month,
            day_of_week,
            valid_days,
            valid_months,
            valid_week_days,
        })
    }

    /// Uses the given [`CronSection`] array and assumes it is includes;
    /// `[minute, hour, day_of_month, month, day_of_week]`.
    ///
    /// # Panics
    ///
    /// Will panic is `cron` is not at least 5 long.
    #[expect(
        clippy::indexing_slicing,
        reason = "it is up to the caller to guarantee this doesn't happen"
    )]
    pub const fn new_unchecked(cron: &[CronSection]) -> Self {
        let (valid_days, valid_months, valid_week_days) =
            CronTime::get_valid_times(cron[2], cron[3], cron[4]);
        Self {
            minute: cron[0],
            hour: cron[1],
            day_of_month: cron[2],
            month: cron[3],
            day_of_week: cron[4],
            valid_days,
            valid_months,
            valid_week_days,
        }
    }

    /// Returns the array representation of this
    #[must_use]
    pub const fn into_array(self) -> [CronSection; 5] {
        [
            self.minute,
            self.hour,
            self.day_of_month,
            self.month,
            self.day_of_week,
        ]
    }

    fn is_valid_day(&self, date: Date) -> bool {
        if let Ok(day) = usize::try_from(date.day()) {
            *self.valid_days.get(day.saturating_sub(1)).unwrap_or(&false)
        } else {
            false
        }
    }
    fn next_valid_day(&self, mut date: Date) -> Date {
        while !self.is_valid_day(date) {
            date = date.saturating_add(Span::new().days(1));
        }
        date
    }
    fn is_valid_month(&self, date: Date) -> bool {
        if let Ok(month) = usize::try_from(date.month()) {
            *self
                .valid_months
                .get(month.saturating_sub(1))
                .unwrap_or(&false)
        } else {
            false
        }
    }
    fn next_valid_month(&self, mut date: Date) -> Date {
        while !self.is_valid_month(date) {
            date = date.saturating_add(Span::new().months(1));
        }
        date.first_of_month()
    }
    fn is_valid_weekday(&self, date: Date) -> bool {
        if let Ok(weekday) =
            usize::try_from(date.weekday().to_sunday_zero_offset())
        {
            *self.valid_week_days.get(weekday).unwrap_or(&false)
        } else {
            false
        }
    }
    fn is_valid_date(&self, date: Date) -> bool {
        self.is_valid_day(date) && self.is_valid_month(date)
    }
    fn next_valid_date(&self, mut date: Date) -> Date {
        date = date.saturating_add(Span::new().days(1)); // make sure to get a NEXT valid date

        while !self.is_valid_date(date) {
            if !self.is_valid_month(date) {
                date = self.next_valid_month(date);
            }
            if !self.is_valid_day(date) {
                date = self.next_valid_day(date);
            }
        }

        date
    }

    #[must_use]
    pub(super) fn get_next_time(&self) -> DateTime {
        let curr = jiff::Zoned::now();

        #[expect(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap,
            clippy::arithmetic_side_effects,
            clippy::as_conversions,
            clippy::expect_used,
            reason = "all clippy lints are 100% guaranteed to NOT happen"
        )]
        let next_minute = match self.minute {
            CronSection::EveryTime => (curr.minute() + 1) % 60,
            CronSection::At(v) => v.try_into().expect("v is between 0 and 59"),
            CronSection::EveryNth(v) => {
                let mut next_val =
                    (curr.minute() as u32).div_ceil(v.into()) as i8 * v as i8;
                while next_val >= 60 {
                    next_val %= 60;
                }
                next_val
            }
        };

        #[expect(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap,
            clippy::arithmetic_side_effects,
            clippy::as_conversions,
            clippy::expect_used,
            reason = "all clippy lints are 100% guaranteed to NOT happen"
        )]
        let next_hour = match self.hour {
            CronSection::EveryTime => (curr.hour() + 1) % 24,
            CronSection::At(v) => v.try_into().expect("v is between 0 and 23"),
            CronSection::EveryNth(v) => {
                let mut next_val =
                    (curr.hour() as u32).div_ceil(v.into()) as i8 * v as i8;
                while next_val >= 24 {
                    next_val %= 24;
                }
                next_val
            }
        };

        let mut datetime = datetime(
            curr.year(),
            curr.month(),
            curr.day(),
            next_hour,
            next_minute,
            0,
            0,
        );
        let time = time(next_hour, next_minute, 0, 0);
        #[expect(clippy::arithmetic_side_effects)]
        if datetime < curr.datetime() {
            datetime += Span::new().days(1);
        }

        if !self.is_valid_date(datetime.date()) {
            datetime = self.next_valid_date(datetime.date()).to_datetime(time);
        }
        while !self.is_valid_weekday(datetime.date()) {
            // get next valid day & month, and retry if weekday fits
            datetime = self.next_valid_date(datetime.date()).to_datetime(time);
        }

        datetime
    }
}
