use core::fmt::{self, Write};

use crate::date_time::{DateTimeLocale, ENGLISH};

#[cfg(feature = "alloc")]
use alloc::string::String;

fn decompose_unix_timestamp(unix_timestamp: i64) -> (u16, u8, u8, u8, u8, u8) {
    const SECONDS_IN_MINUTE: i64 = 60;
    const SECONDS_IN_HOUR: i64 = 60 * SECONDS_IN_MINUTE;
    const SECONDS_IN_DAY: i64 = 24 * SECONDS_IN_HOUR;
    const DAYS_IN_YEAR: i64 = 365;
    const DAYS_IN_LEAP_YEAR: i64 = 366;

    let mut year: i64 = 1970;
    let mut days_since_epoch = unix_timestamp.div_euclid(SECONDS_IN_DAY);
    let mut remaining_seconds = unix_timestamp.rem_euclid(SECONDS_IN_DAY);

    while days_since_epoch
        >= if is_leap_year(year) {
            DAYS_IN_LEAP_YEAR
        } else {
            DAYS_IN_YEAR
        }
    {
        days_since_epoch -= if is_leap_year(year) {
            DAYS_IN_LEAP_YEAR
        } else {
            DAYS_IN_YEAR
        };
        year += 1;
    }

    while days_since_epoch < 0 {
        year -= 1;
        days_since_epoch += if is_leap_year(year) {
            DAYS_IN_LEAP_YEAR
        } else {
            DAYS_IN_YEAR
        };
    }

    let mut month = 0;
    while days_since_epoch >= days_in_month(year, month) {
        days_since_epoch -= days_in_month(year, month);
        month += 1;
    }

    let day = days_since_epoch + 1;

    let hour = remaining_seconds / SECONDS_IN_HOUR;
    remaining_seconds %= SECONDS_IN_HOUR;
    let minute = remaining_seconds / SECONDS_IN_MINUTE;
    let second = remaining_seconds % SECONDS_IN_MINUTE;

    (
        year as u16,
        month as u8 + 1,
        day as u8,
        hour as u8,
        minute as u8,
        second as u8,
    )
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(year: i64, month: usize) -> i64 {
    const DAYS_IN_MONTH: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    if month == 1 && is_leap_year(year) {
        29
    } else {
        DAYS_IN_MONTH[month]
    }
}

/// Formats Unix timestamps with `strftime`-style patterns, localized and
/// without allocation.
///
/// | Token | Output |
/// | --- | --- |
/// | `%Y`, `%y` | Year, with and without century |
/// | `%m`, `%d`, `%e` | Month, day, space-padded day |
/// | `%H`, `%I`, `%M`, `%S` | Hour (24 and 12-hour), minute, second |
/// | `%p` | Localized AM / PM marker |
/// | `%B`, `%OB`, `%b` / `%h` | Localized month name, standalone, abbreviated |
/// | `%A`, `%a` | Localized weekday name, abbreviated |
/// | `%u`, `%w` | Weekday number from Monday (1-7), from Sunday (0-6) |
/// | `%x`, `%X`, `%c` | Localized date, time, date and time |
/// | `%z` | UTC offset (`+0200`) |
/// | `%%` | `%` |
///
/// ```
/// use embedded_i18n::{date_time_locale, DateTimeFormatter};
///
/// let formatter = DateTimeFormatter::new(date_time_locale("fr")).with_utc_offset(2 * 3600);
///
/// let mut output = String::new();
/// formatter.write(&mut output, 1_791_884_800, "%A %d %B %Y, %H:%M %z")?;
/// assert_eq!(output, "mardi 13 octobre 2026, 11:46 +0200");
/// # Ok::<(), core::fmt::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateTimeFormatter {
    locale: DateTimeLocale,
    offset: i32,
}

impl DateTimeFormatter {
    /// Creates a formatter for the given names and patterns, in UTC.
    pub const fn new(locale: DateTimeLocale) -> Self {
        DateTimeFormatter { locale, offset: 0 }
    }

