# How to Migrate From 0.1

Version 0.2 replaces the `translate*!` macros and the `EMBEDDED_I18N_*` environment variables with a rust-i18n style API (see [How to Migrate From rust-i18n][how-to-migrate-rust-i18n]): one [`i18n!`] call to configure the crate and one [`t!`] macro to translate.
The locale file formats, the plural rules and the compile-time guarantees are unchanged.
No deprecated aliases of the 0.1 API are kept.
This guide upgrades a crate, item by item.

## Upgrade Checklist

1. Update the dependency to `embedded-i18n = "0.2"`.
2. Add `embedded_i18n::i18n!("locales");` at the crate root, before the `mod` declarations, and remove `track_locales!()`.
3. Replace every `translate*!` macro, using the table below.
4. Rename positional placeholders (`{}`) in the locale files to named ones (`{value}`).
5. Move the `EMBEDDED_I18N_*` settings to the [`i18n!`] arguments or to `[package.metadata.i18n]`.
6. Replace the runtime locale API, the build script getters and the date and time defaults.

## Macros

| 0.1 | 0.2 |
| --- | --- |
| `translate!("key")` | [`t!`]`("key")` |
| `translate!(c"key")` | `t!(c"key")` |
| `translate!(context = "menu", "Open")` | `t!("Open", context = "menu")`, the old order is accepted too |
| `translate_args!("key", name = value)` | `t!("key", name = value)`, a `Display` value |
| `translate_write!(writer, "key", name = value)` | `write!(writer, "{}", t!("key", name = value))` |
| `translate_write!(writer, "key", value)` | `write!(writer, "{}", t!("key", value = value))`, with a named placeholder |
| `translate_plural!("key", n)` | `t!("key", count = n)`, a `Display` value with `{count}` filled in (`"3 files"`), where 0.1 returned the raw form as a `&'static str` (`"{count} files"`) |
| `translate_plural_write!(writer, "key", n)` | `write!(writer, "{}", t!("key", count = n))` |
| `translate_select!("key", value)` | `t!("key", select = value)` |
| `translate_select_write!(writer, "key", value)` | `write!(writer, "{}", t!("key", select = value))` |
| `track_locales!()` | removed: [`i18n!`] tracks the locale files, `Cargo.toml` and the override variables itself |

Differences to know about:

- Placeholders must be named and arguments are never captured implicitly: `{name}` needs `name = name`.
- Arguments are evaluated once and borrowed, so the result of [`t!`] with arguments lives as long as its arguments.
- A placeholder without argument is a compile error for every locale, plural form and select variant, while an argument that a translation does not use is allowed.
- The `count` option of plural forms ([Plural Rules][reference-plural-rules]) is also available as the `{count}` placeholder.
- `translate!` could be used as a format string (`format!(translate!("key"), ..)`): use `format!("{}", t!("key", ..))`.

Before and after:

```rust
use core::fmt::Write;
use embedded_i18n::t;

// At the crate root, in place of `track_locales!()`.
embedded_i18n::i18n!("locales");

fn render(output: &mut impl Write, name: &str, count: usize) -> core::fmt::Result {
    // 0.1: translate_write!(output, "with_named_args")
    write!(output, "{}", t!("with_named_args", name = name, count = count))?;
    // 0.1: translate_plural_write!(output, "files", count)
    write!(output, " / {}", t!("files", count = count))
}

let mut output = String::new();
render(&mut output, "Ada", 3)?;
assert_eq!(output, "Hello Ada, you have 3 messages / 3 files");
# Ok::<(), core::fmt::Error>(())
```

## Environment Variables

Only `EMBEDDED_I18N_LOCALE` and `EMBEDDED_I18N_LOCALES` remain, as overrides for CI matrices building one firmware per language.

| 0.1 | 0.2 |
| --- | --- |
| `EMBEDDED_I18N_LOCALE` | `default_locale` (`default-locale`), still available as an override |
| `EMBEDDED_I18N_FALLBACK` | `fallback` |
| `EMBEDDED_I18N_PATH` | the first argument of [`i18n!`], or `load-path` |
| `EMBEDDED_I18N_LOCALES` | `available_locales` (`available-locales`), still available as an override |
| `EMBEDDED_I18N_STRICT` | `strict` |
| `EMBEDDED_I18N_BIDI_ISOLATION` | `bidi_isolation` (`bidi-isolation`) |

For instance, a `.cargo/config.toml` with

```toml
[env]
EMBEDDED_I18N_LOCALE = "fr"
EMBEDDED_I18N_FALLBACK = "en"
EMBEDDED_I18N_STRICT = "1"
```

becomes

