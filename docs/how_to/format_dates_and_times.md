# How to Format Dates and Times

[`DateTimeFormatter`] formats Unix timestamps with `strftime`-style patterns, without `libc`, without allocation and without a time zone database.
This guide formats dates in the language of the firmware, in a local time zone, and into a display buffer.

## Format in the Language of the Firmware

Pass the locale of the crate to [`date_time_locale`], and the result to [`DateTimeFormatter::new`]:

```rust
use embedded_i18n::{date_time_locale, DateTimeFormatter};

embedded_i18n::i18n!("locales");

const FORMATTER: DateTimeFormatter =
    DateTimeFormatter::new(date_time_locale(embedded_i18n::locale!()));

let mut output = String::new();
FORMATTER.write(&mut output, 1_791_884_800, "%A %d %B %Y, %X")?;
assert_eq!(output, "Tuesday 13 October 2026, 09:46:40 AM");
// default_locale = "fr": "mardi 13 octobre 2026, 09:46:40"
# Ok::<(), core::fmt::Error>(())
```

With a single embedded locale, the locale is resolved at compile time, and only its month and weekday names end up in the firmware.

With [several embedded locales][how-to-switch-language], [`locale!`] reads the current locale at runtime and is not `const`.
Build the formatter with `let` where you format, so that it follows the current locale:

```rust,standalone_crate
use embedded_i18n::{date_time_locale, set_locale, DateTimeFormatter};

embedded_i18n::i18n!("locales", default_locale = "en", available_locales = ["en", "fr"]);

fn main() -> core::fmt::Result {
    set_locale("fr");
    let formatter = DateTimeFormatter::new(date_time_locale(embedded_i18n::locale!()));

    let mut output = String::new();
    formatter.write(&mut output, 1_791_884_800, "%A %d %B %Y")?;
    assert_eq!(output, "mardi 13 octobre 2026");
    Ok(())
}
```

## Use a Local Time Zone

Timestamps are in UTC.
Shift them with a fixed offset, in seconds, which `%z` prints:

```rust
use embedded_i18n::{date_time_locale, DateTimeFormatter};

const PARIS: DateTimeFormatter = DateTimeFormatter::new(date_time_locale("fr"))
    .with_utc_offset(2 * 3600);

let mut output = String::new();
PARIS.write(&mut output, 1_791_884_800, "%x %H:%M %z")?;
assert_eq!(output, "13/10/2026 11:46 +0200");
# Ok::<(), core::fmt::Error>(())
```

The offset is fixed: daylight saving time and time zone rules are left to the application, which usually gets the offset from the network or its configuration.
Build a new formatter with `with_utc_offset` when the offset changes.

## Insert a Date Into a Message

[`DateTimeFormatter::display`] returns a value implementing [`core::fmt::Display`], for `write!`, `format_args!` or logging, still without allocation:

```rust
use core::fmt::Write;
use embedded_i18n::{date_time_locale, DateTimeFormatter};

let formatter = DateTimeFormatter::new(date_time_locale("fr"));

let mut output = String::new();
write!(output, "Mis à jour le {}", formatter.display(1_791_884_800, "%d %B"))?;
assert_eq!(output, "Mis à jour le 13 octobre");
# Ok::<(), core::fmt::Error>(())
```

With the `alloc` feature, [`DateTimeFormatter::format`] returns a `String` instead.

## Write Month Names Alone

Some languages decline month names: Russian writes `13 октября` in a date but `октябрь` alone.
Use `%B` in dates and `%OB` for a month on its own, such as a calendar header:

```rust
use embedded_i18n::{date_time_locale, DateTimeFormatter};

let russian = DateTimeFormatter::new(date_time_locale("ru"));
let mut output = String::new();
russian.write(&mut output, 1_791_884_800, "%d %B / %OB")?;
assert_eq!(output, "13 октября / октябрь");
# Ok::<(), core::fmt::Error>(())
```

See [Date and Time][reference-date-time] for every token and the supported languages.

[how-to-switch-language]: switch_language_at_runtime.md
[reference-date-time]: ../reference/date_time.md
[`DateTimeFormatter`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/struct.DateTimeFormatter.html
[`DateTimeFormatter::new`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/struct.DateTimeFormatter.html#method.new
[`DateTimeFormatter::display`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/struct.DateTimeFormatter.html#method.display
[`DateTimeFormatter::format`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/struct.DateTimeFormatter.html#method.format
[`locale!`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/macro.locale.html
[`date_time_locale`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.date_time_locale.html
[`core::fmt::Display`]: https://doc.rust-lang.org/core/fmt/trait.Display.html