    /// Uses the names and patterns of another locale.
    pub const fn with_locale(self, locale: DateTimeLocale) -> Self {
        DateTimeFormatter { locale, ..self }
    }

    /// Shifts timestamps by a fixed UTC offset, in seconds, also printed by `%z`.
    pub const fn with_utc_offset(self, seconds: i32) -> Self {
        DateTimeFormatter {
            offset: seconds,
            ..self
        }
    }

    /// Returns the names and patterns used by this formatter.
    pub const fn locale(&self) -> &DateTimeLocale {
        &self.locale
    }

    /// Returns the UTC offset of this formatter, in seconds.
    pub const fn utc_offset(&self) -> i32 {
        self.offset
    }

    /// Writes a Unix timestamp formatted with a pattern.
    pub fn write<W: Write>(
        &self,
        writer: &mut W,
        unix_timestamp: i64,
        pattern: &str,
    ) -> fmt::Result {
        let local = unix_timestamp.saturating_add(i64::from(self.offset));
        let (year, month, day, hour, minute, second) = decompose_unix_timestamp(local);
        let fields = Fields {
            year,
            month,
            day,
            hour,
            minute,
            second,
            weekday: weekday(local),
        };
        self.write_pattern(writer, &fields, pattern, true)
    }

    /// Returns a value implementing [`fmt::Display`], to format a timestamp with
    /// `write!` or `format_args!` without allocation.
    ///
    /// ```
    /// use embedded_i18n::{DateTimeFormatter, date_time_locale};
    ///
    /// let formatter = DateTimeFormatter::new(date_time_locale("en"));
    /// let text = format!("Updated {}", formatter.display(0, "%Y-%m-%d"));
    /// assert_eq!(text, "Updated 1970-01-01");
    /// ```
    pub const fn display<'a>(
        &'a self,
        unix_timestamp: i64,
        pattern: &'a str,
    ) -> DateTimeDisplay<'a> {
        DateTimeDisplay {
            formatter: self,
            unix_timestamp,
            pattern,
        }
    }

    /// Formats a Unix timestamp into a `String`. Available with the `alloc`
    /// feature.
    #[cfg(feature = "alloc")]
    pub fn format(&self, unix_timestamp: i64, pattern: &str) -> String {
        let mut output = String::with_capacity(pattern.len() + 16);
        let _ = self.write(&mut output, unix_timestamp, pattern);
        output
    }

    fn write_pattern<W: Write>(
        &self,
        writer: &mut W,
        fields: &Fields,
        pattern: &str,
        expand: bool,
    ) -> fmt::Result {
        let locale = &self.locale;
        let mut characters = pattern.chars();

        while let Some(character) = characters.next() {
            if character != '%' {
                writer.write_char(character)?;
                continue;
            }

            match characters.next() {
                Some('Y') => write!(writer, "{:04}", fields.year)?,
                Some('y') => write!(writer, "{:02}", fields.year % 100)?,
                Some('m') => write!(writer, "{:02}", fields.month)?,
                Some('d') => write!(writer, "{:02}", fields.day)?,
                Some('e') => write!(writer, "{:2}", fields.day)?,
                Some('H') => write!(writer, "{:02}", fields.hour)?,
                Some('I') => write!(writer, "{:02}", hour_12(fields.hour))?,
                Some('M') => write!(writer, "{:02}", fields.minute)?,
                Some('S') => write!(writer, "{:02}", fields.second)?,
                Some('p') => writer.write_str(if fields.hour < 12 {
                    locale.am
                } else {
                    locale.pm
                })?,
                Some('B') => writer.write_str(locale.months[usize::from(fields.month - 1)])?,
                Some('b' | 'h') => {
                    writer.write_str(locale.months_abbreviated[usize::from(fields.month - 1)])?
                }
                Some('A') => writer.write_str(locale.weekdays[usize::from(fields.weekday)])?,
                Some('a') => {
                    writer.write_str(locale.weekdays_abbreviated[usize::from(fields.weekday)])?
                }
                Some('u') => write!(
                    writer,
                    "{}",
                    if fields.weekday == 0 {
                        7
                    } else {
                        fields.weekday
                    }
                )?,
                Some('w') => write!(writer, "{}", fields.weekday)?,
                Some('z') => {
                    let sign = if self.offset < 0 { '-' } else { '+' };
                    let minutes = self.offset.unsigned_abs() / 60;
                    write!(writer, "{}{:02}{:02}", sign, minutes / 60, minutes % 60)?
                }
                Some('O') => match characters.clone().next() {
                    Some('B') => {
                        characters.next();
                        writer.write_str(locale.months_standalone[usize::from(fields.month - 1)])?
                    }
                    _ => writer.write_str("%O")?,
                },
                Some('x') if expand => self.write_pattern(writer, fields, locale.date, false)?,
                Some('X') if expand => self.write_pattern(writer, fields, locale.time, false)?,
                Some('c') if expand => {
                    self.write_pattern(writer, fields, locale.date, false)?;
                    writer.write_char(' ')?;
                    self.write_pattern(writer, fields, locale.time, false)?
                }
                Some('%') => writer.write_char('%')?,
                Some(other) => {
                    writer.write_char('%')?;
                    writer.write_char(other)?;
                }
                None => writer.write_char('%')?,
            }
        }

        Ok(())
    }
}

