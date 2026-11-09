# Translating

[`t!`] replaces a message key with its translation, at compile time.
Call [`i18n!`] once at the crate root, before any `mod` declaration, to say where the locale files are (see [Configuration][guide-configuration]).
Without arguments, the expansion is a plain string literal, so the translation costs nothing at runtime and can be used anywhere a literal can:

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

const GREETING: &str = t!("greeting");
static FAREWELL: &str = t!("farewell");

assert_eq!(GREETING, "Hello");
assert_eq!(FAREWELL, "Goodbye");
```

With `default_locale = "fr"` in [`i18n!`] (or `EMBEDDED_I18N_LOCALE=fr` on the command line), the same code embeds `"Bonjour"` and `"Au revoir"` instead, and the English strings are not in the binary.

## Syntax

```text
t!(KEY [, option | argument]* [,])
```

| Form | Result | Guide |
| --- | --- | --- |
| `t!("key")` | `&'static str` | this page |
| `t!(c"key")` | `&'static CStr` | this page |
| `t!("key", name = value)`, `t!("key", "name" => value)` | `impl Display` | [Formatting][guide-formatting] |
| `t!("key", name = value : {:08})` | `impl Display` | [Formatting][guide-formatting] |
| `t!("key", count = n)` | `impl Display` | [Plural Forms][guide-plurals] |
| `t!("key", select = value)` | `&'static str` | [Select Variants][guide-select] |
| `t!("key", context = "menu")` | `&'static str` | [Message Context][guide-context] |
| `t!("key", locale = "fr")` | `&'static str` | [Runtime Locale Selection][guide-runtime-locales] |

The options `context`, `locale`, `count` and `select` may also precede the key (`t!(context = "menu", "Open")`).
Each option and argument name can appear once.

## Keys

The key is a string literal.
Escapes are unescaped like in any Rust string, so `t!("Say \"hi\"\n")` looks up the key `Say "hi"` followed by a newline.
Keys of nested locale files are written with dots (`t!("menu.settings.title")`), see [Locale Files][guide-locale-files].

## C Strings

A C string literal key returns a `&'static CStr`, to pass translations to C libraries (LVGL, display drivers...) without copying:

```rust
use core::ffi::CStr;
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

let greeting: &'static CStr = t!(c"c_greeting");
assert_eq!(greeting.to_bytes_with_nul(), b"Hello from C\0");
```

A translation containing a NUL byte cannot be a C string and is a compile error.
C string keys cannot take named arguments or a `count`.

## Lookup

The key is looked up in the locale of the build, then in each locale of the fallback chain (`fr-CA`, `fr`, then the `fallback` option, `en` by default), and the first non-empty translation wins.
See [Configuration][guide-configuration] for the chain and [Coverage][guide-coverage] to forbid falling back.

## Where `t!` Works

`t!` finds its configuration through the macro that [`i18n!`] defines in the crate, so:

- [`i18n!`] must come before the `mod` declarations using `t!`, as in rust-i18n;
- each crate, integration test, example and doctest that translates calls [`i18n!`] once, and a second call in the same crate is a compile error;
- forgetting it reports `cannot find macro embedded_i18n_requires_i18n_at_the_crate_root`.

## Compile Errors

Mistakes are reported at the call site, as regular compile errors:

```rust,compile_fail
# embedded_i18n::i18n!("locales");
let _ = embedded_i18n::t!("no such key");
```

```text
error: translation for "no such key" not found in locales "en" (/path/to/crate/locales)
```

- a key missing from every locale of the chain;
- a key that is not a string or C string literal:

  ```rust,compile_fail
  # embedded_i18n::i18n!("locales");
  let key = "greeting";
  let _ = embedded_i18n::t!(key);
  ```

- a translation with a NUL byte requested as a C string;
- a placeholder of a translation without an argument, see [Formatting][guide-formatting];
- every error of the locale files themselves (see [Locale Files][guide-locale-files]) and of the [`i18n!`] arguments.

[guide-locale-files]: locale_files.md
[guide-configuration]: configuration.md
[guide-coverage]: coverage.md
[guide-formatting]: formatting.md
[guide-plurals]: plurals.md
[guide-select]: select.md
[guide-context]: context.md
[guide-runtime-locales]: runtime_locales.md
[`t!`]: translating.md
[`i18n!`]: configuration.md
