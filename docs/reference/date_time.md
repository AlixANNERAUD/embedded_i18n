# Date and Time

Localized `strftime`-style formatting of Unix timestamps, without `libc`, without allocation and without a time zone database.

## API

| Item | Description |
| --- | --- |
| [`DateTimeFormatter`]`::new(DateTimeLocale)` | `const fn`: a formatter with the names and patterns of a locale, in UTC |
| `DateTimeFormatter::with_locale(DateTimeLocale)` | `const fn`: the same formatter with another locale |
| `DateTimeFormatter::with_utc_offset(i32)` | `const fn`: the same formatter with a fixed UTC offset, in seconds, also printed by `%z` |
| `DateTimeFormatter::locale()`, `DateTimeFormatter::utc_offset()` | the locale data and the offset of the formatter |
| [`DateTimeFormatter::write`]`(&mut impl Write, i64, &str)` | writes a timestamp formatted with a pattern into a [`core::fmt::Write`] |
| [`DateTimeFormatter::display`]`(i64, &str)` | `const fn`: a value implementing [`core::fmt::Display`], for `write!`, `format_args!` or logging, without allocation |
| [`DateTimeFormatter::format`]`(i64, &str)` | with the `alloc` feature, a `String` |
| `write_unix_timestamp(&mut impl Write, i64, &str)` | writes a timestamp in English and UTC |
| `format_unix_timestamp(i64, &str)` | with the `alloc` feature, the same as a `String` |
| [`date_time_locale`]`(&str)` | `const fn`: the `DateTimeLocale` of a locale, English if it is not covered |
| [`find_date_time_locale`]`(&str)` | `const fn`: the `DateTimeLocale` of a locale, `None` if it is not covered |
| `DateTimeLocale` | month and weekday names, AM/PM markers and default date and time patterns of a locale, with public fields |
| `ENGLISH` | the English `DateTimeLocale` |

Timestamps are seconds since the Unix epoch, as `i64`.

```rust
use embedded_i18n::{date_time_locale, DateTimeFormatter};

const PARIS: DateTimeFormatter = DateTimeFormatter::new(date_time_locale("fr"))
    .with_utc_offset(2 * 3600);

let mut output = String::new();
PARIS.write(&mut output, 1_791_884_800, "%x %H:%M %z")?;
assert_eq!(output, "13/10/2026 11:46 +0200");

let text = format!("Mis à jour le {}", PARIS.display(1_791_884_800, "%d %B"));
assert_eq!(text, "Mis à jour le 13 octobre");
# Ok::<(), core::fmt::Error>(())
```

## Tokens

| Token | Description |
| --- | --- |
| `%Y` | Year with century (`2026`) |
| `%y` | Year without century (`00`–`99`) |
| `%m` | Zero-padded month (`01`–`12`) |
| `%d` | Zero-padded day (`01`–`31`) |
| `%e` | Space-padded day (` 1`–`31`) |
| `%H` | Zero-padded hour, 24-hour clock (`00`–`23`) |
| `%I` | Zero-padded hour, 12-hour clock (`01`–`12`) |
| `%M` | Zero-padded minute (`00`–`59`) |
| `%S` | Zero-padded second (`00`–`59`) |
| `%p` | Localized AM / PM marker (`AM`, `午後`, `오전`...) |
| `%B` | Localized month name, in the form used in dates (`октября`) |
| `%OB` | Localized standalone month name (`октябрь`) |
| `%b`, `%h` | Localized abbreviated month name |
| `%A` | Localized weekday name |
| `%a` | Localized abbreviated weekday name |
| `%u` | Weekday number from Monday (`1`–`7`) |
| `%w` | Weekday number from Sunday (`0`–`6`) |
| `%x` | Localized date (`10/13/2026`, `13/10/2026`, `13.10.2026`...) |
| `%X` | Localized time (`09:46:40 AM`, `09:46:40`...) |
| `%c` | Localized date and time (`%x %X`) |
| `%z` | UTC offset (`+0200`, `-0330`) |
| `%%` | Literal `%` |

Unknown tokens are written as is (`%Q` stays `%Q`), and a trailing `%` is kept.

`%B` and `%OB` differ in languages that decline month names, such as Russian (`13 октября`, but `октябрь` alone):

```rust
use embedded_i18n::{date_time_locale, DateTimeFormatter};

let russian = DateTimeFormatter::new(date_time_locale("ru"));
let mut output = String::new();
russian.write(&mut output, 1_791_884_800, "%d %B / %OB")?;
assert_eq!(output, "13 октября / октябрь");
# Ok::<(), core::fmt::Error>(())
```

## Locales

Names and patterns come from a compile-time table covering `ar`, `be`, `cs`, `da`, `de`, `el`, `en`, `es`, `fr`, `he`, `hi`, `hu`, `it`, `ja`, `ko`, `nb`, `nl`, `nn`, `no`, `pl`, `pt`, `ro`, `ru`, `sk`, `sv`, `th`, `tr`, `uk`, `vi` and `zh`, with regional date patterns for `en-GB`, `en-IE`, `en-AU`, `en-NZ`, `en-IN`, `en-CA`, `fr-CA` and `fr-CH`.

[`date_time_locale`] and [`find_date_time_locale`] are `const fn`:

- called with a constant, such as the [`locale!`] of a crate embedding one locale, they are resolved at compile time and only the strings of that locale end up in the firmware;
- called with a runtime string, such as [`locale!`] with several embedded locales, they link the whole table, about 30 locales of names.

```rust
use embedded_i18n::{date_time_locale, DateTimeFormatter};

embedded_i18n::i18n!("locales");

const FORMATTER: DateTimeFormatter =
    DateTimeFormatter::new(date_time_locale(embedded_i18n::locale!()));

let mut output = String::new();
FORMATTER.write(&mut output, 1_791_884_800, "%A %d %B %Y, %X")?;
assert_eq!(output, "Tuesday 13 October 2026, 09:46:40 AM");
# Ok::<(), core::fmt::Error>(())
```

## Limits

- Digits are always ASCII.
- The calendar is always Gregorian.
- Weeks are not numbered.
- The UTC offset is fixed: daylight saving time and time zone rules are left to the application.

[`DateTimeFormatter`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/struct.DateTimeFormatter.html
[`DateTimeFormatter::write`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/struct.DateTimeFormatter.html#method.write
[`DateTimeFormatter::display`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/struct.DateTimeFormatter.html#method.display
[`DateTimeFormatter::format`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/struct.DateTimeFormatter.html#method.format
[`locale!`]: runtime_api.md
[`date_time_locale`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.date_time_locale.html
[`find_date_time_locale`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.find_date_time_locale.html
[`core::fmt::Write`]: https://doc.rust-lang.org/core/fmt/trait.Write.html
[`core::fmt::Display`]: https://doc.rust-lang.org/core/fmt/trait.Display.html