struct Fields {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    weekday: u8,
}

fn weekday(unix_timestamp: i64) -> u8 {
    (unix_timestamp.div_euclid(86_400) + 4).rem_euclid(7) as u8
}

/// Timestamp formatted on display, returned by [`DateTimeFormatter::display`].
pub struct DateTimeDisplay<'a> {
    formatter: &'a DateTimeFormatter,
    unix_timestamp: i64,
    pattern: &'a str,
}

impl fmt::Display for DateTimeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.formatter.write(f, self.unix_timestamp, self.pattern)
    }
}

/// Writes a Unix timestamp formatted with a pattern, in English and UTC. For
/// another locale, use [`DateTimeFormatter`]. See [`DateTimeFormatter`] for the tokens.
///
/// ```
/// let mut output = String::new();
/// embedded_i18n::write_unix_timestamp(&mut output, 0, "%a %d %b %Y %H:%M")?;
/// assert_eq!(output, "Thu 01 Jan 1970 00:00");
/// # Ok::<(), core::fmt::Error>(())
/// ```
pub fn write_unix_timestamp<W: Write>(
    writer: &mut W,
    unix_timestamp: i64,
    pattern: &str,
) -> fmt::Result {
    DateTimeFormatter::new(ENGLISH).write(writer, unix_timestamp, pattern)
}

/// Formats a Unix timestamp into a `String`, in English and UTC. For another
/// locale, use [`DateTimeFormatter`]. Available with the `alloc` feature.
#[cfg(feature = "alloc")]
pub fn format_unix_timestamp(unix_timestamp: i64, pattern: &str) -> String {
    DateTimeFormatter::new(ENGLISH).format(unix_timestamp, pattern)
}

