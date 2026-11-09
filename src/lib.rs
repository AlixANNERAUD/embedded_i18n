//! Compile-time internationalization for embedded and `no_std` Rust, with a
//! [rust-i18n](https://crates.io/crates/rust-i18n) style API.
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
//!   "greeting": "Hello",
//!   "files": { "one": "{count} file", "other": "{count} files" }
//! }
//! ```
//!
//! Call [`i18n!`] once at the crate root, before any `mod` declaration, then
//! translate with [`t!`]:
//!
//! ```
//! embedded_i18n::i18n!("locales");
//! use core::fmt::Write;
//! use embedded_i18n::t;
//!
//! const GREETING: &str = t!("greeting");
//! assert_eq!(GREETING, "Hello");
//!
//! let mut output = String::new();
//! write!(output, "{}", t!("with_arg", value = 42))?;
//! assert_eq!(output, "Value: 42");
//!
//! assert_eq!(t!("files", count = 2).to_string(), "2 files");
//! # Ok::<(), core::fmt::Error>(())
//! ```
//!
//! # Macros
//!
//! | Macro | Result |
//! | --- | --- |
//! | [`i18n!`] | Configures the translations of the crate, once at its root |
//! | [`t!`] | `&'static str` (or `&'static CStr`) without arguments, a [`core::fmt::Display`] value with arguments or `count` |
//! | [`locale!`] | Name of the locale the crate translates into |
//! | [`available_locales!`] | Locales embedded in the crate |
//!
//! # Configuration
//!
//! [`i18n!`] takes the locale directory and options, read from
//! `[package.metadata.i18n]` in `Cargo.toml` when not given:
//!
//! | `i18n!` argument | `[package.metadata.i18n]` key | Default | Meaning |
//! | --- | --- | --- | --- |
//! | first positional string | `load-path` | `"locales"` | Locale directory, relative to `Cargo.toml` |
//! | `fallback = "en"` or `["en", "es"]` | `fallback` | `["en"]` | Fallback chain after the locale and its parents |
//! | `default_locale = "fr"` | `default-locale` | `"en"` | Locale active at startup |
//! | `available_locales = [..]` | `available-locales` | `[default_locale]` | Embedded locales; more than one enables runtime selection |
//! | `strict = true` | `strict` | `false` | Forbids falling back for messages missing from the primary locale |
//! | `bidi_isolation = true` | `bidi-isolation` | `false` | Isolates placeholders of right-to-left locales |
//!
//! The `EMBEDDED_I18N_LOCALE` and `EMBEDDED_I18N_LOCALES` (comma separated)
//! environment variables override `default_locale` and `available_locales`, to
//! build one firmware per language. Cargo rebuilds the crate when a locale
//! file, `Cargo.toml` or one of these variables changes.
//!
//! Locales are normalized like BCP 47 tags and resolved from the most specific
//! to the most generic (`fr-CA`, `fr`, then the fallbacks). The special locale
//! `pseudo` generates a pseudo-localization of the fallback locales.
//!
//! # Runtime locales
//!
//! With several `available_locales`, [`set_locale`] selects the locale at
//! runtime, for every crate of the program: each one translates into its
//! closest embedded locale, or its `default_locale`. `t!("key", locale =
//! "fr")` translates into a given locale without touching the global one.
//!
//! ```rust,standalone_crate
//! embedded_i18n::i18n!("locales", available_locales = ["en", "fr"]);
//! use embedded_i18n::{locale, set_locale, t};
//!
//! fn main() {
//!     assert_eq!(t!("greeting", locale = "fr-CA"), "Bonjour");
//!     set_locale("fr");
//!     assert_eq!(t!("greeting"), "Bonjour");
//!     assert_eq!(locale!(), "fr");
//!     set_locale("de");
//!     assert_eq!(t!("greeting"), "Hello");
//! }
//! ```
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
//! - [`Locale`] and [`set_locale`]: runtime locale selection when several
//!   locales are embedded.
//!
//! # Features
//!
//! - `alloc`: `String` and `Vec` returning variants of the helpers.
//! - `std`: enables `alloc` and `system_locale()`, the locale of the operating
//!   system.
//!
//! # Guide
//!
//! The [`guide`] follows the [Diátaxis](https://diataxis.fr) structure:
//!
//! - **Tutorial**: [getting started](guide::tutorials::getting_started), from
//!   an empty project to a program translated into two languages.
//! - **How-to guides**:
//!   [switch language at runtime](guide::how_to::switch_language_at_runtime),
//!   [localize per request](guide::how_to::localize_per_request),
//!   [build one firmware per language](guide::how_to::build_one_firmware_per_language),
//!   [use gettext PO files and other formats](guide::how_to::use_gettext_po_files),
//!   [ship complete translations](guide::how_to::ship_complete_translations),
//!   [pseudo-localize](guide::how_to::pseudo_localize),
//!   [support right-to-left languages](guide::how_to::support_right_to_left_languages),
//!   [format dates and times](guide::how_to::format_dates_and_times),
//!   [subset fonts](guide::how_to::subset_fonts),
//!   [migrate from rust-i18n](guide::how_to::migrate_from_rust_i18n) and
//!   [migrate from 0.1](guide::how_to::migrate_from_0_1).
//! - **Reference**: [the `t!` macro](guide::reference::t_macro),
//!   [configuration](guide::reference::configuration),
//!   [locale files](guide::reference::locale_files),
//!   [plural rules](guide::reference::plural_rules),
//!   [runtime API](guide::reference::runtime_api),
//!   [date and time](guide::reference::date_time),
//!   [Unicode ranges](guide::reference::unicode_ranges) and
//!   [features and `no_std`](guide::reference::features_and_no_std).
//! - **Explanation**: [how it works](guide::explanation::how_it_works),
//!   [the runtime locale model](guide::explanation::runtime_locale_model),
//!   [design choices](guide::explanation::design_choices) and a
//!   [comparison with other crates](guide::explanation::comparison).
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
mod time;

