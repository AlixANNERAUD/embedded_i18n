# How It Works

`embedded-i18n` does its work while your crate compiles.
The procedural macros read the locale files on the build machine and replace each [`t!`] call with the translation it selects, so the firmware contains string literals and, for plurals, selects and runtime locales, a few comparisons.
This page explains what the macros generate, why [`i18n!`] must come first, and what each feature costs in ROM and RAM.

## Translations Become Literals

Without arguments, [`t!`]`("greeting")` expands to the literal `"Hello"`, exactly as if you had written it.
It costs nothing at runtime, can initialize `const` and `static` items, and the firmware only contains the strings it uses, in the language it is built for.
Building with `default_locale = "fr"` embeds `"Bonjour"` instead, and the English strings are not in the binary.

The parsers of the locale formats (JSON, YAML, TOML, gettext, Fluent) run inside the procedural macros.
They are build dependencies only: whatever the formats you enable, they add no code to the firmware.

Everything else the macros generate is small and checked at compile time:

- with arguments, a value implementing `Display` that formats the literal format string of the translation with `core::fmt`, as `format_args!` would;
- with `count`, the plural rule of the locale compiled into an `if` / `match` on the count, each arm being the expansion of one form;
- with `select`, a `match` on string literals, each arm being one variant;
- with `bidi_isolation` or the `pseudo` locale, a transformed literal, computed at compile time.

Since the macros see every translation they may select, they check the placeholders of every locale, plural form and select variant against the arguments, and report a mismatch as a compile error at the call site.

## How `t!` Finds the Configuration

A procedural macro only sees its own tokens: [`t!`] cannot read the arguments of an [`i18n!`] call written elsewhere in the crate.
The configuration therefore travels through a crate-local `macro_rules!` macro:

1. [`i18n!`] reads its arguments, the `[package.metadata.i18n]` table of `Cargo.toml` and the environment overrides, resolves the full configuration (the absolute path of the locale files, the fallback chain, the embedded locales, the flags), and expands to a hidden `macro_rules!` named `embedded_i18n_requires_i18n_at_the_crate_root`, whose body forwards its tokens to the procedural macro `__expand!`, with the resolved configuration prepended.
2. [`t!`], [`locale!`] and [`available_locales!`] are `macro_rules!` macros exported by `embedded_i18n`. They expand to `embedded_i18n_requires_i18n_at_the_crate_root!` by its bare name, which is resolved at the call site, with a marker saying what is requested.
3. `__expand!` parses the configuration and the call, loads the locale files (cached per configuration in the compiler process) and expands the translation.

This has a few consequences:

- [`i18n!`] must come before the `mod` declarations that use [`t!`], because `macro_rules!` macros are only visible after their definition in the source order, as in rust-i18n.
- Each crate that translates calls [`i18n!`] once. Integration tests, examples and doctests are separate crates, so each calls it too, and may have its own configuration.
- Forgetting [`i18n!`] reports `` cannot find macro `embedded_i18n_requires_i18n_at_the_crate_root` ``, whose name says what to do.
- A second call in the same crate is a compile error, so that a crate never has two configurations.

## Rebuild Tracking

Cargo rebuilds a crate when its sources change, but it does not know that macros read other files.
[`i18n!`] therefore expands to an `include_bytes!` of every locale file it read and of `Cargo.toml`, and to an `option_env!` of `EMBEDDED_I18N_LOCALE` and `EMBEDDED_I18N_LOCALES`.
The compiler records these as dependencies of the crate, so Cargo rebuilds it when one of them changes; the bytes themselves are not kept in the binary.

Only the files that existed at the previous build are recorded, which is why a locale file created later, such as a new `fr-CA.json` next to an existing `fr.json`, is not seen until the crate is rebuilt for another reason.
A build script with `cargo:rerun-if-changed=locales` watches the whole directory instead, see [Configuration][reference-configuration].

## Several Locales

With several `available_locales`, the translation depends on a runtime value, the current locale:

- [`i18n!`] also generates the crate's resolution function, which maps the current locale to the index of one of the crate's locales, as a `match` on integers;
- [`t!`] without arguments expands to an index into a static array holding one literal per locale;
- with arguments, plural or select, [`t!`] expands to a `match` on that index, each arm being the single-locale expansion of that locale;
- [`locale!`] indexes an array of names the same way.

Each embedded locale is resolved at compile time with its own fallback chain, and every message must be found for every embedded locale, otherwise the build fails and names the locales missing it.
A call with a literal `locale = "fr"` is resolved at compile time and still expands to the French literal.
With a single embedded locale (`available_locales = ["fr"]`), every call expands to a literal, as by default.

The current locale is a global `AtomicU32` used with plain loads and stores: no compare-and-swap, so it works on `thumbv6m-none-eabi`, and no allocation, lock or critical section.
It is linked only when [`set_locale`] or a crate embedding several locales is.
See [The Runtime Locale Model][explanation-runtime-locale-model] for what it stores.

## ROM and RAM Cost

| Situation | ROM | RAM | Per call |
| --- | --- | --- | --- |
| Single locale, `t!("key")` | the literal | none | none |
| Single locale, `t!` with arguments | the literal and the `fmt` code | none | one `Display` indirection |
| Explicit literal locale | the selected literal | none | none |
| Several locales | every embedded locale's strings, an array per `t!` call site, one resolution function per crate | 4 bytes | the resolution `match` and an index |
| `std` and [`system_locale`] | only if called | an allocation per call | |

Date and time names follow the same logic: [`date_time_locale`] is a `const fn`, so with the [`locale!`] of a single-locale crate it is resolved at compile time and only the names of that locale are linked, while a runtime locale links the whole table.

[reference-configuration]: ../reference/configuration.md
[explanation-runtime-locale-model]: runtime_locale_model.md
[`t!`]: ../reference/t_macro.md
[`i18n!`]: ../reference/configuration.md
[`locale!`]: ../reference/runtime_api.md
[`available_locales!`]: ../reference/runtime_api.md
[`set_locale`]: ../reference/runtime_api.md
[`system_locale`]: ../reference/runtime_api.md
[`date_time_locale`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.date_time_locale.html