const fn hour_12(hour_24: u8) -> u8 {
    match hour_24 % 12 {
        0 => 12,
        value => value,
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;

    #[test]
    fn format_24_hour_time() {
        let timestamp = 13 * 3600 + 5 * 60;
        assert_eq!(format_unix_timestamp(timestamp, "%H:%M"), "13:05");
    }

    #[test]
    fn format_12_hour_time_with_am_pm() {
        let midnight = 0;
        let afternoon = 13 * 3600 + 5 * 60;

        assert_eq!(format_unix_timestamp(midnight, "%I:%M %p"), "12:00 AM");
        assert_eq!(format_unix_timestamp(afternoon, "%I:%M %p"), "01:05 PM");
    }

    #[test]
    fn format_date_and_time() {
        assert_eq!(
            format_unix_timestamp(0, "%Y-%m-%d %H:%M:%S"),
            "1970-01-01 00:00:00"
        );
    }

    #[test]
    fn format_negative_unix_time() {
        assert_eq!(
            format_unix_timestamp(-1, "%Y-%m-%d %H:%M:%S"),
            "1969-12-31 23:59:59"
        );
    }

    #[test]
    fn unix_epoch_is_correct() {
        assert_eq!(decompose_unix_timestamp(0), (1970, 1, 1, 0, 0, 0));
    }

    #[test]
    fn one_second_before_epoch_is_correct() {
        assert_eq!(decompose_unix_timestamp(-1), (1969, 12, 31, 23, 59, 59));
    }

    #[test]
    fn one_day_before_epoch_is_correct() {
        assert_eq!(decompose_unix_timestamp(-86_400), (1969, 12, 31, 0, 0, 0));
    }

    #[test]
    fn leap_day_2024_is_correct() {
        assert_eq!(
            decompose_unix_timestamp(1_709_164_800),
            (2024, 2, 29, 0, 0, 0)
        );
    }

    #[test]
    fn leap_year_rules_are_correct() {
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
    }

    #[test]
    fn february_days_are_correct() {
        assert_eq!(days_in_month(2024, 1), 29);
        assert_eq!(days_in_month(2023, 1), 28);
    }

    #[test]
    fn only_time() {
        assert_eq!(format_unix_timestamp(3661, "%H:%M:%S"), "01:01:01");
    }

    #[test]
    fn only_date() {
        assert_eq!(format_unix_timestamp(0, "%Y-%m-%d"), "1970-01-01");
    }

    #[test]
    fn literal_percent() {
        assert_eq!(format_unix_timestamp(0, "100%%"), "100%");
    }

    #[test]
    fn unknown_format_token() {
        assert_eq!(format_unix_timestamp(0, "%q"), "%q");
    }

    #[test]
    fn partial_percent_at_end() {
        assert_eq!(format_unix_timestamp(0, "end%"), "end%");
    }

    #[test]
    fn mixed_literal_and_format() {
        assert_eq!(
            format_unix_timestamp(0, "Date: %Y-%m-%d, Time: %H:%M:%S"),
            "Date: 1970-01-01, Time: 00:00:00"
        );
    }

    #[test]
    fn noon_is_12_pm() {
        let noon = 12 * 3600;
        assert_eq!(decompose_unix_timestamp(noon), (1970, 1, 1, 12, 0, 0));
    }

    #[test]
    fn hour_12_midnight() {
        assert_eq!(hour_12(0), 12);
    }

    #[test]
    fn hour_12_noon() {
        assert_eq!(hour_12(12), 12);
    }

    #[test]
    fn hour_12_various() {
        assert_eq!(hour_12(1), 1);
        assert_eq!(hour_12(11), 11);
        assert_eq!(hour_12(13), 1);
        assert_eq!(hour_12(23), 11);
    }

    #[test]
    fn format_12_hour_noon() {
        let noon = 12 * 3600;
        assert_eq!(format_unix_timestamp(noon, "%I:%M %p"), "12:00 PM");
    }

    #[test]
    fn format_12_hour_midnight() {
        assert_eq!(format_unix_timestamp(0, "%I:%M %p"), "12:00 AM");
    }

    #[test]
    fn year_2000_timestamp() {
        let ts = 946684800; // 2000-01-01T00:00:00Z
        assert_eq!(format_unix_timestamp(ts, "%Y"), "2000");
    }

    #[test]
    fn leap_year_2000() {
        let ts = 951782400; // 2000-02-29T00:00:00Z
        assert_eq!(decompose_unix_timestamp(ts), (2000, 2, 29, 0, 0, 0));
        assert_eq!(format_unix_timestamp(ts, "%Y-%m-%d"), "2000-02-29");
    }

    #[test]
    fn non_leap_year_1900() {
        assert!(!is_leap_year(1900));
    }

    #[test]
    fn century_boundary_2000_is_leap() {
        assert!(is_leap_year(2000));
    }

    #[test]
    fn year_2038_problem() {
        let ts = 2147483648i64; // just after 2038-01-19T03:14:07Z
        assert_eq!(format_unix_timestamp(ts, "%Y"), "2038");
    }

    #[test]
    fn empty_pattern() {
        assert_eq!(format_unix_timestamp(0, ""), "");
    }

    #[test]
    fn no_format_tokens() {
        assert_eq!(
            format_unix_timestamp(0, "just literal text"),
            "just literal text"
        );
    }

    #[test]
    fn negative_timestamp_bce_date() {
        let ts = -31536000i64; // 1969-01-01
        assert_eq!(format_unix_timestamp(ts, "%Y-%m-%d"), "1969-01-01");
    }

    #[test]
    fn multiple_percent_escape() {
        assert_eq!(format_unix_timestamp(0, "%%Y%%m%%d%%"), "%Y%m%d%");
    }
}

#[cfg(test)]
mod writer_tests {
    use super::*;

    struct Buffer {
        bytes: [u8; 64],
        length: usize,
    }

    impl Buffer {
        fn new() -> Self {
            Buffer {
                bytes: [0; 64],
                length: 0,
            }
        }

        fn as_str(&self) -> &str {
            core::str::from_utf8(&self.bytes[..self.length]).unwrap()
        }
    }

    impl Write for Buffer {
        fn write_str(&mut self, value: &str) -> fmt::Result {
            let end = self.length + value.len();
            if end > self.bytes.len() {
                return Err(fmt::Error);
            }
            self.bytes[self.length..end].copy_from_slice(value.as_bytes());
            self.length = end;
            Ok(())
        }
    }

    #[test]
    fn write_epoch() {
        let mut buffer = Buffer::new();
        write_unix_timestamp(&mut buffer, 0, "%Y-%m-%d %H:%M:%S").unwrap();
        assert_eq!(buffer.as_str(), "1970-01-01 00:00:00");
    }

    #[test]
    fn write_12_hour_time() {
        let mut buffer = Buffer::new();
        write_unix_timestamp(&mut buffer, 13 * 3600 + 5 * 60, "%I:%M %p %%").unwrap();
        assert_eq!(buffer.as_str(), "01:05 PM %");
    }

    #[test]
    fn write_propagates_errors() {
        let mut buffer = Buffer::new();
        let pattern = "%Y-%m-%d %H:%M:%S %Y-%m-%d %H:%M:%S %Y-%m-%d %H:%M:%S %Y-%m-%d";
        assert!(write_unix_timestamp(&mut buffer, 0, pattern).is_err());
    }
}

#[cfg(test)]
mod localized_tests {
    use super::*;
    use crate::date_time::date_time_locale;

    struct Buffer {
        bytes: [u8; 128],
        length: usize,
    }

    impl Write for Buffer {
        fn write_str(&mut self, value: &str) -> fmt::Result {
            let end = self.length + value.len();
            if end > self.bytes.len() {
                return Err(fmt::Error);
            }
            self.bytes[self.length..end].copy_from_slice(value.as_bytes());
            self.length = end;
            Ok(())
        }
    }

    fn render(formatter: &DateTimeFormatter, unix_timestamp: i64, pattern: &str) -> Buffer {
        let mut buffer = Buffer {
            bytes: [0; 128],
            length: 0,
        };
        formatter
            .write(&mut buffer, unix_timestamp, pattern)
            .unwrap();
        buffer
    }

    fn assert_renders(
        formatter: &DateTimeFormatter,
        unix_timestamp: i64,
        pattern: &str,
        expected: &str,
    ) {
        let buffer = render(formatter, unix_timestamp, pattern);
        assert_eq!(
            core::str::from_utf8(&buffer.bytes[..buffer.length]).unwrap(),
            expected
        );
    }

    const OCTOBER_13_2026: i64 = 1_791_884_800;

    #[test]
    fn english_names_and_patterns() {
        let formatter = DateTimeFormatter::new(ENGLISH);
        assert_renders(
            &formatter,
            0,
            "%A %a %B %b %h",
            "Thursday Thu January Jan Jan",
        );
        assert_renders(&formatter, OCTOBER_13_2026, "%c", "10/13/2026 09:46:40 AM");
        assert_renders(&formatter, OCTOBER_13_2026, "%y %e %u %w", "26 13 2 2");
    }

    #[test]
    fn french_names_and_patterns() {
        let formatter = DateTimeFormatter::new(date_time_locale("fr"));
        assert_renders(
            &formatter,
            OCTOBER_13_2026,
            "%A %d %B %Y, %X",
            "mardi 13 octobre 2026, 09:46:40",
        );
        assert_renders(&formatter, OCTOBER_13_2026, "%x", "13/10/2026");
        assert_renders(&formatter, OCTOBER_13_2026, "%a %b", "mar. oct.");
    }

    #[test]
    fn standalone_months() {
        let formatter = DateTimeFormatter::new(date_time_locale("ru"));
        assert_renders(
            &formatter,
            OCTOBER_13_2026,
            "%d %B / %OB %Y",
            "13 октября / октябрь 2026",
        );
        assert_renders(&formatter, OCTOBER_13_2026, "%Oq", "%Oq");
    }

    #[test]
    fn localized_am_pm() {
        let formatter = DateTimeFormatter::new(date_time_locale("ko"));
        assert_renders(&formatter, OCTOBER_13_2026, "%X", "오전 09:46:40");
        assert_renders(&formatter, OCTOBER_13_2026 + 6 * 3600, "%p", "오후");
    }

    #[test]
    fn utc_offset_shifts_time_and_prints_z() {
        let paris = DateTimeFormatter::new(ENGLISH).with_utc_offset(2 * 3600);
        assert_renders(&paris, OCTOBER_13_2026, "%H:%M %z", "11:46 +0200");
        let newfoundland = DateTimeFormatter::new(ENGLISH).with_utc_offset(-(3 * 3600 + 1800));
        assert_renders(
            &newfoundland,
            OCTOBER_13_2026,
            "%d %H:%M %z",
            "13 06:16 -0330",
        );
        let before_midnight = DateTimeFormatter::new(ENGLISH).with_utc_offset(-10 * 3600);
        assert_renders(&before_midnight, OCTOBER_13_2026, "%A %d", "Monday 12");
        assert_renders(&DateTimeFormatter::new(ENGLISH), 0, "%z", "+0000");
    }

    #[test]
    fn weekday_before_epoch() {
        assert_renders(&DateTimeFormatter::new(ENGLISH), -1, "%A", "Wednesday");
        assert_renders(
            &DateTimeFormatter::new(ENGLISH),
            -86_400 * 4,
            "%A",
            "Sunday",
        );
    }

    #[test]
    fn patterns_do_not_expand_recursively() {
        let formatter = DateTimeFormatter::new(DateTimeLocale {
            date: "%x",
            ..date_time_locale("en")
        });
        assert_renders(&formatter, 0, "%x", "%x");
    }

    #[test]
    fn display_adapter() {
        let formatter = DateTimeFormatter::new(ENGLISH);
        let mut buffer = Buffer {
            bytes: [0; 128],
            length: 0,
        };
        write!(buffer, "[{}]", formatter.display(0, "%Y-%m-%d")).unwrap();
        assert_eq!(&buffer.bytes[..buffer.length], b"[1970-01-01]");
    }

    #[test]
    fn reports_writer_errors() {
        let mut buffer = Buffer {
            bytes: [0; 128],
            length: 120,
        };
        assert!(
            DateTimeFormatter::new(ENGLISH)
                .write(&mut buffer, 0, "%A %B")
                .is_err()
        );
    }
}
