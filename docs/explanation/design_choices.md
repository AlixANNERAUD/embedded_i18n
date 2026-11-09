# Design Choices

`embedded-i18n` targets firmware: flash, RAM and the allocator are scarce, there is no file system to load language packs from, and a mistake found on a device in the field is expensive.
The API follows rust-i18n where it can, and departs from it where these constraints call for it.
This page explains those departures.

## Compile Errors Instead of Runtime Fallbacks

A runtime library that cannot find a translation shows the key, an empty string, or the source language, and maybe logs a warning nobody reads on a device.
`embedded-i18n` sees every translation while compiling, so it reports these mistakes at the call site instead, in every locale you build:

- a key missing from every locale of the fallback chain;
- a placeholder without a matching argument, such as a translator writing `{nom}` instead of `{name}`;
- a missing plural form, an invalid locale file or an invalid `Plural-Forms` expression.

Falling back to another language is still the default, because partial translations are normal during development.
Strict mode turns that fallback into an error too, for release builds, see [How to Ship Complete Translations][how-to-ship-complete].
Building each shipped locale in CI catches all these mistakes before release.

An argument that a translation does not use is allowed, because a singular plural form often omits `{count}`, and a language may not need every value.

## `Display` Instead of `Cow<str>`

`rust_i18n::t!` returns a `Cow<str>`: formatting a message with arguments allocates a `String`.
[`t!`] instead returns:

- a `&'static str` without arguments, a literal that can even initialize `const` items;
- with arguments, a value implementing `core::fmt::Display`, which formats on demand into any `core::fmt::Write`, such as a display driver or a fixed-size buffer, without allocating.

The arguments are evaluated once and borrowed, as by `format_args!`, so the value lives as long as its arguments.
The price is that `&*t!(..)` does not work on a result with arguments, the formatting flags of an outer format string (width, alignment) do not apply to the whole message, and a value with arguments is not a `&str`: write it, or call `.to_string()` with `alloc`.

## Named Placeholders Only

Arguments are always named, and never captured implicitly.
A positional placeholder (`{}`) has no meaning across languages, where the order of words changes; a named one can be moved by each translator, and checked against the arguments of the call.
Requiring `name = name` keeps every value a translation can use visible at the call site.

## One Locale Embedded by Default

rust-i18n embeds every locale file of the directory and selects one at runtime.
`embedded-i18n` embeds the `default_locale` alone, so a firmware only contains the strings it can display, and every [`t!`] call stays a literal.
The same code builds one firmware per language with the `EMBEDDED_I18N_LOCALE` override.

Runtime selection is opt-in, with `available_locales`, and costs 4 bytes of RAM, an array per call site and a resolution function per crate, see [How It Works][explanation-how-it-works].
It is configuration rather than a Cargo feature, because it is decided for each crate while Cargo features are unified across the dependency graph.

## Configuration in the Code, Overrides in the Environment

The configuration lives in the [`i18n!`] arguments or in `Cargo.toml`, so it is versioned with the code and reviewed like it.
Only the two settings that select what to build, the locale and the embedded locales, can be overridden by environment variables, for CI matrices and one-off builds.

## No Key Extraction or Unused-Key Report

There is no built-in extraction of the keys used in the source code (an `xgettext` equivalent), nor a report of keys present in locale files but never used.
A procedural macro only sees its own call site, cannot know when the last call site of a crate has been expanded, and cannot emit warnings on stable Rust, so these reports would need a separate tool scanning the sources.
Strict mode covers the most harmful case, untranslated strings, at compile time.

## Keys Are Not Their Own Translation

gettext displays the `msgid` when a catalog has no translation.
Here, a key must be translated in at least one locale file, such as an English catalog or a `fallback` locale written in the source language.
Otherwise a typo in a key would silently become user-visible text, instead of a compile error.

## A Subset of Fluent

Fluent functions (`NUMBER()`, `DATETIME()`), parameterized terms and selects mixed with text need a resolver running on the device.
`embedded-i18n` supports the subset that can be resolved at compile time: simple messages, terms, attributes, variables and whole-message selects, which cover plural forms and select variants.

## Bidi Isolation Off by Default

Unicode isolates around placeholders keep left-to-right values in place inside right-to-left text, but many embedded text renderers do not implement the Unicode bidirectional algorithm and would draw them as missing glyphs.
They are therefore opt-in, see [How to Support Right-to-Left Languages][how-to-right-to-left].

## Not `defmt` Format Strings

`defmt` interns format strings at compile time and needs them as literal tokens in the source, which a macro expanding to a translation cannot provide.
Translations are logged as `defmt` arguments instead: `defmt::info!("{=str}", t!("greeting"))`.

[how-to-ship-complete]: ../how_to/ship_complete_translations.md
[how-to-right-to-left]: ../how_to/support_right_to_left_languages.md
[explanation-how-it-works]: how_it_works.md
[`t!`]: ../reference/t_macro.md
[`i18n!`]: ../reference/configuration.md
