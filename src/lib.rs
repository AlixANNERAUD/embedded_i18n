//! Compile-time internationalization for embedded and `no_std` Rust.
//!
//! Translations are read from locale files at build time by procedural macros
//! and embedded in the binary as `&'static str` literals: no runtime parsing, no
//! allocator, no file system, and a missing translation is a compile error at
//! the call site.
//!
//! # Quick start
//!
//! Put locale files in a `locales/` directory next to `Cargo.toml`:
//!
//! ```json
//! {
//!   "greeting": "Bonjour",
//!   "files": { "one": "{count} fichier", "other": "{count} fichiers" }
//! }
//! ```
//!
//! Pick the locale at build time with `EMBEDDED_I18N_LOCALE=fr cargo build`, and
//! translate with the macros:
//!
//! ```
//! use core::fmt::Write;
//! use embedded_i18n::{translate, translate_plural, translate_write};
//!
//! embedded_i18n::track_locales!();
//!
//! const GREETING: &str = translate!("greeting");
//! assert_eq!(GREETING, "Hello");
//!
//! assert_eq!(translate_plural!("files", 2), "{count} files");
//!
//! let mut output = String::new();
//! translate_write!(&mut output, "with_arg", 42)?;
//! assert_eq!(output, "Value: 42");
//! # Ok::<(), core::fmt::Error>(())
//! ```
//!
//! # Macros
//!
//! | Macro | Result |
//! | --- | --- |
//! | [`translate!`] | `&'static str` or `&'static CStr` literal |
//! | [`translate_args!`] | [`core::fmt::Arguments`] with the translation as format string |
//! | [`translate_write!`] | Writes the formatted translation into a [`core::fmt::Write`] |
//! | [`translate_plural!`] / [`translate_plural_write!`] | Plural form selected from a count |
//! | [`translate_select!`] / [`translate_select_write!`] | Variant selected from an `AsRef<str>` value |
//! | [`track_locales!`] | Rebuilds the crate when a locale file changes |
//!
//! Every macro accepts a message context before the key: `translate!(context =
//! "menu", "Open")`.
//!
//! # Configuration
//!
//! The macros are configured with environment variables read at compile time,
//! which can be set in `.cargo/config.toml`:
//!
//! | Variable | Default | Description |
//! | --- | --- | --- |
//! | `EMBEDDED_I18N_LOCALE` | `en` | Locale to embed |
//! | `EMBEDDED_I18N_FALLBACK` | `en` | Comma separated fallback locales |
//! | `EMBEDDED_I18N_PATH` | `locales` | Locale files directory, relative to the calling crate |
//! | `EMBEDDED_I18N_LOCALES` | unset | Locales to embed for [runtime selection](set_locale) |
//! | `EMBEDDED_I18N_STRICT` | unset | Forbids falling back for messages missing from the primary locale |
//! | `EMBEDDED_I18N_BIDI_ISOLATION` | unset | Isolates placeholders of right-to-left locales |
//!
//! Locales are normalized like BCP 47 tags and resolved from the most specific
//! to the most generic (`fr-CA`, `fr`, then the fallbacks). The special locale
//! `pseudo` generates a pseudo-localization of the fallback locales.
//!
//! # Locale file formats
//!
//! Each format is enabled by a Cargo feature; all parsing happens in the
//! procedural macro, so formats add nothing to the binary.
//!
//! | Feature | Files |
//! | --- | --- |
//! | `backend-json` (default) | `.json`, nested objects flattened to dotted keys |
//! | `backend-yaml` | `.yaml`, `.yml` |
//! | `backend-toml` | `.toml` |
//! | `backend-po` | GNU gettext `.po`, with `msgctxt`, plural forms and `Plural-Forms` |
//! | `backend-mo` | Compiled GNU gettext `.mo` |
//! | `backend-fluent` | Fluent `.ftl` simple messages and whole-message selects |
//!
//! # Runtime helpers
//!
//! - [`DateTimeFormatter`] and [`write_unix_timestamp`]: localized `strftime`-style
//!   formatting without allocation.
//! - [`is_right_to_left`]: text direction.
//! - [`get_locale_ranges`] and [`write_ranges`]: Unicode ranges to subset fonts.
//! - [`set_locale`], [`locale`] and [`LOCALES`]: runtime locale selection when
//!   several locales are embedded.
//!
//! # Features
//!
//! - `alloc`: `String` and `Vec` returning variants of the helpers.
//! - `std`: enables `alloc` and [`system_locale`].
//!
//! # Guide
//!
//! The [`guide`] has one page per topic: [locale files](guide::locale_files),
//! [translating](guide::translating), [message context](guide::context),
//! [plural forms](guide::plurals), [select variants](guide::select),
//! [formatting](guide::formatting), [rebuild tracking](guide::rebuild_tracking),
//! [configuration](guide::configuration),
//! [runtime locale selection](guide::runtime_locales),
//! [translation coverage](guide::coverage),
//! [right-to-left languages](guide::right_to_left),
//! [pseudo-localization](guide::pseudo_localization),
//! [date and time](guide::date_time), [Unicode ranges](guide::unicode_ranges),
//! [`no_std` support](guide::no_std) and a
//! [comparison with other crates](guide::comparison).
#![no_std]
#![deny(missing_docs)]

