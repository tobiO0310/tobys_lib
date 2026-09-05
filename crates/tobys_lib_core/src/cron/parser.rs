#[cfg(feature = "std")]
use std::str::FromStr;

use ::core::fmt;
use itertools::Itertools;
use jiff::{
    Span,
    civil::{Date, DateTime, Time, datetime, time},
};
use thiserror::Error;

use crate::alias::Vec;

/// A single section of a cron string as an enum type.
///
/// # Implementation notes
///
/// There are currently implemented 3 different formats, they are;
/// - `*`: Every unit
/// - `n`: At `n` unit
/// - `n-m`, `x,y,z`, & `n-m,x,y,a-f`: Range from `n` to `m` or multiple values/ranges.
/// - `*/n`: Every `n`th unit
/// - `x/n`: Starting at `x` and then every `n`th unit from there
///
/// You may get a `NotCorrectFormat` error when using valid multiple value/range notation,
/// this error is returned if any of the numbers in the notation is `>= 60`.
/// The reason behind this, is that it is not possible for us to store numbers 60 or bigger,
/// due to the internal storage setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive] // new syntax may be added later (no longer breaking change to add)
pub enum CronSection {
    /// Represents `*` in this section
    EveryTime,
    /// Represents a single value in this section
    At(u8),
    /// Represents the array of multiple chosen values.
    Multiple([bool; 60]),
    /// Represents `*/n` in this section
    ///
    /// Please remember, that this syntax starts at the unit's lowest value.
    EveryNth(u8),
    /// Represents `x/n` in this section
    StartingAtXEveryNth(u8, u8),
}

/// An error returned during parsing of the cron string.
///
/// Please see the variants for their own explanations :)
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
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
    #[error(
        "The first part in the cron string (minute section) is not in a correct format"
    )]
    MinuteIsNotCorrectFormat,
    /// Minute is above 59
    #[error("Minute {0} is above 59")]
    MinuteIsTooBig(u8),

    /// The second part in the cron string (hour section) is not in a correct format
    #[error(
        "The second part in the cron string (hour section) is not in a correct format"
    )]
    HourIsNotCorrectFormat,
    /// Hour is above 59
    #[error("Hour {0} is above 23")]
    HourIsTooBig(u8),

    /// The third part in the cron string (day of month section) is not in a correct format
    #[error(
        "The third part in the cron string (day of month section) is not in a correct format"
    )]
    DayOfMonthIsNotCorrectFormat,
    /// Day of month must be a valid `u8` between 1 and 31, inclusive on both sides.
    #[error("Day {0} is not a valid month day (1-31)")]
    DayOfMonthIsOutsideRange(u8),

    /// The fourth part in the cron string (month section) is not in a correct format
    #[error(
        "The fourth part in the cron string (month section) is not in a correct format"
    )]
    MonthIsNotCorrectFormat,
    /// Month must be a valid `u8` between 1 and 12, inclusive on both sides.
    #[error("Month {0} is not a valid month (1-12)")]
    MonthIsOutsideRange(u8),

    /// The fifth part in the cron string (week day section) is not in a correct format
    #[error(
        "The fifth part in the cron string (week day section) is not in a correct format"
    )]
    DayOfWeekIsNotCorrectFormat,
    /// Day of week must be a valid `u8` between 0 and 7, inclusive on both sides.
    #[error("Day {0} is not a valid week day (0-7)")]
    DayOfWeekIsTooBig(u8),
}

/// A parsed cron string.
///
/// A cron string is a string that has 5 sections, separated by a space,
/// which all together signify how often a [`Job`] should run.
/// The sections are as follows, and their allowed range of values, inclusive on both sides;
/// 1. minute (0-59)
/// 2. hour (0-23)
/// 3. day of month (1-31)
/// 4. month (1-12)
/// 5. day of week (0-7)
///
/// See [`CronSection`] for the valid formats for each section.
///
/// [`Job`]: crate::cron::Job
///
/// # Examples
///
/// Parse an every-minute cron string
/// ```
/// # use tobys_lib_core::cron::CronTime;
/// # assert!(
/// CronTime::new("* * * * *")
/// # .is_ok());
/// ```
///
/// Parse an every minute, every 5th hour, on even weekdays,
/// on the second day of the month, every other month cron string
/// ```
/// # use tobys_lib_core::cron::CronTime;
/// # assert!(
/// CronTime::new("* */5 2 */2 */2")
/// # .is_ok());
/// ```
///
/// # Implementation notes
///
/// The numbers and their corresponding weekday is as follows;
/// - 0 and 7 is Sunday
/// - 1 is Monday
/// - 2 is Tuesday
/// - 3 is Wednesday
/// - 4 is Thursday
/// - 5 is Friday
/// - 6 is Saturday
#[must_use = "CronTime requires validation that is dead code if this is not used and does nothing on its own"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CronTime {
    minute: CronSection,
    hour: CronSection,
    day_of_month: CronSection,
    month: CronSection,
    day_of_week: CronSection,

    /// 0 = first day (day 1) of the month,
    /// 30 = day 31st of the month
    valid_days: [bool; 31],
    /// 0 = january, 1 = february, ... 10 = november, 11 = december
    valid_months: [bool; 12],
    /// 0 = sunday, monday, tuesday, wednesday, thursday, friday, 6 = saturday
    valid_week_days: [bool; 7],
}

