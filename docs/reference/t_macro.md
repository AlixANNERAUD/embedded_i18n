# The `t!` Macro

[`t!`] replaces a message key with its translation, at compile time.
It requires one call to [`i18n!`] at the crate root, before the `mod` declarations using [`t!`] (see [Configuration][reference-configuration]).

## Syntax

```text
t!(KEY [, ITEM]* [,])

KEY  = string literal | C string literal
ITEM = context = <string literal>
     | locale = <expression>
     | count = <expression>
     | select = <expression>
     | <name> = <expression> [: {<format spec>}]
     | "<name>" => <expression> [: {<format spec>}]
```

- The options `context`, `locale`, `count` and `select` may also precede the key: `t!(context = "menu", "Open")`.
- Named arguments must follow the key.
- Each option and argument name can appear once.
- Positional arguments are not supported, and a variable is not captured implicitly: write `name = name`.
- A raw identifier names the argument without its `r#` prefix: `r#type = value` fills `{type}`.

## Forms and Result Types

| Form | Result | Section |
| --- | --- | --- |
| `t!("key")` | `&'static str` | [Keys](#keys) |
| `t!(c"key")` | `&'static CStr` | [C Strings](#c-strings) |
| `t!("key", name = value)`, `t!("key", "name" => value)` | `impl Display` | [Arguments](#arguments) |
| `t!("key", name = value : {:08})` | `impl Display` | [Format Specifications](#format-specifications) |
| `t!("key", count = n)` | `impl Display` | [`count`](#count) |
| `t!("key", select = value)` | `&'static str` | [`select`](#select) |
| `t!("key", context = "menu")` | `&'static str` | [`context`](#context) |
| `t!("key", locale = "fr")` | `&'static str` | [`locale`](#locale) |

Without named arguments and without `count`, the result is a `&'static str` (a `&'static CStr` with a C string key).
`select`, `context` and `locale` keep this type.
With a single embedded locale, it is a literal, usable in `const` and `static` items:

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

const GREETING: &str = t!("greeting");
static FAREWELL: &str = t!("farewell");

assert_eq!(GREETING, "Hello");
assert_eq!(FAREWELL, "Goodbye");
```

With several embedded locales, it is a runtime value, which cannot initialize a `const` or `static` item; a literal `locale = "fr"` keeps it a literal.
Without arguments, the translation is returned as written in the file, placeholders included.

With named arguments or `count`, the result implements [`core::fmt::Display`] and formats on demand, without allocation:

- it works with `write!`, `format_args!` and every formatting macro, into any [`core::fmt::Write`] (a display driver, a fixed-size buffer, a UART, a `String`), or any writer with a `write_fmt` method, such as `std::io::Write`;
- with `alloc` or `std`, `.to_string()` and `format!("{}", ...)` return a `String`;
- formatting flags of the outer format string (width, alignment) are not applied to the whole message;
- `&*t!(..)` is not supported.

```rust
use core::fmt::Write;
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

let mut output = String::new();
write!(output, "{}", t!("with_arg", value = 42))?;
assert_eq!(output, "Value: 42");
# Ok::<(), core::fmt::Error>(())
```

## Keys

The key is a string literal.
Escapes are unescaped like in any Rust string, so `t!("Say \"hi\"\n")` looks up the key `Say "hi"` followed by a newline.
Keys of nested locale files are written with dots (`t!("menu.settings.title")`), see [Locale Files][reference-locale-files].

## C Strings

A C string literal key returns a `&'static CStr`, to pass translations to C libraries (LVGL, display drivers...) without copying:

```rust
use core::ffi::CStr;
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

let greeting: &'static CStr = t!(c"c_greeting");
assert_eq!(greeting.to_bytes_with_nul(), b"Hello from C\0");
```

- The key must be valid UTF-8.
- A translation containing a NUL byte cannot be a C string and is a compile error.
- C string keys cannot take named arguments or a `count`; they accept `context`, `select` and `locale`.

## Arguments

Arguments are named: `name = value`, or `"name" => value` as in rust-i18n.
Each fills the `{name}` placeholders of the translation.

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

let name = "Ada";
let count = 3;

assert_eq!(
    t!("with_named_args", name = name, count = count).to_string(),
    "Hello Ada, you have 3 messages",
);
assert_eq!(
    t!("with_named_args", "name" => "Grace", count = 1).to_string(),
    "Hello Grace, you have 1 messages",
);
```

- Arguments are evaluated once and borrowed, as by `format_args!`: temporaries live until the end of the enclosing statement.
- The result can be stored in a `let` when the arguments are places (variables, fields), not temporaries.
- An argument that a translation does not use is allowed, since a singular plural form may not use `{count}`.
- A placeholder without an argument is a compile error.

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

let name = String::from("Ada");
let count = 3;
let text = t!("with_named_args", name = name, count = count);
assert_eq!(text.to_string(), "Hello Ada, you have 3 messages");
```

## Format Specifications

A specification after an argument, `: {:spec}`, applies to every placeholder of that name, unless the placeholder has its own specification in the translation (`{value:>5}`):

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

assert_eq!(t!("with_arg", value = 42 : {:05}).to_string(), "Value: 00042");
assert_eq!(t!("with_arg", "value" => 7 : {:03}).to_string(), "Value: 007");
```

The specification uses the syntax of `format!` after the colon.

## Placeholders

- Translations use Rust format syntax, `{name}`, optionally with a specification, `{name:>5}`; literal braces are written `{{` and `}}`.
- The rust-i18n syntax `%{name}` is accepted too. A translation containing `%{` is read in rust-i18n syntax: `%{name}` is a placeholder and every other brace is literal.
- Placeholder names are Rust identifiers. Positional placeholders (`{}`, `{0}`) are compile errors.
- The macro parses the placeholders of every translation it may select: every embedded locale, plural form and select variant.

## `count`

`count = n` selects the plural form of the message matching `n`, with the plural rules of the locale the message comes from, and fills the `{count}` placeholder:

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

assert_eq!(t!("files", count = 1).to_string(), "1 file");
assert_eq!(t!("files", count = 5u32).to_string(), "5 files");
assert_eq!(t!("files_with_zero", count = 0).to_string(), "No files");
```

- The count can be any integer type. Negative counts use their absolute value. Fractional counts (`1.5 files`) are not supported.
- If the message has no plural forms, `count` is an ordinary named argument that only fills `{count}`.
- Other named arguments can follow: `t!("files", count = n, folder = name)`.
- A form that does not use an argument, such as a singular form without `{count}`, is allowed.
- `count` combines with `context`, and cannot be used with a C string key.
- Only the selection runs at runtime: the plural rule is compiled into an `if` / `match` on the count.

The categories, the built-in rules and gettext `Plural-Forms` are described in [Plural Rules][reference-plural-rules].

## `select`

`select = value` chooses a variant of the message from a value implementing `AsRef<str>`, such as a gender, a device state or an enum:

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

assert_eq!(t!("replied", select = "female"), "She replied");
assert_eq!(t!("replied", select = "male"), "He replied");
assert_eq!(t!("replied", select = "unknown"), "They replied");

enum Gender {
    Female,
    Male,
    Unspecified,
}

impl AsRef<str> for Gender {
    fn as_ref(&self) -> &str {
        match self {
            Gender::Female => "female",
            Gender::Male => "male",
            Gender::Unspecified => "other",
        }
    }
}

assert_eq!(t!("replied", select = Gender::Female), "She replied");
assert_eq!(t!("replied", select = Gender::Unspecified), "They replied");
```

- The expansion is a `match` on string literals returning a `&'static str` (or a `&'static CStr`), with no allocation and no table lookup.
- The `other` variant is required, and used for every value without a variant of its own.
- All variants come from the first locale of the chain defining a non-empty `other` variant for the message: variants are never mixed between locales.
- With named arguments next to `select`, the result implements `Display`.
- `select` combines with `context`.

The layout of variants in each file format is described in [Locale Files][reference-locale-files].

## `context`

`context = "menu"` selects the translation of the message in a context, to translate identical source strings differently (`Open` as an action is `Ouvrir` in French, as a state `Ouvert`):

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

assert_eq!(t!("Open", context = "menu"), "Open (menu)");
assert_eq!(t!(context = "state", "Open"), "Open (state)");
assert_eq!(t!("files", context = "menu", count = 2).to_string(), "Open files");
```

- The value is a string literal.
- The message is looked up as `menu.Open` in JSON, YAML, TOML and Fluent files, and with `msgctxt "menu"` in PO and MO files.
- A message with a context never falls back to the same key without context: if `menu.Open` is missing from every locale of the chain, the call is a compile error, even if `Open` exists.
- The result keeps the type it has without context. `context` works with C string keys, `count` and `select`.

## `locale`

`locale = ...` translates into the given locale instead of the current one, without changing the current one:

- a string literal is resolved at compile time: `t!("greeting", locale = "fr")` is the French literal;
- any other expression of a type implementing `Into<Locale>` (`&str`, [`Locale`]) is resolved at runtime, by the resolution of [`set_locale`];
- with a single embedded locale, the expression is evaluated and ignored.

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

assert_eq!(t!("greeting", locale = "en"), "Hello");
```

See [How to Localize per Request][how-to-localize-per-request].

## Lookup

The key is looked up in the locale being translated into, then in each locale of the fallback chain (`fr-CA`, `fr`, then the `fallback` option, `en` by default), and the first non-empty translation wins.
With `strict = true`, it must be found in the primary locale.
See [Configuration][reference-configuration] for the chain.

## `defmt`

`defmt` interns format strings at compile time and needs them as literal tokens, so translations cannot be `defmt` format strings.
Log the translation as an argument instead: `defmt::info!("{=str}", t!("greeting"))`.

## Compile Errors

Mistakes are reported at the call site.
In the messages below, the subject is `translation for "key"`, `plural translation for "key"` (with a plural `count`) or ``select translation for "key" with an `other` variant`` (with `select`), followed by `in context "menu"` when a context is given.

| Mistake | Message |
| --- | --- |
| No [`i18n!`] in the crate | ``cannot find macro `embedded_i18n_requires_i18n_at_the_crate_root` in this scope`` |
| Key missing from every locale of the chain | `translation for "no such key" not found in locales "en" (/path/to/crate/locales)` |
| With `strict = true`, key missing from the primary locale | `translation for "missing_in_french" is missing from the primary locale "fr" (strict mode)` |
| Key that is not a string or C string literal | `expected a message key (string or C string literal)` |
| Positional argument | ``positional arguments are not supported, name the argument: `name = value` `` |
| Named argument before the key | `arguments must follow the message key` |
| Option or argument given twice | `` `name` is given twice `` |
| `context` that is not a string literal | `` `context` expects a string literal `` |
| Format specification without `:` | ``a format specification starts with `:`, as in `{:08}` `` |
| Arguments or `count` with a C string key | `C strings cannot take arguments` |
| C string key that is not valid UTF-8 | `C string key must be valid UTF-8` |
| Translation with a NUL byte requested as a C string | `translation for "key" contains a nul byte` |
| Placeholder without an argument | `translation of "with_named_args" in locale "en" uses the placeholder {count}, which is not an argument of t!` |
| Unescaped brace | `translation of "key" in locale "en": unbalanced brace in "..."` |
| Positional placeholder | `translation of "key" in locale "en": positional placeholder {} in "...", use a named placeholder` |
| Invalid placeholder name | `translation of "key" in locale "en": invalid placeholder {...} in "..."` |
| Placeholder named `self`, `Self`, `super` or `crate` (other keywords such as `{type}` work) | `placeholder {self} in the translation of "key" (locale "en") is a Rust keyword` |
| Missing plural category | `plural translation for "files" in locale "ru" is missing the few, many forms` |
| Plural message from a locale without built-in rules | `no plural rules for locale "xx", use a gettext PO or MO file with a Plural-Forms header` |
| Select message without `other` variant | `` select translation for "replied" with an `other` variant not found in locales "en" (...) `` |

```rust,compile_fail
# embedded_i18n::i18n!("locales");
let _ = embedded_i18n::t!("no such key");
```

```rust,compile_fail
# embedded_i18n::i18n!("locales");
let key = "greeting";
let _ = embedded_i18n::t!(key);
```

```rust,compile_fail
# embedded_i18n::i18n!("locales");
// `with_named_args` also needs `count`.
let _ = embedded_i18n::t!("with_named_args", name = "Ada").to_string();
```

```rust,compile_fail
# embedded_i18n::i18n!("locales");
let _ = embedded_i18n::t!(c"greeting", name = 1);
```

The errors of the locale files are listed in [Locale Files][reference-locale-files], and those of the [`i18n!`] arguments in [Configuration][reference-configuration].

[reference-configuration]: configuration.md
[reference-locale-files]: locale_files.md
[reference-plural-rules]: plural_rules.md
[how-to-localize-per-request]: ../how_to/localize_per_request.md
[`t!`]: t_macro.md
[`i18n!`]: configuration.md
[`Locale`]: runtime_api.md
[`set_locale`]: runtime_api.md
[`core::fmt::Display`]: https://doc.rust-lang.org/core/fmt/trait.Display.html
[`core::fmt::Write`]: https://doc.rust-lang.org/core/fmt/trait.Write.html