pub use date_time::*;
pub use direction::*;
pub use locale::*;
pub use range::*;
pub use time::*;

/// Configures the translations of the calling crate.
///
/// Invoke it once, at the crate root (`lib.rs` or `main.rs`), before the `mod`
/// declarations that use [`t!`], as with rust-i18n. Each crate, integration
/// test, example and doctest that translates calls it once; a second call in
/// the same crate is a compile error. The arguments and the
/// `[package.metadata.i18n]` keys are listed in the
/// [crate documentation](crate#configuration).
///
/// ```
/// // Everything from `[package.metadata.i18n]`, or the defaults:
/// embedded_i18n::i18n!();
/// ```
///
/// ```
/// embedded_i18n::i18n!("locales", fallback = ["en"], strict = false);
/// ```
///
/// With more than one available locale, `i18n!` also generates the crate's
/// function resolving the [current locale](set_locale) to one of its locales:
/// the same tag, else the same language, else `default_locale`.
///
/// Errors are reported on the call: unknown option, wrong value type, empty
/// locale name, `default_locale` not in `available_locales`, more than 255
/// available locales, two available locales with the same [`Locale`]
/// identifier, missing locale directory.
///
/// ```compile_fail
/// embedded_i18n::i18n!(available_locales = ["fr-CA", "fr_ca"], default_locale = "fr-CA");
/// ```
pub use embedded_i18n_macros::i18n;

#[doc(hidden)]
pub use embedded_i18n_macros::__expand;

