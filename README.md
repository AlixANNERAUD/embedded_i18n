# embedded-i18n

[![crates.io](https://img.shields.io/crates/v/embedded-i18n.svg)](https://crates.io/crates/embedded-i18n)
[![docs.rs](https://img.shields.io/docsrs/embedded-i18n)](https://docs.rs/embedded-i18n)
[![CI](https://github.com/AlixANNERAUD/embedded_i18n/actions/workflows/check.yml/badge.svg)](https://github.com/AlixANNERAUD/embedded_i18n/actions/workflows/check.yml)
[![codecov](https://codecov.io/github/AlixANNERAUD/embedded_i18n/graph/badge.svg?token=0C27GIS0BK)](https://codecov.io/github/AlixANNERAUD/embedded_i18n)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![MSRV 1.85](https://img.shields.io/badge/MSRV-1.85-orange.svg)](#minimum-supported-rust-version)
[![no_std](https://img.shields.io/badge/no__std-no%20alloc-success.svg)](docs/reference/features_and_no_std.md)

**Compile-time internationalization (i18n) and localization (l10n) for embedded Rust, `no_std` firmware and microcontrollers.**

`embedded-i18n` translates your strings **at build time**: translations from JSON, YAML, TOML, GNU gettext PO/MO or Fluent files are compiled into your binary as plain `&'static str` literals stored in flash/ROM. No runtime parser, no allocator, no file system, no heap, and a missing translation is a **compile error**, not a blank screen on a device in the field.

The API follows [rust-i18n](https://crates.io/crates/rust-i18n): `i18n!` configures the crate and `t!` translates, with the same names and most of the same syntax.

## Why embedded-i18n?

- **Zero runtime cost.** Each `t!("key")` expands to a string literal. Your firmware only contains the strings it uses, in the language you build for.
- **Fits on a microcontroller.** `#![no_std]`, no allocator, no `std::fs`, no `libc`. Builds for bare-metal targets such as `thumbv7em-none-eabihf` and `thumbv6m-none-eabi` (no atomic compare-and-swap).
- **Mistakes caught by the compiler.** Missing keys, missing plural forms, invalid locale files and format placeholders that do not match their arguments are reported as compile errors at the call site. An optional strict mode forbids falling back to another language.
- **Real-world linguistics.** CLDR plural rules for 40 languages, gettext `Plural-Forms`, message context (`msgctxt`), select variants (gender, enums), locale fallback chains (`fr-CA` → `fr` → `en`), right-to-left bidi isolation and localized dates without `libc`.
- **Use the files translators already know.** JSON (nested keys), YAML, TOML, GNU gettext `.po` / `.mo`, and Fluent `.ftl`, all parsed at compile time, so the formats add no code to the firmware.
- **Runtime language switching when you need it.** Embed several locales with `available_locales` and switch with `set_locale("de")`, still without allocation.

## Example

`locales/fr.json`, next to `Cargo.toml`:

```json
{
  "Settings": "Paramètres",
  "files": { "one": "{count} fichier", "other": "{count} fichiers" }
}
```

`src/main.rs` (or `lib.rs`):

```rust
use core::fmt::Write;
use embedded_i18n::t;

embedded_i18n::i18n!("locales", default_locale = "fr");

const TITLE: &str = t!("Settings"); // "Paramètres", a literal in flash

fn render(display: &mut impl Write, count: usize) -> core::fmt::Result {
    display.write_str(TITLE)?;
    write!(display, "{}", t!("files", count = count)) // "1 fichier", "3 fichiers", without allocation
}
```

Build the same code in another language with `EMBEDDED_I18N_LOCALE=de cargo build --release`.
The [Getting Started][tutorial-getting-started] tutorial walks through it, from an empty project to runtime language switching.

## Comparison

| | **embedded-i18n** | [rust-i18n](https://crates.io/crates/rust-i18n) | [Fluent](https://crates.io/crates/fluent) | [gettext-rs](https://crates.io/crates/gettext-rs) |
| --- | --- | --- | --- | --- |
| `no_std`, no allocator | ✅ | ❌ | ❌ | ❌ |
| Translations resolved | At compile time, as literals | Compile-time embedded, looked up at runtime | Parsed and resolved at runtime | Loaded from `.mo` files at runtime |
| Missing translation | Compile error | Runtime fallback | Runtime error / fallback | Runtime fallback to the source string |
| Format placeholders checked at compile time | ✅ (every locale) | ❌ | ❌ | ❌ |
| Plural forms | ✅ CLDR rules and gettext `Plural-Forms` | Not built in | ✅ | ✅ |
| Message context / select variants | ✅ / ✅ | ❌ / ❌ | Message ids / ✅ | ✅ / ❌ |
| Runtime locale switching | Optional (`available_locales`) | ✅ | ✅ | ✅ |
| File formats | JSON, YAML, TOML, PO, MO, Fluent (subset) | YAML, JSON, TOML | Fluent | MO (via GNU gettext) |
| Localized dates without `libc` | ✅ | ❌ | ❌ | ❌ |

Pick `embedded-i18n` when the binary must be small, deterministic and allocation-free, and the set of languages is known at build time. On desktop or server applications with downloadable language packs and full Fluent features (functions, parameterized terms), a runtime library is a better fit. See the [comparison][explanation-comparison] for details.

## Documentation

The documentation follows the [Diátaxis](https://diataxis.fr) structure.
It is also available on [docs.rs](https://docs.rs/embedded-i18n/latest/embedded_i18n/guide/index.html), with every example compiled and tested.

### Tutorials

- [Getting Started][tutorial-getting-started]: from an empty project to a program translated into two languages, without allocator.

### How-to Guides

- [Switch language at runtime][how-to-switch-language]
- [Localize per request][how-to-localize-per-request], with an explicit locale per call
- [Build one firmware per language][how-to-build-per-language], with environment overrides and a CI matrix
- [Use gettext PO files and other formats][how-to-use-po-files]
- [Ship complete translations][how-to-ship-complete], with strict mode
- [Pseudo-localize][how-to-pseudo-localize]
- [Support right-to-left languages][how-to-right-to-left]
- [Format dates and times][how-to-format-dates]
- [Subset fonts][how-to-subset-fonts]
- [Migrate from rust-i18n][how-to-migrate-rust-i18n]
- [Migrate from 0.1][how-to-migrate-0-1]

### Reference

- [The `t!` macro][reference-t]: grammar, options, result types, placeholders, plural, select, context and compile errors
- [Configuration][reference-configuration]: `i18n!`, `[package.metadata.i18n]`, environment overrides, locale resolution and rebuild tracking
- [Locale files][reference-locale-files]: JSON, YAML, TOML, PO, MO and Fluent, and their Cargo features
- [Plural rules][reference-plural-rules]: CLDR categories, supported languages and gettext `Plural-Forms`
- [Runtime API][reference-runtime-api]: `set_locale`, `Locale`, `locale!`, `available_locales!`, `system_locale`
- [Date and time][reference-date-time]: tokens, supported languages and API
- [Unicode ranges][reference-unicode-ranges]
- [Features and `no_std`][reference-features]: Cargo features, allocation and targets

### Explanation

- [How it works][explanation-how-it-works]: compile-time expansion, why `i18n!` comes first, and the ROM and RAM cost
- [The runtime locale model][explanation-runtime-locale-model]: locale identifiers, per-crate resolution and their limits
- [Design choices][explanation-design-choices]: compile errors, `Display` values and one embedded locale by default
- [Comparison with other crates][explanation-comparison]: rust-i18n, Fluent and gettext

## Cargo Features

| Feature | Description |
| --- | --- |
| `backend-json` | **Default.** JSON locale files |
| `backend-yaml`, `backend-toml` | YAML and TOML locale files |
| `backend-po`, `backend-mo` | GNU gettext `.po` and compiled `.mo` locale files |
| `backend-fluent` | Fluent `.ftl` locale files (simple messages) |
| `alloc` | `String` / `Vec` returning helpers |
| `std` | Enables `alloc`, and `system_locale()` to read the locale of the operating system |

The backends only affect the procedural macros: they add no code and no dependency to the firmware.

## Minimum Supported Rust Version

Rust 1.85 (edition 2024). Raising it is not considered a breaking change.

## Contributing

Issues and pull requests are welcome. Pull request titles follow [Conventional Commits](https://www.conventionalcommits.org) (`feat: ...`, `fix: ...`), as they become the changelog; releases are automated, see [RELEASING.md](RELEASING.md).

## License

Licensed under the <a href="LICENSE">MIT License</a>.

---

Developed by [Alix ANNERAUD](https://alix.anneraud.fr).

[tutorial-getting-started]: docs/tutorials/getting_started.md
[how-to-switch-language]: docs/how_to/switch_language_at_runtime.md
[how-to-localize-per-request]: docs/how_to/localize_per_request.md
[how-to-build-per-language]: docs/how_to/build_one_firmware_per_language.md
[how-to-use-po-files]: docs/how_to/use_gettext_po_files.md
[how-to-ship-complete]: docs/how_to/ship_complete_translations.md
[how-to-pseudo-localize]: docs/how_to/pseudo_localize.md
[how-to-right-to-left]: docs/how_to/support_right_to_left_languages.md
[how-to-format-dates]: docs/how_to/format_dates_and_times.md
[how-to-subset-fonts]: docs/how_to/subset_fonts.md
[how-to-migrate-rust-i18n]: docs/how_to/migrate_from_rust_i18n.md
[how-to-migrate-0-1]: docs/how_to/migrate_from_0_1.md
[reference-t]: docs/reference/t_macro.md
[reference-configuration]: docs/reference/configuration.md
[reference-locale-files]: docs/reference/locale_files.md
[reference-plural-rules]: docs/reference/plural_rules.md
[reference-runtime-api]: docs/reference/runtime_api.md
[reference-date-time]: docs/reference/date_time.md
[reference-unicode-ranges]: docs/reference/unicode_ranges.md
[reference-features]: docs/reference/features_and_no_std.md
[explanation-how-it-works]: docs/explanation/how_it_works.md
[explanation-runtime-locale-model]: docs/explanation/runtime_locale_model.md
[explanation-design-choices]: docs/explanation/design_choices.md
[explanation-comparison]: docs/explanation/comparison.md