```toml
[package.metadata.i18n]
default-locale = "fr"
fallback = ["en"]
strict = true
```

and `EMBEDDED_I18N_LOCALE=de cargo build` still overrides the default locale.
See [Configuration][reference-configuration].

## Runtime Locales

Runtime selection is enabled by `available_locales` with more than one locale, as before with `EMBEDDED_I18N_LOCALES`.
The global locale is now a compact identifier of the requested tag instead of an index into the locales of one crate, see [The Runtime Locale Model][explanation-runtime-locale-model] and [How to Switch Language at Runtime][how-to-switch-language].

| 0.1 | 0.2 |
| --- | --- |
| `set_locale("fr")?` | [`set_locale`]`("fr")`, which returns `()`: each crate uses its closest embedded locale |
| `set_locale_index(i)`, `locale_index()` | removed: use [`set_locale`] with a name |
| `locale()` | [`locale!`]`()`, the locale the calling crate translates into |
| `LOCALES` | [`available_locales!`]`()` |
| `LOCALE_COUNT` | `available_locales!().len()` |
| `find_locale(name)` | removed: iterate [`available_locales!`]`()` |
| `DEFAULT_LOCALE_INDEX` | removed: the startup locale is the `default_locale` |
| `UnknownLocale` | removed: [`set_locale`] never fails |
| (none) | [`Locale`], a `Copy` handle of a tag, and `t!("key", locale = ...)` to translate into a given locale |
| (none) | [`system_locale`] with the `std` feature, to read the locale of the operating system |

The `locale = ...` option replaces the explicit locale per call that PR #62 proposed.

## Build Script Getters

`get_locale`, `get_fallback_locale`, `get_locale_build` and `get_fallback_locale_build` are removed with the global build locale.
Use [`locale!`]`()` in the firmware, and read `EMBEDDED_I18N_LOCALE` yourself in a build script, see [How to Subset Fonts][how-to-subset-fonts].

## Date, Time and Text Direction

| 0.1 | 0.2 |
| --- | --- |
| `DATE_TIME_LOCALE` | removed |
| `DateTimeFormatter::new()` | `DateTimeFormatter::new(date_time_locale(locale!()))`, still resolved at compile time with a single locale |
| `DateTimeFormatter::default()` | removed with `impl Default for DateTimeFormatter`: use `DateTimeFormatter::new(locale)` |
| `write_unix_timestamp(..)` | `DateTimeFormatter::new(date_time_locale(locale!())).write(..)` |
| `format_unix_timestamp(..)` | `DateTimeFormatter::new(date_time_locale(locale!())).format(..)` |
| `is_locale_right_to_left()` | `is_right_to_left(locale!())` |

`write_unix_timestamp` and `format_unix_timestamp` remain, but no longer follow the build locale: 0.1 formatted in the build locale and UTC, 0.2 formats in English and UTC.
Replace them with a [`DateTimeFormatter`] built from [`locale!`]`()` to keep translated month and weekday names:

```rust
use embedded_i18n::{date_time_locale, DateTimeFormatter};

embedded_i18n::i18n!("locales");

let mut output = String::new();
// 0.1: write_unix_timestamp(&mut output, 1_791_884_800, "%d %B")
DateTimeFormatter::new(date_time_locale(embedded_i18n::locale!()))
    .write(&mut output, 1_791_884_800, "%d %B")?;
assert_eq!(output, "13 October");
# Ok::<(), core::fmt::Error>(())
```

See [How to Format Dates and Times][how-to-format-dates] and [How to Support Right-to-Left Languages][how-to-right-to-left].

## Cargo Features

The `backend-*` and `alloc` features are unchanged.
The `std` feature enables [`system_locale`] instead of the build script getters.

[reference-configuration]: ../reference/configuration.md
[reference-plural-rules]: ../reference/plural_rules.md
[how-to-migrate-rust-i18n]: migrate_from_rust_i18n.md
[how-to-switch-language]: switch_language_at_runtime.md
[how-to-subset-fonts]: subset_fonts.md
[how-to-format-dates]: format_dates_and_times.md
[how-to-right-to-left]: support_right_to_left_languages.md
[explanation-runtime-locale-model]: ../explanation/runtime_locale_model.md
[`i18n!`]: ../reference/configuration.md
[`t!`]: ../reference/t_macro.md
[`set_locale`]: ../reference/runtime_api.md
[`locale!`]: ../reference/runtime_api.md
[`available_locales!`]: ../reference/runtime_api.md
[`Locale`]: ../reference/runtime_api.md
[`system_locale`]: ../reference/runtime_api.md
[`DateTimeFormatter`]: ../reference/date_time.md
