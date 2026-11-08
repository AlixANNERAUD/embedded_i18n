# Runtime Locale Selection

By default a firmware embeds one locale, and every macro expands to a literal.
When the user picks the language at runtime (a settings menu, a DIP switch, a configuration stored in flash), list the locales to embed:

```sh
EMBEDDED_I18N_LOCALES=en,fr,de EMBEDDED_I18N_LOCALE=en cargo build --release
```

Every macro then selects the translation of the current locale, and [`set_locale`] changes it:

```rust
use embedded_i18n::{locale, set_locale, translate, LOCALES};

for &language in LOCALES.iter() {
    set_locale(language)?;
    assert_eq!(locale(), language);
    let greeting: &'static str = translate!("greeting");
    // en: "Hello", fr: "Bonjour", de: "Hallo"
    assert!(!greeting.is_empty());
}
# Ok::<(), embedded_i18n::UnknownLocale>(())
```

## Translating Into a Given Locale

The current locale is shared by the whole program.
When several languages are needed at once, such as a server answering each request in its user's language, give the locale to each macro instead: the `locale` option takes an index in [`LOCALES`] and leaves the current locale untouched.

```rust
use core::fmt::Write;
use embedded_i18n::{match_locale, translate, translate_write, DEFAULT_LOCALE_INDEX};

// Once per request, from a user setting or an `Accept-Language` tag.
let locale = match_locale("fr-CA").unwrap_or(DEFAULT_LOCALE_INDEX);

let greeting: &'static str = translate!(locale = locale, "greeting");
let mut page = String::new();
translate_write!(page, locale = locale, "with_named_args", name = "Ada", count = 3)?;
# Ok::<(), core::fmt::Error>(())
```

Like `context`, the option comes before the message key, in any order with `context`; arguments after the key are format arguments.
[`match_locale`] finds the embedded locale serving a requested one: the locale itself, then its parents (`fr-CA` for `fr-CA-x`, then `fr`), then any embedded locale of the same language (`fr-FR` for `fr`).
An index outside [`LOCALES`] selects [`DEFAULT_LOCALE_INDEX`].
In single-locale builds the option is accepted, evaluated and ignored, so the same code builds in both modes.

## API

| Item | Description |
| --- | --- |
| [`LOCALES`] | Embedded locales, in the order of `EMBEDDED_I18N_LOCALES` (the build locale alone without it) |
| [`LOCALE_COUNT`] | Number of embedded locales |
| [`set_locale`] / [`set_locale_index`] | Select the current locale by name or by index, or return [`UnknownLocale`] |
| [`locale`] / [`locale_index`] | Read the current locale |
| [`find_locale`] | Index of a locale in [`LOCALES`], in `const` context |
| [`match_locale`] | Index of the embedded locale best serving a requested one, for the `locale` option |
| [`DEFAULT_LOCALE_INDEX`] | Locale active at startup: `EMBEDDED_I18N_LOCALE`, or the first embedded locale if it is not listed |

Names are compared case-insensitively, with `-` and `_` treated alike.
Without `EMBEDDED_I18N_LOCALES`, only the build locale is accepted, so code using these functions builds in both modes.

The current locale is a global `AtomicU8` used with plain loads and stores.
It works on targets without compare-and-swap, such as `thumbv6m-none-eabi`, and needs no allocation, lock or critical section.
Up to 255 locales can be embedded.

## How It Expands

- Each embedded locale is resolved at compile time with its own fallback chain (`fr`, then `EMBEDDED_I18N_FALLBACK`), and every message must be found for every embedded locale, otherwise the build fails and names the locales missing it.
- [`translate!`] expands to an index into a static array with one entry per locale.
- The plural, select and write macros expand to a `match` on the locale index, each arm being the single-locale expansion of that locale.
- [`translate_args!`] formats through a closure, so formatting flags such as width or alignment are not applied to the whole message.

With a single embedded locale (`EMBEDDED_I18N_LOCALES=fr`), every macro still expands to a literal.

## Limitations

[`translate!`] returns a runtime value, so it cannot initialize a `const` or `static`, nor be the format string of `format!` or `write!`:

```text
error[E0015]: cannot call non-const function `locale_index` in constants
```

Use [`translate_write!`] and [`translate_args!`] for formatting, which support both modes.

[`DATE_TIME_LOCALE`] keeps following `EMBEDDED_I18N_LOCALE`.
To format dates in the current locale, use `DateTimeFormatter::new().with_locale(date_time_locale(locale()))`, which links the date and time names of every supported language (see [Date and Time][guide-date-time]).

## ROM Cost

- the strings of every embedded locale, for each message used (identical strings are usually merged by the linker);
- per [`translate!`] call site, a table of one string reference per locale (two words each) and a few instructions to index it;
- per call site of the other macros, one comparison per embedded locale.

[guide-date-time]: date_time.md
[`translate!`]: translating.md
[`translate_args!`]: formatting.md
[`translate_write!`]: formatting.md
[`LOCALES`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/constant.LOCALES.html
[`LOCALE_COUNT`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/constant.LOCALE_COUNT.html
[`set_locale`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.set_locale.html
[`set_locale_index`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.set_locale_index.html
[`locale`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.locale.html
[`locale_index`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.locale_index.html
[`find_locale`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.find_locale.html
[`match_locale`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.match_locale.html
[`DEFAULT_LOCALE_INDEX`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/constant.DEFAULT_LOCALE_INDEX.html
[`UnknownLocale`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/struct.UnknownLocale.html
[`DATE_TIME_LOCALE`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/constant.DATE_TIME_LOCALE.html
