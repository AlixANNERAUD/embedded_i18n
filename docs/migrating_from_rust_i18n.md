# Migrating From rust-i18n

`embedded-i18n` 0.2 follows the API of [rust-i18n](https://crates.io/crates/rust-i18n): `i18n!` and `t!` have the same names, most of the same syntax, and read the same kind of locale files.
What differs is where the work happens.
rust-i18n looks translations up at runtime and returns owned strings, while `embedded-i18n` resolves them at compile time and returns literals, so it needs no allocator and embeds one language unless you ask for more.

## `i18n!` and `[package.metadata.i18n]`

Call [`i18n!`] once at the crate root, before the `mod` declarations using [`t!`], as with `rust_i18n::i18n!`:

```rust
embedded_i18n::i18n!("locales", fallback = "en");
# fn main() {}
```

The options and the metadata keys are those of rust-i18n where they exist:

| rust-i18n | embedded-i18n |
| --- | --- |
| `i18n!("locales")` | `i18n!("locales")` |
| `i18n!("locales", fallback = "en")`, `fallback = ["en", "es"]` | the same |
| `[package.metadata.i18n]` `load-path` | the same, and the default is `"locales"` too |
| `default-locale`, `available-locales` | the same, see [below](#only-default_locale-is-embedded-by-default) |
| `minify-key`, `minify-key-len`, `minify-key-prefix`, `minify-key-thresh` | not provided, see [Features Not Provided](#features-not-provided) |
| | `strict` and `bidi-isolation`, see [Coverage][guide-coverage] and [Right-to-Left Languages][guide-right-to-left] |

Each integration test, example and doctest that translates calls [`i18n!`] once, as it is a separate crate.
Forgetting the call reports `cannot find macro embedded_i18n_requires_i18n_at_the_crate_root`.
See [Configuration][guide-configuration] for the full list of options and the precedence rules.

The locale files keep their formats: JSON, YAML and TOML files work with the matching backend features, one file per locale (`en.yml`, `fr.json`).
Files containing several locales at once are not supported.

## `t!` Syntax That Maps 1:1

| rust-i18n | embedded-i18n |
| --- | --- |
| `t!("hello")` | `t!("hello")` |
| `t!("messages.hello", name = user)` | `t!("messages.hello", name = user)` |
| `t!("messages.hello", "name" => user)` | `t!("messages.hello", "name" => user)` |
| `t!("serial", sn = 123 : {:08})` | `t!("serial", sn = 123 : {:08})` |
| `t!("hello", locale = "fr")` | `t!("hello", locale = "fr")` |
| `%{name}` in locale files | `%{name}`, and `{name}` (Rust format syntax) |

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

assert_eq!(t!("greeting"), "Hello");
assert_eq!(t!("with_arg", value = 42).to_string(), "Value: 42");
assert_eq!(t!("with_arg", "value" => 42).to_string(), "Value: 42");
assert_eq!(t!("with_arg", value = 42 : {:05}).to_string(), "Value: 00042");
assert_eq!(t!("greeting", locale = "en"), "Hello");
```

[`t!`] adds options for what rust-i18n leaves to its own data: `count = n` for [plural forms][guide-plurals], `select = value` for [select variants][guide-select] and `context = "menu"` for a [message context][guide-context].
Positional arguments are not supported, as in rust-i18n.
A translation containing `%{` is read in rust-i18n syntax: `%{name}` is a placeholder and other braces are literal.

## `Cow<str>` vs `&'static str` and `Display`

`rust_i18n::t!` returns a `Cow<str>`, which needs an allocator.
[`t!`] returns:

- a `&'static str` (or `&'static CStr`) without named arguments or `count`, a literal usable in `const` items when a single locale is embedded;
- a value implementing `core::fmt::Display` otherwise, which formats on demand without allocation.

| rust-i18n | embedded-i18n |
| --- | --- |
| `let text: String = t!("hello").into_owned()` | `let text = t!("hello").to_string()` with `alloc` |
| `&*t!("hello")` | `t!("hello")`, already a `&'static str` |
| `t!("hello", name = n).to_string()` | the same |
| `println!("{}", t!("hello", name = n))` | the same |
| `format!("{}", t!("hello", name = n))` | the same |

Without an allocator, write the result with `write!`:

```rust
use core::fmt::Write;
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

let mut output = String::new();
write!(output, "{}", t!("files", count = 3))?;
assert_eq!(output, "3 files");
# Ok::<(), core::fmt::Error>(())
```

`&*t!(..)` is not supported on a result with arguments.
The arguments are evaluated once and borrowed, as by `format_args!`, so temporaries live until the end of the enclosing statement.
Storing the result in a `let` works when the arguments are variables or fields, not temporaries.
See [Formatting][guide-formatting].

## Missing Keys and Placeholders Are Compile Errors

rust-i18n returns the key (or logs a miss) for a missing translation, and leaves a `%{name}` without argument in the text.
`embedded-i18n` reports both at the call site, in every locale you build:

```rust,compile_fail
# embedded_i18n::i18n!("locales");
// The translation uses `{name}` and `{count}`.
let _ = embedded_i18n::t!("with_named_args", name = "Ada").to_string();
```

- a key missing from every locale of the fallback chain, naming the key and the chain;
- a placeholder without a matching argument, naming the key, the locale and the placeholder;
- with `strict = true`, a message missing from the primary locale, see [Coverage][guide-coverage].

An argument that a translation does not use is allowed, since a singular plural form may not use `{count}`.

## Only `default_locale` Is Embedded by Default

rust-i18n embeds every locale file of the directory and selects one at runtime.
`embedded-i18n` embeds the `default-locale` alone (`"en"` unless configured), so the firmware only contains the strings it can display.
Add `available_locales` to embed several locales and enable runtime selection:

```rust
embedded_i18n::i18n!("locales", default_locale = "en", available_locales = ["en", "fr"]);
# fn main() {}
```

or in `Cargo.toml`:

```toml
[package.metadata.i18n]
default-locale = "en"
available-locales = ["en", "fr"]
```

This is configuration rather than a Cargo feature, as it is decided for each crate.
With one locale, `t!("key")` is a literal; with several, it is an index into an array of literals.
See [Runtime Locale Selection][guide-runtime-locales] for the cost.

## `set_locale`, `locale!` and `available_locales!`

| rust-i18n | embedded-i18n |
| --- | --- |
| `rust_i18n::set_locale("fr")` | [`set_locale`]`("fr")` |
| `rust_i18n::locale()` | [`locale!`]`()`, a `&'static str` |
| `rust_i18n::available_locales!()` | [`available_locales!`]`()`, a `&'static [&'static str]` |

```rust
use embedded_i18n::{set_locale, t};

embedded_i18n::i18n!("locales", default_locale = "en", available_locales = ["en", "fr"]);

set_locale("fr");
assert_eq!(embedded_i18n::locale!(), "fr");
assert_eq!(t!("greeting"), "Bonjour");
assert_eq!(embedded_i18n::available_locales!(), &["en", "fr"]);
```

[`set_locale`] returns `()`, as in rust-i18n: each crate falls back to its closest embedded locale (`fr-CA` selects `fr`), then to its `default_locale`.
[`locale!`] is the locale the calling crate translates into, which can differ from the requested one.
With the `std` feature, [`system_locale`] reads the locale of the operating system.

## Features Not Provided

| rust-i18n | Why |
| --- | --- |
| `Backend` and custom backends | translations are compiled into the binary |
| `load-path` loading at runtime, `extend!` | no translation data is loaded at runtime |
| `minify_key` | keys never reach the binary |
| `log-miss-tr` | a missing translation is a compile error |
| returning the key for a missing translation | the same |

[guide-configuration]: configuration.md
[guide-coverage]: coverage.md
[guide-right-to-left]: right_to_left.md
[guide-plurals]: plurals.md
[guide-select]: select.md
[guide-context]: context.md
[guide-formatting]: formatting.md
[guide-runtime-locales]: runtime_locales.md
[`i18n!`]: configuration.md
[`t!`]: translating.md
[`set_locale`]: runtime_locales.md
[`locale!`]: runtime_locales.md
[`available_locales!`]: runtime_locales.md
[`system_locale`]: runtime_locales.md