/// Translates a message key, at compile time.
///
/// `t!(KEY, items...)` where `KEY` is a string literal, or a C string literal
/// to get a `&'static CStr` for FFI, and each item is one of:
///
/// - `context = "menu"`: a message context, to disambiguate identical source
///   strings (gettext `msgctxt`, or the `menu.Open` key);
/// - `locale = <expr>`: translates into this locale instead of the current one;
/// - `count = <integer>`: selects the plural form, and is the `{count}`
///   placeholder;
/// - `select = <expr>`: selects a variant (`replied.male`, `replied.other`) by
///   `AsRef<str>`, `other` being the default. When both `select` and `count`
///   are given, `select` picks the variant and `count` is an ordinary
///   argument (the `{count}` placeholder): it selects no plural form;
/// - `name = <expr>` or `"name" => <expr>`: an argument, the value of the
///   `{name}` (or rust-i18n `%{name}`) placeholder, optionally followed by a
///   format specification: `sn = 123 : {:08}`.
///
/// Without arguments nor `count`, the result is a `&'static str`: in
/// single-locale mode, a literal usable in `const` items.
///
/// ```
/// use core::ffi::CStr;
/// embedded_i18n::i18n!();
/// use embedded_i18n::t;
///
/// const GREETING: &str = t!("greeting");
/// assert_eq!(GREETING, "Hello");
///
/// let greeting: &'static CStr = t!(c"greeting");
/// assert_eq!(greeting.to_bytes(), b"Hello");
///
/// assert_eq!(t!("Open", context = "menu"), "Open (menu)");
/// assert_eq!(t!("replied", select = "female"), "She replied");
/// ```
///
/// With arguments or `count`, the result implements [`core::fmt::Display`]
/// and formats on demand, without allocation. Arguments are evaluated once and
/// borrowed, as by `format_args!`.
///
/// ```
/// embedded_i18n::i18n!();
/// use embedded_i18n::t;
///
/// let name = "Ada";
/// assert_eq!(
///     t!("with_named_args", name = name, count = 3).to_string(),
///     "Hello Ada, you have 3 messages"
/// );
/// assert_eq!(t!("files", count = 1).to_string(), "1 file");
/// assert_eq!(t!("with_arg", "value" => 7 : {:03}).to_string(), "Value: 007");
/// ```
///
/// A placeholder without an argument, a key missing from every locale of the
/// chain, arguments given to a C string key, or a crate without [`i18n!`] are
/// compile errors:
///
/// ```compile_fail
/// embedded_i18n::i18n!();
/// // The translation uses {count}.
/// let _ = embedded_i18n::t!("with_named_args", name = "Ada");
/// ```
///
/// ```compile_fail
/// embedded_i18n::i18n!();
/// let _ = embedded_i18n::t!("no such key");
/// ```
///
/// ```compile_fail
/// embedded_i18n::i18n!();
/// let _ = embedded_i18n::t!(c"greeting", name = 1);
/// ```
///
/// ```compile_fail
/// // No `embedded_i18n::i18n!()` in this crate.
/// let _ = embedded_i18n::t!("greeting");
/// ```
#[macro_export]
macro_rules! t {
    ($($tokens:tt)*) => {
        embedded_i18n_requires_i18n_at_the_crate_root!(@t $($tokens)*)
    };
}

/// Returns the name of the locale the calling crate translates into, as a
/// `&'static str`.
///
/// In single-locale mode it is the `default_locale` literal; with several
/// available locales, the one the [current locale](set_locale) resolves to.
///
/// ```
/// embedded_i18n::i18n!();
///
/// assert_eq!(embedded_i18n::locale!(), "en");
/// assert!(!embedded_i18n::is_right_to_left(embedded_i18n::locale!()));
/// ```
#[macro_export]
macro_rules! locale {
    () => {
        embedded_i18n_requires_i18n_at_the_crate_root!(@locale)
    };
}

/// Lists the locales embedded in the calling crate, as
/// `&'static [&'static str]`, in configuration order.
///
/// ```
/// embedded_i18n::i18n!();
///
/// assert_eq!(embedded_i18n::available_locales!(), ["en"]);
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