impl fmt::Display for CronSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CronSection::EveryTime => f.write_str("*"),
            CronSection::At(v) => f.write_fmt(format_args!("{v}")),
            CronSection::Multiple(v) => f.write_fmt(format_args!(
                "{}",
                v.iter()
                    .copied()
                    .enumerate()
                    .filter(|(_, v)| *v)
                    .map(|(i, _)| i)
                    .join(",")
            )),
            CronSection::EveryNth(v) => f.write_fmt(format_args!("*/{v}")),
            Self::StartingAtXEveryNth(x, n) => {
                f.write_fmt(format_args!("{x}/{n}"))
            }
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
            str if str.starts_with("*/") => {
                let (_, x) = str.split_at(2);
                if let Ok(y) = x.parse()
                    && y != 0
                {
                    // cannot do every 0th day
                    Some(CronSection::EveryNth(y))
                } else {
                    None
                }
            }
            str if let Some(split) = str.find('/') => {
                let (x, n) = str.split_at(split);
                let (_, n) = n.split_at(1);
                let (x, n) = (x.parse().ok()?, n.parse().ok()?);
                if n != 0 {
                    Some(CronSection::StartingAtXEveryNth(x, n))
                } else {
                    None
                }
            }
            str if let Ok(y) = str.parse() => Some(CronSection::At(y)),
            #[expect(clippy::indexing_slicing, clippy::string_slice)]
            str => {
                let mut arr = [false; 60];
                for part in str.split(',') {
                    if let Some(sp) = part.find('-') {
                        let (min, max) = part.split_at(sp);
                        let max = &max[1..];

                        if let Ok(min) = min.parse::<usize>()
                            && let Ok(max) = max.parse()
                            && min <= max
                            && max < 60
                        {
                            for item in arr
                                .iter_mut()
                                .take(max.saturating_add(1))
                                .skip(min)
                            {
                                *item = true;
                            }
                        } else {
                            return None;
                        }
                    } else if let Ok(y) = part.parse::<usize>()
                        && y < 60
                    {
                        arr[y] = true;
                    } else {
                        return None;
                    }
                }

                Some(CronSection::Multiple(arr))
            }
        }
    }

    /// Returns [`Ok`] if all inner values are within `min` and `max`, inclusive on both sides.
    ///
    /// # Errors
    ///
    /// Returns an error with the inner number that does not fit inside range.
    #[inline]
    pub const fn within(self, min: u8, max: u8) -> Result<(), u8> {
        debug_assert!(min <= max);
        debug_assert!(max <= 59);

        #[expect(
            clippy::as_conversions,
            clippy::indexing_slicing,
            clippy::arithmetic_side_effects
        )]
        match self {
            Self::At(x)
            | Self::EveryNth(x)
            | Self::StartingAtXEveryNth(x, _)
            | Self::StartingAtXEveryNth(_, x)
                if !(min <= x && x <= max) =>
            {
                Err(x)
            }
            Self::Multiple(x) => {
                let mut i = 0;
                while i < min {
                    if x[i as usize] {
                        return Err(i);
                    }
                    i += 1; // will ever wrap (min is u8, i must be lower than min)
                }

                if max != 59 {
                    i = max + 1;
                    while i < 60 {
                        if x[i as usize] {
                            return Err(i);
                        }
                        i += 1;
                    }
                }

                Ok(())
            }
            Self::EveryTime
            | Self::At(_)
            | Self::EveryNth(_)
            | Self::StartingAtXEveryNth(_, _) => Ok(()),
        }
    }
    /// Returns `true` if all inner values are within `min` and `max`, inclusive on both sides.
    ///
    /// Runs the [`within`] function, and just checks if it's [`Ok`] or not.
    ///
    /// [`within`]: Self::within
    #[inline]
    #[must_use]
    pub const fn within_bool(self, min: u8, max: u8) -> bool {
        self.within(min, max).is_ok()
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

    /// Returns `true` if the cron parsing error is [`DayOfMonthIsOutsideRange`].
    ///
    /// [`DayOfMonthIsOutsideRange`]: CronParsingError::DayOfMonthIsOutsideRange
    #[must_use]
    pub fn is_day_of_month_is_outside_range(&self) -> bool {
        matches!(self, Self::DayOfMonthIsOutsideRange(..))
    }

    /// Returns `true` if the cron parsing error is [`MonthIsNotCorrectFormat`].
    ///
    /// [`MonthIsNotCorrectFormat`]: CronParsingError::MonthIsNotCorrectFormat
    #[must_use]
    pub fn is_month_is_not_correct_format(&self) -> bool {
        matches!(self, Self::MonthIsNotCorrectFormat)
    }

    /// Returns `true` if the cron parsing error is [`MonthIsOutsideRange`].
    ///
    /// [`MonthIsOutsideRange`]: CronParsingError::MonthIsOutsideRange
    #[must_use]
    pub fn is_month_is_outside_range(&self) -> bool {
        matches!(self, Self::MonthIsOutsideRange(..))
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
macro_rules! get_valid {
    ($sec:ident, $max:literal) => {{
        match $sec {
            CronSection::EveryTime => [true; $max],
            CronSection::At(v) => {
                let mut arr = [false; $max];
                // v is between 1 and $max, as this is array indexing,
                // the value must be shifted left once (minus 1)
                arr[(v as usize).saturating_sub(1)] = true;
                arr
            }
            CronSection::Multiple(v) => {
                let mut arr = [false; $max];
                let mut i = 1;
                while i <= $max {
                    if v[i] {
                        // i's can only be between 1 and $max (exactly)
                        arr[i.saturating_sub(1)] = true;
                    }
                    i += 1;
                }

                arr
            }
            CronSection::EveryNth(v) => {
                let mut arr = [false; $max];

                let v = v as usize;
                let mut i = 0;
                while i < $max {
                    // i starts at 0, which is the defined behavior
                    // after that, every v-th value is true (until it goes over max)
                    arr[i] = true;
                    i = i.saturating_add(v);
                }

                arr
            }
            CronSection::StartingAtXEveryNth(x, n) => {
                let mut arr = [false; $max];

                let n = n as usize;
                let mut i = (x as usize).saturating_sub(1);
                while i < $max {
                    // i starts at x, which is the defined behavior
                    // after that, every n-th value is true (until it goes over max)
                    arr[i] = true;
                    i = i.saturating_add(n);
                }

                arr
            }
        }
    }}
}

impl CronTime {
    #[expect(
        clippy::indexing_slicing,
        clippy::as_conversions,
        reason = "some trait equivalents are unstable in const, all values are guaranteed to be valid conversions and all indexing is within bounds"
    )]
    /// Converts cron sections for day of month, month, and day of week into three arrays.
    /// Each entry into the array dictates whether that day/month/weekday is valid.
    ///
    /// First array is the day of month array, where day 1 is indexed at 0.
    /// Second array is the month array, where month 1 (January) is indexed at 0.
    /// Last array is the weekday array, where Sunday (0, Monday = 1) is indexed at 0.
    #[expect(clippy::arithmetic_side_effects)]
    const fn get_valid_times(
        day_of_month: CronSection,
        month: CronSection,
        day_of_week: CronSection,
    ) -> ([bool; 31], [bool; 12], [bool; 7]) {
        let valid_days = get_valid!(day_of_month, 31);
        let valid_months = get_valid!(month, 12);
        let valid_week_days = match day_of_week {
            CronSection::EveryTime => [true; 7],
            CronSection::At(v) => {
                let mut arr = [false; 7];
                // v is a number between 0 and 7, incl. on both sides,
                // however 7 and 0 means the same thing, so therefore the modulus is taken
                arr[v as usize % 7] = true;
                arr
            }
            CronSection::Multiple(v) => {
                let mut arr = [false; 7];
                let mut i = 0;
                while i < 7 {
                    if v[i] {
                        // i's can only be between 1 and 31 (exactly)
                        arr[i] = true;
                    }
                    i += 1;
                }

                arr
            }
            CronSection::EveryNth(v) => {
                let mut arr = [false; 7];

                let v = v as usize;
                let mut i = 0;
                while i <= 7 {
                    // i starts at 0, which is the defined behavior
                    // after that, every nth value is true (until it goes over max)
                    arr[i % 7] = true;
                    i = i.saturating_add(v);
                }

                arr
            }
            CronSection::StartingAtXEveryNth(x, n) => {
                let mut arr = [false; 7];

                let n = n as usize;
                let mut i = x as usize;
                while i <= 7 {
                    // i starts at x, which is the defined behavior
                    // after that, every n-th value is true (until it goes over max)
                    arr[i % 7] = true;
                    i = i.saturating_add(n);
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
        let split = cron.trim().split(' ');
        let vec: Vec<_> = split.collect();
        if vec.len() != 5 {
            return Err(CronParsingError::NotCorrectLength);
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
            CronParsingError::DayOfMonthIsOutsideRange
        );
        let month = unwrap_cron!(
            vec,
            3,  // vec pos
            1,  // min
            12, // max
            CronParsingError::MonthIsNotCorrectFormat,
            CronParsingError::MonthIsOutsideRange
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
    /// # Undefined behavior
    ///
    /// Is it undefined behavior, if any of these properties are not guaranteed;
    /// - `cron[0].within(0, 59).is_ok()`
    /// - `cron[1].within(0, 23).is_ok();`
    /// - `cron[2].within(1, 31).is_ok();`
    /// - `cron[3].within(1, 12.is_ok());`
    /// - `cron[4].within(0, 7).is_ok();`
    ///
    /// If any of this happens, the scheduler may---and likely will---get
    /// the wrong next date for evaluating a connected job. Therefore, it is recommended
    /// to have this code---or something similar---somewhere before calling the function;
    /// ```
    /// # use tobys_lib_core::cron::CronSection;
    /// # let cron = [CronSection::EveryTime, CronSection::EveryTime,
    /// # CronSection::EveryTime, CronSection::EveryTime, CronSection::EveryTime];
    /// debug_assert!(cron[0].within_bool(0, 59));
    /// debug_assert!(cron[1].within_bool(0, 23));
    /// debug_assert!(cron[2].within_bool(1, 31));
    /// debug_assert!(cron[3].within_bool(1, 12));
    /// debug_assert!(cron[4].within_bool(0, 7));
    /// ```
    ///
    /// # Panics
    ///
    /// Will panic if `cron` is not at least 5 long,
    /// or any of the sections have invalid numbers.
    #[expect(
        clippy::indexing_slicing,
        reason = "it is up to the caller to guarantee this doesn't happen"
    )]
    pub const fn new_unchecked(cron: &[CronSection]) -> Self {
        debug_assert!(cron[0].within_bool(0, 59));
        debug_assert!(cron[1].within_bool(0, 23));
        debug_assert!(cron[2].within_bool(1, 31));
        debug_assert!(cron[3].within_bool(1, 12));
        debug_assert!(cron[4].within_bool(0, 7));

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
                // remember to remove one, so month is in the range 0..=30 (not 1..=31)
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

    fn is_valid_time(&self, time: Time) -> bool {
        #[expect(
            clippy::as_conversions,
            clippy::cast_possible_wrap,
            clippy::cast_sign_loss,
            clippy::arithmetic_side_effects,
            clippy::indexing_slicing,
            reason = "all clippy lints are 100% guaranteed to NOT happen"
        )]
        let hour = match self.hour {
            CronSection::EveryTime => true,
            CronSection::At(v) => time.hour() == v as i8,
            CronSection::Multiple(v) => v[time.hour() as usize],
            CronSection::EveryNth(v) => (time.hour() as u8).is_multiple_of(v),
            CronSection::StartingAtXEveryNth(x, n) => {
                let h = time.hour() as u8;
                // h must be bigger than x (starting at)
                // and if so, h - x >= 0, therefore h - n will not wrap
                // and every nth starting at x is a multiple of n after subtracting x
                h >= x && (h - x).is_multiple_of(n)
            }
        };
        if !hour {
            return false;
        }

        #[expect(
            clippy::as_conversions,
            clippy::cast_possible_wrap,
            clippy::cast_sign_loss,
            clippy::arithmetic_side_effects,
            clippy::indexing_slicing,
            reason = "all clippy lints are 100% guaranteed to NOT happen"
        )]
        match self.minute {
            CronSection::EveryTime => true,
            CronSection::At(v) => time.minute() == v as i8,
            CronSection::Multiple(v) => v[time.minute() as usize],
            CronSection::EveryNth(v) => (time.minute() as u8).is_multiple_of(v),
            CronSection::StartingAtXEveryNth(x, n) => {
                let m = time.minute() as u8;
                // h must be bigger than x (starting at)
                // and if so, h - x >= 0, therefore h - n will not wrap
                // and every nth starting at x is a multiple of n after subtracting x
                m >= x && (m - x).is_multiple_of(n)
            }
        }
    }
    fn next_time<const ALLOW_DAY_WRAP: bool>(
        &self,
        curr: impl Into<DateTime>,
    ) -> DateTime {
        let curr = curr.into();
        #[expect(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap,
            clippy::arithmetic_side_effects,
            clippy::as_conversions,
            clippy::expect_used,
            clippy::indexing_slicing,
            reason = "all clippy lints are 100% guaranteed to NOT happen"
        )]
        let next_minute = match self.minute {
            // get the next minute and wrap around at 60
            CronSection::EveryTime => (curr.minute() + 1) % 60,
            CronSection::At(v) => v.try_into().expect("v is between 0 and 59"),
            CronSection::Multiple(v) => {
                let mut next = (curr.minute() + 1) as usize;

                while !v[next] {
                    next = (next + 1) % 60;
                }

                next as i8
            }
            CronSection::EveryNth(v) => {
                // get the next valid minute, using;
                // (ceil(current minute / v) * v) % 60
                //
                // ceil(current minute / v) * v) gets the smallest number x, st. v | x and x >= current minute
                //
                // % 60 makes sure the number wraps at 60 (is in 0..=59)

                ((curr.minute() as u32).div_ceil(v.into()) as i8 * v as i8) % 60
            }
            CronSection::StartingAtXEveryNth(x, n) => {
                let m = curr.minute() as u32;
                let x: u32 = x.into();
                let n: u32 = n.into();

                // if h < x, the next must be x (starting at)
                if m < x {
                    x as i8
                } else {
                    // else get next every nth number using the same logic as EveryNth branch,
                    // however subtracting x first to get new starting position (and then re-adding after)
                    let val = ((m - x).div_ceil(n) * n) + x;
                    // if val >= 60, wrap and set to starting at val (x)
                    // else val is within range
                    (if val >= 60 { x } else { val }) as i8
                }
            }
        };

        // if next_minute is less than current minute, hour MUST be moved forward
        let move_hour = next_minute <= curr.minute();

        #[expect(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap,
            clippy::arithmetic_side_effects,
            clippy::as_conversions,
            clippy::expect_used,
            clippy::indexing_slicing,
            reason = "all clippy lints are 100% guaranteed to NOT happen"
        )]
        let next_hour = match self.hour {
            CronSection::EveryTime => {
                if move_hour {
                    // wrap at 24, so val is in 0..=23
                    (curr.hour() + 1) % 24
                } else {
                    // minute is within same hour
                    curr.hour()
                }
            }
            CronSection::At(v) => v.try_into().expect("v is between 0 and 23"),
            CronSection::Multiple(v) => {
                let mut next = (curr.hour() + 1) as usize;

                while !v[next] {
                    next = (next + 1) % 24;
                }

                next as i8
            }
            CronSection::EveryNth(v) => {
                // get the next valid hour, using;
                // (ceil(current hour / v) * v) % 24
                //
                // ceil(current hour / v) * v) gets the smallest number x, st. v | x and x >= current hour
                //
                // % 24 makes sure the number wraps at 24 (is in 0..=23)
                ((curr.hour() as u32).div_ceil(v.into()) as i8 * v as i8) % 24
            }
            CronSection::StartingAtXEveryNth(x, n) => {
                let h = curr.hour() as u32;
                let x: u32 = x.into();
                let n: u32 = n.into();

                // if h < x, the next must be x (starting at)
                if h < x {
                    x as i8
                } else {
                    // else get next every nth number using the same logic as EveryNth branch,
                    // however subtracting x first to get new starting position (and then re-adding after)
                    let val = ((h - x).div_ceil(n) * n) + x;
                    // if val >= 24, wrap and set to starting at val (x)
                    // else val is within range
                    (if val >= 24 { x } else { val }) as i8
                }
            }
        };

        let next = datetime(
            curr.year(),
            curr.month(),
            curr.day(),
            next_hour,
            next_minute,
            0,
            0,
        );

        if ALLOW_DAY_WRAP && next <= curr {
            // if next is before or is current time, move to next day
            next.saturating_add(Span::new().days(1))
        } else {
            // else return valid NEXT time
            next
        }
    }

    #[must_use]
    pub(super) fn get_next_time(&self, curr: impl Into<DateTime>) -> DateTime {
        let mut curr = curr.into();
        #[cfg(debug_assertions)]
        let og = curr;

        // get the next valid time, and allow day wrap if needed
        curr = self.next_time::<true>(curr);
        if !self.is_valid_date(curr.date()) {
            // if not a valid date, move to the next valid date (day of month and month)
            // and set current time to 00:00
            curr = self
                .next_valid_date(curr.date())
                .to_datetime(time(0, 0, 0, 0));
        }
        while !self.is_valid_weekday(curr.date()) {
            // if it is a valid weekday, continue moving to next valid date
            // until one is.
            curr = self
                .next_valid_date(curr.date())
                .to_datetime(time(0, 0, 0, 0));
        }
        if !self.is_valid_time(curr.time()) {
            // this only happens if the date was moved via either the prior if or while statement
            // in that case, recalculate the next available (valid) time of day
            //
            // BUT, if the next is 00:00, DO NOT wrap to next day.
            curr = self.next_time::<false>(curr);
        }

        debug_assert_ne!(og, curr);

        curr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alias::format;

    // TODO; add tests to multiple and starting at syntax

    #[test]
    fn section_parse_test() {
        // tests that all types of sections can be parsed
        assert_eq!(CronSection::new("*"), Some(CronSection::EveryTime));
        assert_eq!(CronSection::new("2"), Some(CronSection::At(2)));
        assert_eq!(
            CronSection::new("2,5,8,1,59,30"),
            Some(CronSection::Multiple([
                false, true, true, false, false, true, false, false, true,
                false, false, false, false, false, false, false, false, false,
                false, false, false, false, false, false, false, false, false,
                false, false, false, true, false, false, false, false, false,
                false, false, false, false, false, false, false, false, false,
                false, false, false, false, false, false, false, false, false,
                false, false, false, false, false, true
            ]))
        );
        assert_eq!(
            CronSection::new("5,8,30-45"),
            Some(CronSection::Multiple([
                false, false, false, false, false, true, false, false, true,
                false, false, false, false, false, false, false, false, false,
                false, false, false, false, false, false, false, false, false,
                false, false, false, true, true, true, true, true, true, true,
                true, true, true, true, true, true, true, true, true, false,
                false, false, false, false, false, false, false, false, false,
                false, false, false, false
            ]))
        );
        assert_eq!(CronSection::new("*/5"), Some(CronSection::EveryNth(5)));
        assert_eq!(
            CronSection::new("56/45"),
            Some(CronSection::StartingAtXEveryNth(56, 45))
        );

        // tests some bad parsing
        assert_eq!(CronSection::new("woah"), None);
        assert_eq!(CronSection::new("*/+"), None);
        assert_eq!(CronSection::new("*/dawi9dja90"), None);
        assert_eq!(CronSection::new("*/0"), None);
    }

    #[test]
    fn time_parse_test() {
        // test parsing of 5 different sections in a cron time work
        let cron_time = CronTime::new("*/54 */12 * 4 1").unwrap();
        assert_eq!(cron_time.minute, CronSection::EveryNth(54));
        assert_eq!(cron_time.hour, CronSection::EveryNth(12));
        assert_eq!(cron_time.day_of_month, CronSection::EveryTime);
        assert_eq!(cron_time.month, CronSection::At(4));
        assert_eq!(cron_time.day_of_week, CronSection::At(1));

        // wrong length
        assert_eq!(CronTime::new(" "), Err(CronParsingError::NotCorrectLength));
        assert_eq!(
            CronTime::new("* "),
            Err(CronParsingError::NotCorrectLength)
        );
        assert_eq!(
            CronTime::new("* * "),
            Err(CronParsingError::NotCorrectLength)
        );
        assert_eq!(
            CronTime::new("* * * "),
            Err(CronParsingError::NotCorrectLength)
        );
        assert_eq!(
            CronTime::new("* * * * "),
            Err(CronParsingError::NotCorrectLength)
        );

        // test parsing but outside ranges fail individually
        assert_eq!(
            CronTime::new("woah * * * *"),
            Err(CronParsingError::MinuteIsNotCorrectFormat)
        );
        assert_eq!(
            CronTime::new("*/60 * * * *"),
            Err(CronParsingError::MinuteIsTooBig(60))
        );

        assert_eq!(
            CronTime::new("* woah * * *"),
            Err(CronParsingError::HourIsNotCorrectFormat)
        );
        assert_eq!(
            CronTime::new("* */24 * * *"),
            Err(CronParsingError::HourIsTooBig(24))
        );

        assert_eq!(
            CronTime::new("* * woah * *"),
            Err(CronParsingError::DayOfMonthIsNotCorrectFormat)
        );
        assert_eq!(
            CronTime::new("* * 0 * *"),
            Err(CronParsingError::DayOfMonthIsOutsideRange(0))
        );
        assert_eq!(
            CronTime::new("* * */32 * *"),
            Err(CronParsingError::DayOfMonthIsOutsideRange(32))
        );

        assert_eq!(
            CronTime::new("* * * woah *"),
            Err(CronParsingError::MonthIsNotCorrectFormat)
        );
        assert_eq!(
            CronTime::new("* * * 0 *"),
            Err(CronParsingError::MonthIsOutsideRange(0))
        );
        assert_eq!(
            CronTime::new("* * * */13 *"),
            Err(CronParsingError::MonthIsOutsideRange(13))
        );

        assert_eq!(
            CronTime::new("* * * * woah"),
            Err(CronParsingError::DayOfWeekIsNotCorrectFormat)
        );
        assert_eq!(
            CronTime::new("* * * * */8"),
            Err(CronParsingError::DayOfWeekIsTooBig(8))
        );
    }

    #[test]
    #[expect(unused_must_use)]
    fn valid_numbers_parse_test() {
        CronTime::new("0 * * * *").unwrap();
        for i in 1..=59 {
            CronTime::new(&format!("*/{i} * * * *")).unwrap();
            CronTime::new(&format!("{i}/{i} * * * *")).unwrap();
        }
        for i in 60..=u8::MAX {
            CronTime::new(&format!("{i} * * * *")).unwrap_err();
            CronTime::new(&format!("*/{i} * * * *")).unwrap_err();
            CronTime::new(&format!("{i}/{i} * * * *")).unwrap_err();
        }
        CronTime::new("0-59 * * * *").unwrap();
        CronTime::new("59-60 * * * *").unwrap_err();
        CronTime::new(&format!("60-{} * * * *", u8::MAX)).unwrap_err();
        CronTime::new(&format!("{} * * * *", (0..=59).join(","))).unwrap();
        CronTime::new(&format!("{} * * * *", (60..=u8::MAX).join(",")))
            .unwrap_err();

        CronTime::new("* 0 * * *").unwrap();
        for i in 1..=23 {
            CronTime::new(&format!("* {i} * * *")).unwrap();
            CronTime::new(&format!("* */{i} * * *")).unwrap();
            CronTime::new(&format!("* {i}/{i} * * *")).unwrap();
        }
        for i in 24..=u8::MAX {
            CronTime::new(&format!("* {i} * * *")).unwrap_err();
            CronTime::new(&format!("* */{i} * * *")).unwrap_err();
            CronTime::new(&format!("* {i}/{i} * * *")).unwrap_err();
        }
        CronTime::new("* 0-23 * * *").unwrap();
        CronTime::new("* 23-24 * * *").unwrap_err();
        CronTime::new(&format!("* 24-{} * * *", u8::MAX)).unwrap_err();
        CronTime::new(&format!("* {} * * *", (0..=23).join(","))).unwrap();
        CronTime::new(&format!("* {} * * *", (24..=u8::MAX).join(",")))
            .unwrap_err();

        for i in 1..=31 {
            CronTime::new(&format!("* * {i} * *")).unwrap();
            CronTime::new(&format!("* * */{i} * *")).unwrap();
            CronTime::new(&format!("* * {i}/{i} * *")).unwrap();
        }
        CronTime::new("* * 0 * *").unwrap_err();
        for i in 32..=u8::MAX {
            CronTime::new(&format!("* * {i} * *")).unwrap_err();
            CronTime::new(&format!("* * */{i} * *")).unwrap_err();
            CronTime::new(&format!("* * {i}/{i} * *")).unwrap_err();
        }
        CronTime::new("* * 0-1 * *").unwrap_err();
        CronTime::new("* * 1-31 * *").unwrap();
        CronTime::new("* * 31-32 * *").unwrap_err();
        CronTime::new(&format!("* * 32-{} * *", u8::MAX)).unwrap_err();
        CronTime::new(&format!("* * {} * *", (1..=31).join(","))).unwrap();
        CronTime::new(&format!("* * 0,{} * *", (32..=u8::MAX).join(",")))
            .unwrap_err();

        for i in 1..=12 {
            CronTime::new(&format!("* * * {i} *")).unwrap();
            CronTime::new(&format!("* * * */{i} *")).unwrap();
            CronTime::new(&format!("* * * {i}/{i} *")).unwrap();
        }
        CronTime::new("* * * 0 *").unwrap_err();
        for i in 13..=u8::MAX {
            CronTime::new(&format!("* * * {i} *")).unwrap_err();
            CronTime::new(&format!("* * * */{i} *")).unwrap_err();
            CronTime::new(&format!("* * * {i}/{i} *")).unwrap_err();
        }
        CronTime::new("* * * 0-1 *").unwrap_err();
        CronTime::new("* * * 1-12 *").unwrap();
        CronTime::new("* * * 12-13 *").unwrap_err();
        CronTime::new(&format!("* * * 13-{} *", u8::MAX)).unwrap_err();
        CronTime::new(&format!("* * * {} *", (1..=12).join(","))).unwrap();
        CronTime::new(&format!("* * * 0,{} *", (13..=u8::MAX).join(",")))
            .unwrap_err();

        CronTime::new("* * * * 0").unwrap();
        for i in 1..=7 {
            CronTime::new(&format!("* * * * {i}")).unwrap();
            CronTime::new(&format!("* * * * */{i}")).unwrap();
            CronTime::new(&format!("* * * * {i}/{i}")).unwrap();
        }
        for i in 8..=u8::MAX {
            CronTime::new(&format!("* * * * {i}")).unwrap_err();
            CronTime::new(&format!("* * * * */{i}")).unwrap_err();
            CronTime::new(&format!("* * * * {i}/{i}")).unwrap_err();
        }
        CronTime::new("* * * * 0-7").unwrap();
        CronTime::new("* * * * 7-8").unwrap_err();
        CronTime::new(&format!("* * * * 8-{}", u8::MAX)).unwrap_err();
        CronTime::new(&format!("* * * * {}", (0..=7).join(","))).unwrap();
        CronTime::new(&format!("* * * * {}", (8..=u8::MAX).join(",")))
            .unwrap_err();
    }

    #[test]
    fn next_time_test() {
        const BASE_LINE: DateTime =
            DateTime::constant(2026, 8, 30, 12, 00, 30, 00);

        // test that a lot of variants (of crontime) actually give the correct next date

        assert_eq!(
            CronTime::new("* * * * *").unwrap().get_next_time(BASE_LINE),
            DateTime::constant(2026, 8, 30, 12, 1, 0, 0)
        );
        assert_eq!(
            CronTime::new("0 * * * *").unwrap().get_next_time(BASE_LINE),
            DateTime::constant(2026, 8, 30, 13, 0, 0, 0)
        );
        assert_eq!(
            CronTime::new("0 12 * * *")
                .unwrap()
                .get_next_time(BASE_LINE),
            DateTime::constant(2026, 8, 31, 12, 0, 0, 0)
        );
        assert_eq!(
            CronTime::new("0 */21 */7 */3 *")
                .unwrap()
                .get_next_time(BASE_LINE),
            DateTime::constant(2026, 10, 1, 0, 0, 0, 0)
        );

        // next sunday
        assert_eq!(
            CronTime::new("0 12 * * 7")
                .unwrap()
                .get_next_time(BASE_LINE),
            DateTime::constant(2026, 9, 6, 12, 0, 0, 0)
        );

        // next sunday/wednesday/saturday that is the 13th of the month
        assert_eq!(
            CronTime::new("0 12 13 * */3")
                .unwrap()
                .get_next_time(BASE_LINE),
            DateTime::constant(2026, 9, 13, 12, 0, 0, 0)
        );

        // next friday the 13th
        assert_eq!(
            CronTime::new("0 12 13 * 5")
                .unwrap()
                .get_next_time(BASE_LINE),
            DateTime::constant(2026, 11, 13, 12, 0, 0, 0)
        );

        // next day divisible by 10 (10, 20, 30)
        assert_eq!(
            CronTime::new("0 12 10/10 * *")
                .unwrap()
                .get_next_time(BASE_LINE),
            DateTime::constant(2026, 9, 10, 12, 0, 0, 0)
        );

        // More tests could be argued that could be added,
        // however most lines in the parsing has been commented on why they will always be true.
    }
}