extern crate self as embedded_i18n;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod date_time;
mod direction;
pub mod guide;
mod locale;
mod range;
mod selection;
mod time;

pub use date_time::*;
pub use direction::*;
pub use locale::*;
pub use range::*;
pub use selection::*;
pub use time::*;

/// Makes the calling crate depend on every locale file read by the macros.
///
/// Cargo does not know that the macros read locale files, so editing them would
/// not rebuild the crate. Invoke this macro once, at module level, to rebuild
/// whenever one of the loaded files changes. It expands to `include_bytes!` of
/// each file in an unused constant and adds nothing to the binary.
///
/// ```
/// embedded_i18n::track_locales!();
/// ```
pub use embedded_i18n_macros::track_locales;

/// Replaces a message key with its translation, at compile time.
///
/// The key is a string literal, or a C string literal to get a `&'static CStr`
/// for FFI. An optional `context` disambiguates identical source strings.
///
/// ```
/// use core::ffi::CStr;
/// use embedded_i18n::translate;
///
/// const GREETING: &str = translate!("greeting");
/// assert_eq!(GREETING, "Hello");
///
/// let greeting: &'static CStr = translate!(c"greeting");
/// assert_eq!(greeting.to_bytes(), b"Hello");
///
/// assert_eq!(translate!(context = "menu", "Open"), "Open (menu)");
/// ```
///
/// In single-locale builds the expansion is a literal, usable in `const` items
/// and as the format string of `format!` or `write!`. A key missing from every
/// locale of the chain is a compile error:
///
/// ```compile_fail
/// let _ = embedded_i18n::translate!("no such key");
/// ```
pub use embedded_i18n_macros::translate;

/// Builds [`core::fmt::Arguments`] from a translated format string.
///
/// The translation is the format string, so placeholders are checked at compile
/// time and can capture variables in scope. Usable wherever `format_args!` is,
/// without allocation.
///
/// ```
/// use embedded_i18n::translate_args;
///
/// let name = "Ada";
/// let text = format!("{}", translate_args!("with_named_args", count = 3));
/// assert_eq!(text, "Hello Ada, you have 3 messages");
/// ```
pub use embedded_i18n_macros::translate_args;

/// Writes a translated, formatted message into a [`core::fmt::Write`].
///
/// Expands to `write!(writer, translation, arguments...)` and returns
/// [`core::fmt::Result`].
///
/// ```
/// use core::fmt::Write;
/// use embedded_i18n::translate_write;
///
/// let mut output = String::new();
/// translate_write!(&mut output, "with_arg", 42)?;
/// assert_eq!(output, "Value: 42");
/// # Ok::<(), core::fmt::Error>(())
/// ```
pub use embedded_i18n_macros::translate_write;

/// Selects the plural form of a message matching a count.
///
/// Forms are CLDR categories (`files.one`, `files.other`...) in JSON, YAML, TOML
/// and Fluent files, or `msgstr[N]` with the `Plural-Forms` rule in gettext
/// files. The rule is compiled into an `if` / `match` on the count; the result
/// is a `&'static str`, or a `&'static CStr` with a C string key.
///
/// ```
/// use embedded_i18n::translate_plural;
///
/// assert_eq!(translate_plural!("files", 1), "{count} file");
/// assert_eq!(translate_plural!("files", 5u32), "{count} files");
/// ```
pub use embedded_i18n_macros::translate_plural;

/// Writes the plural form of a message matching a count into a
/// [`core::fmt::Write`].
///
/// The count is bound to `count`, so forms can use the `{count}` placeholder.
/// Extra format arguments can follow the count.
///
/// ```
/// use core::fmt::Write;
/// use embedded_i18n::translate_plural_write;
///
/// let mut output = String::new();
/// translate_plural_write!(&mut output, "files", 3)?;
/// assert_eq!(output, "3 files");
/// # Ok::<(), core::fmt::Error>(())
/// ```
pub use embedded_i18n_macros::translate_plural_write;

