# The Runtime Locale Model

When several locales are embedded, [`set_locale`] selects the language and every [`t!`] call follows it.
This page explains what the current locale is, how each crate turns it into one of its own locales, and the limits of this design.

## Why Not a Global Index

A program is made of several crates, and each crate calling [`i18n!`] has its own configuration: a firmware can embed English, French and German, while a library it uses only embeds English and French, and another dependency only English.
An index into the locales of one crate would be meaningless to the others.

The global state is therefore a compact identifier of the requested tag, a [`Locale`], and each crate maps it to one of its embedded locales.
That is also why [`set_locale`] returns `()`, as in rust-i18n, and never fails: a locale that one crate does not embed can still be served by another, and each crate falls back to its closest embedded locale.
[`locale!`] then tells which locale the calling crate actually translates into.

For the same reason, runtime selection is configuration (`available_locales`) rather than a Cargo feature: it is decided for each crate, while Cargo features are unified across the dependency graph.

## The Packed Identifier

[`Locale`] packs a tag into a `u32`, with a `const fn`, so tags written in the code cost nothing at runtime:

- the language subtag, up to three letters, is stored exactly in the high 15 bits, 5 bits per letter;
- the rest of the tag (script, region, variants) is hashed into the low 17 bits;
- tags are normalized first: case-insensitive, `_` equivalent to `-`, surrounding whitespace ignored;
- a language subtag longer than three letters (`pseudo`, rare registered subtags) gets a reserved language value and is hashed whole, so it only matches exactly;
- `Locale::NONE` (`0`) means that no locale was set, and every crate then uses its `default_locale`.

Storing the language exactly is what makes the language fallback reliable: `fr-CA` and `fr-BE` are always recognized as French, whatever their hash.
A 32-bit identifier fits in an `AtomicU32` read and written with plain loads and stores, which every target with atomics supports, including those without compare-and-swap such as `thumbv6m-none-eabi`, and takes 4 bytes of RAM.

## Per-Crate Resolution

Each crate embedding several locales maps the requested identifier to one of its locales, in this order:

1. an exact identifier match;
2. the same language: the locale that is the language alone (`fr`) if embedded, otherwise the first embedded locale of that language (`fr-CA`);
3. the `default_locale`.

[`i18n!`] generates this mapping as a `match` at compile time: a few integer comparisons, no lookup by string.
Two embedded locales with the same identifier could not be told apart, so they are a compile error.

The same mapping serves `t!("key", locale = ...)`, which translates into a given locale without touching the global one, for servers or for several displays showing different languages at once.
With a literal locale, the mapping runs at compile time, so the call is the selected literal.

## Limitation: Scripts

The 17 low bits keep no readable script, so the same-language step ignores scripts: a request for `zh-Hans` can be served by an embedded `zh-Hant` when `zh-Hans` is not embedded.
Embed the exact tags you support (`zh-Hans` and `zh-Hant`) to get an exact match.

Hash collisions between two different tags of the same language are possible in theory; the compile error on embedded locales guarantees that a crate's own locales are distinct, and a colliding request would at worst select another region or script of the same language.

## Limitation: Runtime Values

With several locales, [`t!`] returns a runtime value, so it cannot initialize a `const` or `static` item, and the compiler reports a non-const expression.
Code calling [`set_locale`], [`locale!`] and [`available_locales!`] still builds with a single locale, where [`set_locale`] has no effect, so libraries can use them whatever the configuration of the final firmware.

To format dates in the current locale, `DateTimeFormatter::new(date_time_locale(locale!()))` works in both modes, but links the date and time names of every supported language when the locale is a runtime value.

## Reading the System Locale

[`system_locale`], with the `std` feature, turns the POSIX locale variables into a tag (`fr_FR.UTF-8@euro` becomes `fr-FR`) to pass to [`set_locale`].
It allocates a `String` on each call, and is meant to be called once at startup.

[`t!`]: ../reference/t_macro.md
[`i18n!`]: ../reference/configuration.md
[`set_locale`]: ../reference/runtime_api.md
[`Locale`]: ../reference/runtime_api.md
[`locale!`]: ../reference/runtime_api.md
[`available_locales!`]: ../reference/runtime_api.md
[`system_locale`]: ../reference/runtime_api.md
