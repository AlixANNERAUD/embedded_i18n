# Runtime Locale Selection

By default a firmware embeds one locale, and every [`t!`] call expands to a literal.
When the user picks the language at runtime (a settings menu, a DIP switch, a configuration stored in flash), list the locales to embed in [`i18n!`]:

```rust,no_run
embedded_i18n::i18n!("locales", default_locale = "en", available_locales = ["en", "fr", "de"]);
# fn main() {}
```

or in `[package.metadata.i18n]`:

```toml
[package.metadata.i18n]
default-locale = "en"
available-locales = ["en", "fr", "de"]
```

This is configuration, not a Cargo feature: it is decided for each crate, while Cargo features are unified across the dependency graph.
[`t!`] then selects the translation of the current locale, and [`set_locale`] changes it:

```rust
use embedded_i18n::{set_locale, t};

embedded_i18n::i18n!("locales", default_locale = "en", available_locales = ["en", "fr"]);

assert_eq!(t!("greeting"), "Hello");

for &language in embedded_i18n::available_locales!() {
    set_locale(language);
    assert_eq!(embedded_i18n::locale!(), language);
    let greeting: &'static str = t!("greeting");
    // en: "Hello", fr: "Bonjour"
    assert!(!greeting.is_empty());
}

set_locale("fr-CA");
assert_eq!(t!("greeting"), "Bonjour");
```

## API

| Item | Description |
| --- | --- |
| [`set_locale`]`(&str)` | Sets the global current locale. It returns `()`, as in rust-i18n: each crate falls back to its closest embedded locale |
| [`Locale`] | `Copy` handle of a locale tag: `Locale::new("fr-CA")` (`const fn`), `From<&str>`, `PartialEq`, `.id()`, `Locale::NONE` |
| [`locale!`]`()` | Name of the locale the calling crate translates into, a `&'static str`, and a literal with a single locale |
| [`available_locales!`]`()` | Embedded locales of the calling crate, a `&'static [&'static str]`, in configuration order |
| [`system_locale`]`()` | With the `std` feature, the locale of the operating system as an `Option<String>` |

The `EMBEDDED_I18N_LOCALES` variable overrides `available_locales` at build time, see [Configuration][guide-configuration].
Code calling these items builds with a single locale too: [`set_locale`] has no effect then, since only the default locale exists.

## Locale Identifiers and Resolution

The global state is a compact identifier of the requested tag, not an index into the locales of one crate, so that crates embedding different locales can share it.
[`Locale`] packs a tag into a `u32`:

- the language subtag (up to three letters) is stored exactly in the high bits;
- the rest of the tag (script, region, variants) is hashed into the low 17 bits;
- tags are normalized first: case-insensitive, `_` equivalent to `-`, surrounding whitespace ignored;
- a language subtag longer than three letters (`pseudo`) is hashed whole and only matches exactly;
- `Locale::NONE` means that no locale was set, and every crate then uses its `default_locale`.

Each crate maps the requested locale to one of its embedded locales, in this order:

1. an exact identifier match;
2. the same language: the locale that is the language alone (`fr`) if embedded, otherwise the first embedded locale of that language (`fr-CA`);
3. the `default_locale`.

The mapping is a `match` generated at compile time: a few comparisons, no lookup by string.
Two embedded locales with the same identifier are a compile error.

> **Limitation.** The 17 low bits keep no readable script, so the same-language step ignores scripts: a request for `zh-Hans` can be served by an embedded `zh-Hant` when `zh-Hans` is not embedded.
> Embed the exact tags you support (`zh-Hans` and `zh-Hant`) to get an exact match.

## Explicit Locale per Call

`locale = ...` translates one message into another locale without touching the global one, for servers or several displays showing different languages at once:

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales", default_locale = "en", available_locales = ["en", "fr"]);

// A string literal is resolved at compile time: this is the French literal.
assert_eq!(t!("greeting", locale = "fr"), "Bonjour");

// Any `Into<Locale>` (`&str`, `Locale`) is resolved at runtime.
let language = "fr";
assert_eq!(t!("greeting", locale = language), "Bonjour");
```

With a single locale, the expression is evaluated and ignored, so the same code builds in both modes.

## System Locale

With the `std` feature, [`system_locale`] reads `LC_ALL`, `LC_MESSAGES`, then `LANG`, skipping empty, `C` and `POSIX` values, and normalizes `fr_FR.UTF-8@euro` to `fr-FR`:

```rust
use embedded_i18n::{set_locale, system_locale};

embedded_i18n::i18n!("locales", available_locales = ["en", "fr"]);

set_locale(&system_locale().unwrap_or_default());
```

It allocates a `String` on each call, so call it once at startup.
Windows and macOS graphical applications have no such variables: use the [`sys-locale`](https://crates.io/crates/sys-locale) crate there.

## How It Expands

- [`t!`] without arguments expands to an index into a static array with one literal per locale.
- With arguments, plural or select, [`t!`] expands to a `match` on that index, each arm being the single-locale expansion of that locale.
- [`locale!`] indexes an array of names the same way.
- Each embedded locale is resolved at compile time with its own fallback chain, and every message must be found for every embedded locale, otherwise the build fails and names the locales missing it.
- The result of [`t!`] with arguments formats through a `Display` value, so formatting flags such as width or alignment are not applied to the whole message.

With a single embedded locale (`available_locales = ["fr"]`), every call still expands to a literal.

## Limitations

With several locales, [`t!`] returns a runtime value, so it cannot initialize a `const` or `static`, and the compiler reports a non-const expression.
A call with a literal `locale = "fr"` is resolved at compile time and expands to the French literal, with no runtime cost.

To format dates in the current locale, use `DateTimeFormatter::new(date_time_locale(locale!()))`, which links the date and time names of every supported language when the locale is a runtime value (see [Date and Time][guide-date-time]).

## ROM and RAM Cost

| Situation | ROM | RAM | Per call |
| --- | --- | --- | --- |
| Single locale, `t!("key")` | the literal | none | none |
| Single locale, `t!` with arguments | the literal and the `fmt` code | none | one `Display` indirection |
| Explicit literal locale | the selected literal | none | none |
| Several locales | every embedded locale's strings, an array per `t!` call site, one resolution function per crate | 4 bytes | the resolution `match` and an index |
| `std` and [`system_locale`] | only if called | an allocation per call | |

The current locale is a global `AtomicU32` used with plain loads and stores: no compare-and-swap, so it works on `thumbv6m-none-eabi`, and no allocation, lock or critical section.
It is linked only when [`set_locale`] or a crate embedding several locales is.
Up to 255 locales can be embedded in a crate.

[guide-configuration]: configuration.md
[guide-date-time]: date_time.md
[`t!`]: translating.md
[`i18n!`]: configuration.md
[`set_locale`]: runtime_locales.md
[`Locale`]: runtime_locales.md
[`locale!`]: runtime_locales.md
[`available_locales!`]: runtime_locales.md
[`system_locale`]: runtime_locales.md
