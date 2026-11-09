# embedded-i18n

[![crates.io](https://img.shields.io/crates/v/embedded-i18n.svg)](https://crates.io/crates/embedded-i18n)
[![docs.rs](https://img.shields.io/docsrs/embedded-i18n)](https://docs.rs/embedded-i18n)
[![CI](https://github.com/AlixANNERAUD/embedded_i18n/actions/workflows/check.yml/badge.svg)](https://github.com/AlixANNERAUD/embedded_i18n/actions/workflows/check.yml)
[![codecov](https://codecov.io/github/AlixANNERAUD/embedded_i18n/graph/badge.svg?token=0C27GIS0BK)](https://codecov.io/github/AlixANNERAUD/embedded_i18n)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![MSRV 1.85](https://img.shields.io/badge/MSRV-1.85-orange.svg)](#minimum-supported-rust-version)
[![no_std](https://img.shields.io/badge/no__std-no%20alloc-success.svg)](docs/no_std.md)

**Compile-time internationalization (i18n) and localization (l10n) for embedded Rust, `no_std` firmware and microcontrollers.**

`embedded-i18n` translates your strings **at build time**: translations from JSON, YAML, TOML, GNU gettext PO/MO or Fluent files are compiled into your binary as plain `&'static str` literals stored in flash/ROM. No runtime parser, no allocator, no file system, no heap, and a missing translation is a **compile error**, not a blank screen on a device in the field.

```rust
use embedded_i18n::{translate, translate_plural_write};

const TITLE: &str = translate!("Settings");        // EMBEDDED_I18N_LOCALE=fr -> "Paramètres"

translate_plural_write!(&mut display, "files", count)?; // "1 fichier", "3 fichiers"
```

## Why embedded-i18n?

- **Zero runtime cost.** Each `translate!()` expands to a string literal. Your firmware only contains the strings it uses, in the language you build for.
- **Fits on a microcontroller.** `#![no_std]`, no allocator, no `std::fs`, no `libc`. Builds for bare-metal targets such as `thumbv7em-none-eabihf` and `thumbv6m-none-eabi` (no atomic compare-and-swap).
- **Mistakes caught by the compiler.** Missing keys, missing plural forms, invalid locale files and format placeholders that do not match their arguments are reported as compile errors at the call site. An optional strict mode forbids falling back to another language.
- **Real-world linguistics.** CLDR plural rules for 40 languages, gettext `Plural-Forms`, message context (`msgctxt`), select variants (gender, enums), locale fallback chains (`fr-CA` → `fr` → `en`), right-to-left bidi isolation and localized dates without `libc`.
- **Use the files translators already know.** JSON (nested keys), YAML, TOML, GNU gettext `.po` / `.mo`, and Fluent `.ftl`, all parsed at compile time, so the formats add no code to the firmware.
- **Runtime language switching when you need it.** Embed several locales and switch with `set_locale("de")`, still without allocation.

## Comparison

| | **embedded-i18n** | [rust-i18n](https://crates.io/crates/rust-i18n) | [Fluent](https://crates.io/crates/fluent) | [gettext-rs](https://crates.io/crates/gettext-rs) |
| --- | --- | --- | --- | --- |
| `no_std`, no allocator | ✅ | ❌ | ❌ | ❌ |
| Translations resolved | At compile time, as literals | Compile-time embedded, looked up at runtime | Parsed and resolved at runtime | Loaded from `.mo` files at runtime |
| Missing translation | Compile error | Runtime fallback | Runtime error / fallback | Runtime fallback to the source string |
| Format placeholders checked at compile time | ✅ (`format_args!`) | ❌ | ❌ | ❌ |
| Plural forms | ✅ CLDR rules and gettext `Plural-Forms` | Not built in | ✅ | ✅ |
| Message context / select variants | ✅ / ✅ | ❌ / ❌ | Message ids / ✅ | ✅ / ❌ |
| Runtime locale switching | Optional (`EMBEDDED_I18N_LOCALES`) | ✅ | ✅ | ✅ |
| File formats | JSON, YAML, TOML, PO, MO, Fluent (subset) | YAML, JSON, TOML | Fluent | MO (via GNU gettext) |
| Localized dates without `libc` | ✅ | ❌ | ❌ | ❌ |

Pick `embedded-i18n` when the binary must be small, deterministic and allocation-free, and the set of languages is known at build time. On desktop or server applications with downloadable language packs and full Fluent features (functions, parameterized terms), a runtime library is a better fit.

## Quick start (60 seconds)

1. Add the crate:

   ```toml
   [dependencies]
   embedded-i18n = "0.1"
   ```

2. Create `locales/fr.json` next to your `Cargo.toml`, and a `locales/en.json` with the same keys in English (the default fallback):

   ```json
   {
     "Settings": "Paramètres",
     "Hello {name}!": "Bonjour {name} !",
     "files": { "one": "{count} fichier", "other": "{count} fichiers" }
   }
   ```

3. Translate:

   ```rust
   use core::fmt::Write;
   use embedded_i18n::{translate, translate_plural_write, translate_write};

   embedded_i18n::track_locales!();

   fn render(display: &mut impl Write, name: &str, count: usize) -> core::fmt::Result {
       display.write_str(translate!("Settings"))?;
       translate_write!(display, "Hello {name}!")?;
       translate_plural_write!(display, "files", count)
   }
   ```

4. Build for a language:

   ```sh
   EMBEDDED_I18N_LOCALE=fr cargo build --release
   ```

   Set it once in `.cargo/config.toml` (`[env] EMBEDDED_I18N_LOCALE = "fr"`) to avoid repeating it.

## Features at a glance

**Locale files in the formats translators already use**: JSON with nested keys, YAML, TOML, GNU gettext `.po` / `.mo` and Fluent `.ftl`, parsed at compile time. [Guide][guide-locale-files]

**Message context**, to translate the same source string differently. [Guide][guide-context]

```rust
let action = translate!(context = "menu", "Open"); // "Ouvrir"
let state = translate!(context = "state", "Open"); // "Ouvert"
```

**Plural forms** with CLDR rules for 40 languages, or any gettext `Plural-Forms` expression. [Guide][guide-plurals]

```rust
translate_plural_write!(&mut display, "files", count)?; // "1 plik", "3 pliki", "5 plików"
```

**Select variants** for genders, states and enums. [Guide][guide-select]

```rust
let reply = translate_select!("replied", user.gender); // "Elle a répondu"
```

**Formatting without allocation**, with placeholders checked at compile time in every language. [Guide][guide-formatting]

```rust
translate_write!(&mut display, "Hello {name}, you have {count} messages")?;
```

**Fallback chains and strict builds**: `fr-CA` → `fr` → `en`, and `EMBEDDED_I18N_STRICT=1` to fail the build on untranslated strings. [Configuration][guide-configuration], [Coverage][guide-coverage]

**Runtime language switching** between locales embedded at build time, with one atomic and no allocation. [Guide][guide-runtime-locales]

```rust
set_locale("de")?;
```

**Localized dates and times** without `libc`: month and weekday names, AM/PM, date patterns and UTC offsets for 30 languages. [Guide][guide-date-time]

```rust
DateTimeFormatter::new().write(&mut display, timestamp, "%A %d %B %Y")?; // "mardi 13 octobre 2026"
```

**Right-to-left languages, pseudo-localization and font subsetting**: bidi isolation of placeholders, a `pseudo` locale to catch clipped and hardcoded strings, and the Unicode ranges of each language for font conversion tools. [RTL][guide-right-to-left], [Pseudo-localization][guide-pseudo-localization], [Unicode ranges][guide-unicode-ranges]

## Guide

| Topic | |
| --- | --- |
| [Locale Files][guide-locale-files] | JSON, YAML, TOML, PO, MO and Fluent files, and their Cargo features |
| [Translating][guide-translating] | `translate!`, C strings, lookup and compile errors |
| [Message Context][guide-context] | Disambiguating identical source strings |
| [Plural Forms][guide-plurals] | CLDR categories, gettext `Plural-Forms` and the supported languages |
| [Select Variants][guide-select] | Gender, states and enums |
| [Formatting][guide-formatting] | `translate_write!`, `translate_args!`, `format!` and `defmt` |
| [Rebuild Tracking][guide-rebuild-tracking] | Rebuilding when locale files change |
| [Configuration][guide-configuration] | Environment variables, locale resolution, fallback chains and the locales directory |
| [Runtime Locale Selection][guide-runtime-locales] | Embedding several locales and switching at runtime |
| [Translation Coverage][guide-coverage] | Strict mode for complete releases |
| [Right-to-Left Languages][guide-right-to-left] | Text direction and bidi isolation |
| [Pseudo-Localization][guide-pseudo-localization] | Testing layouts before translating |
| [Date and Time][guide-date-time] | Localized `strftime`-style formatting |
| [Unicode Ranges][guide-unicode-ranges] | Font subsetting for each language |
| [`no_std` Support][guide-no-std] | Features, allocation and targets |
| [Comparison][guide-comparison] | rust-i18n, Fluent and gettext, and how to migrate |

The guide is also available on [docs.rs](https://docs.rs/embedded-i18n/latest/embedded_i18n/guide/index.html), with every example compiled and tested.

## Cargo features

| Feature | Description |
| --- | --- |
| `backend-json` | **Default.** JSON locale files |
| `backend-yaml`, `backend-toml` | YAML and TOML locale files |
| `backend-po`, `backend-mo` | GNU gettext `.po` and compiled `.mo` locale files |
| `backend-fluent` | Fluent `.ftl` locale files (simple messages) |
| `alloc` | `String` / `Vec` returning helpers |
| `std` | Enables `alloc`, and environment getters for build scripts |

The backends only affect the procedural macros: they add no code and no dependency to the firmware.

## Minimum Supported Rust Version

Rust 1.85 (edition 2024). Raising it is not considered a breaking change.

## Contributing

Issues and pull requests are welcome. Pull request titles follow [Conventional Commits](https://www.conventionalcommits.org) (`feat: ...`, `fix: ...`), as they become the changelog; releases are automated, see [RELEASING.md](RELEASING.md).

## License

Licensed under the <a href="LICENSE">MIT License</a>.

---

Developed by [Alix ANNERAUD](https://alix.anneraud.fr).

[guide-locale-files]: docs/locale_files.md
[guide-translating]: docs/translating.md
[guide-context]: docs/context.md
[guide-plurals]: docs/plurals.md
[guide-select]: docs/select.md
[guide-formatting]: docs/formatting.md
[guide-rebuild-tracking]: docs/rebuild_tracking.md
[guide-configuration]: docs/configuration.md
[guide-runtime-locales]: docs/runtime_locales.md
[guide-coverage]: docs/coverage.md
[guide-right-to-left]: docs/right_to_left.md
[guide-pseudo-localization]: docs/pseudo_localization.md
[guide-date-time]: docs/date_time.md
[guide-unicode-ranges]: docs/unicode_ranges.md
[guide-no-std]: docs/no_std.md
[guide-comparison]: docs/comparison.md