/// Selects a variant of a message from a value implementing `AsRef<str>`.
///
/// Variants are nested keys (`replied.male`, `replied.other`); the `other`
/// variant is required and used for every value without its own variant.
///
/// ```
/// use embedded_i18n::translate_select;
///
/// assert_eq!(translate_select!("replied", "female"), "She replied");
/// assert_eq!(translate_select!("replied", "unknown"), "They replied");
/// ```
pub use embedded_i18n_macros::translate_select;

/// Writes the variant of a message selected from a value implementing
/// `AsRef<str>` into a [`core::fmt::Write`].
///
/// ```
/// use core::fmt::Write;
/// use embedded_i18n::translate_select_write;
///
/// let mut output = String::new();
/// translate_select_write!(&mut output, "replied", "male")?;
/// assert_eq!(output, "He replied");
/// # Ok::<(), core::fmt::Error>(())
/// ```
pub use embedded_i18n_macros::translate_select_write;

/// Translates a message key into the current locale.
///
/// The macro is generated by `i18n!`, invoked once at the crate root; it
/// reads the current locale selected with [`set_locale`] at runtime.
///
/// ```ignore
/// embedded_i18n::i18n!();
///
/// let greeting = embedded_i18n::t!("greeting");
/// ```
#[macro_export]
macro_rules! t {
    ($($tokens:tt)*) => {
        embedded_i18n_requires_i18n_at_the_crate_root!(@t $($tokens)*)
    };
}

/// Returns the current [`Locale`] among the locales embedded by `i18n!`.
///
/// ```ignore
/// embedded_i18n::i18n!();
///
/// let current = embedded_i18n::locale!();
/// ```
#[macro_export]
macro_rules! locale {
    () => {
        embedded_i18n_requires_i18n_at_the_crate_root!(@locale)
    };
}

/// Lists the locales embedded by `i18n!`, as `&'static [Locale]`.
///
/// ```ignore
/// embedded_i18n::i18n!();
///
/// for locale in embedded_i18n::available_locales!() {
///     let _ = locale.id();
/// }
/// ```
#[macro_export]
macro_rules! available_locales {
    () => {
        embedded_i18n_requires_i18n_at_the_crate_root!(@available)
    };
}

/// Returns the locale of the operating system, from the `LC_ALL`,
/// `LC_MESSAGES` and `LANG` environment variables, as a BCP 47 style tag
/// (`fr_FR.UTF-8` becomes `fr-FR`). Available with the `std` feature.
///
/// The first variable that holds a usable value wins; the `C` and `POSIX`
/// locales are ignored. Returns `None` when no variable is set.
///
/// This reads the POSIX environment only. Windows and macOS GUI applications
/// should use the [`sys-locale`](https://crates.io/crates/sys-locale) crate
/// instead, and pass its result to [`set_locale`].
///
/// ```ignore
/// if let Some(locale) = embedded_i18n::system_locale() {
///     embedded_i18n::set_locale(&locale);
/// }
/// ```
#[cfg(feature = "std")]
pub fn system_locale() -> Option<std::string::String> {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .find_map(|name| normalize_system_locale(&std::env::var(name).ok()?))
}

/// Turns a POSIX locale value (`fr_FR.UTF-8@euro`) into a tag (`fr-FR`).
#[cfg(feature = "std")]
fn normalize_system_locale(value: &str) -> Option<std::string::String> {
    let end = value.find(['.', '@']).unwrap_or(value.len());
    let base = value[..end].trim();
    if base.is_empty() || base == "C" || base == "POSIX" {
        return None;
    }
    Some(base.replace('_', "-"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_locale_ranges_supported_locales() {
        for locale in &[
            "en", "fr", "de", "es", "it", "pt", "nl", "pl", "ru", "ja", "zh",
        ] {
            assert!(
                get_locale_ranges(locale).is_some(),
                "{} should have ranges",
                locale
            );
        }
    }

    #[test]
    fn get_locale_ranges_unsupported_locale() {
        assert!(get_locale_ranges("unsupported").is_none());
        assert!(get_locale_ranges("xx").is_none());
        assert!(get_locale_ranges("").is_none());
    }

    #[test]
    fn translate_macro_is_reexported() {
        let _ = translate!("greeting");
    }
}

#[cfg(all(test, feature = "std"))]
mod system_locale_tests {
    use super::*;

    #[test]
    fn normalizes_system_locales() {
        assert_eq!(
            normalize_system_locale("fr_FR.UTF-8@euro").as_deref(),
            Some("fr-FR")
        );
        assert_eq!(normalize_system_locale("de_DE").as_deref(), Some("de-DE"));
        assert_eq!(normalize_system_locale("C"), None);
        assert_eq!(normalize_system_locale("POSIX"), None);
        assert_eq!(normalize_system_locale(""), None);
        assert_eq!(normalize_system_locale("C.UTF-8"), None);
    }
}
