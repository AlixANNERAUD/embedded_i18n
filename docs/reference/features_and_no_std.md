# Features and `no_std`

`embedded-i18n` is `#![no_std]` and needs no allocator by default.

## Cargo Features

| Feature | Description |
| --- | --- |
| `backend-json` | **Default.** JSON locale files |
| `backend-yaml`, `backend-toml` | YAML and TOML locale files |
| `backend-po`, `backend-mo` | GNU gettext `.po` and compiled `.mo` locale files |
| `backend-fluent` | Fluent `.ftl` locale files (subset) |
| `alloc` | `String` / `Vec` returning helpers: `format_unix_timestamp`, `DateTimeFormatter::format`, `merge_contiguous_ranges`, `format_range`, `format_ranges` |
| `std` | Enables `alloc`, and [`system_locale`], which reads the locale of the operating system from the environment |

The backends only affect the procedural macros: they add no code and no dependency to the firmware.
See [Locale Files][reference-locale-files].

Runtime locale selection is not a feature: it is enabled per crate by `available_locales`, see [Configuration][reference-configuration].

## What Each Part Needs

| Part | Needs |
| --- | --- |
| [`t!`] without arguments, plural, select and context | nothing: `&'static str` or `&'static CStr` literals |
| [`t!`] with arguments | [`core::fmt`], through a `Display` value |
| Several embedded locales, [`set_locale`] | an `AtomicU32` with load and store, no compare-and-swap |
| [Date and time][reference-date-time], [Unicode ranges][reference-unicode-ranges] | [`core::fmt`], writing into a [`core::fmt::Write`] |
| `String` and `Vec` returning helpers | the `alloc` feature |
| [`system_locale`] | the `std` feature |

All the parsing happens in the procedural macros, on the build machine: the firmware only contains string literals and, for plurals, selects and runtime locales, a few comparisons.
Results with arguments are written into any [`core::fmt::Write`]: a display driver, a fixed-size buffer, or a fixed-capacity string from a crate such as `heapless`.

## Targets

CI builds the crate for `thumbv7em-none-eabihf` (Cortex-M4F) with the JSON and PO backends, with and without `alloc`, and for `thumbv6m-none-eabi` (Cortex-M0) with the JSON backend.
The runtime locale selection only uses atomic loads and stores, so it builds for targets without compare-and-swap, such as `thumbv6m-none-eabi`.
Any target with `core` and 32-bit atomic loads and stores works.

## Minimum Supported Rust Version

Rust 1.85 (edition 2024), for the procedural macros and the runtime crate.

[reference-configuration]: configuration.md
[reference-locale-files]: locale_files.md
[reference-date-time]: date_time.md
[reference-unicode-ranges]: unicode_ranges.md
[`t!`]: t_macro.md
[`set_locale`]: runtime_api.md
[`system_locale`]: runtime_api.md
[`core::fmt::Write`]: https://doc.rust-lang.org/core/fmt/trait.Write.html
[`core::fmt`]: https://doc.rust-lang.org/core/fmt/index.html
