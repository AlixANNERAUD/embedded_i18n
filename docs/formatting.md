# Formatting

Translations often contain placeholders: `"Hello {name}, you have {count} messages"`.
Pass named arguments to [`t!`] to fill them.
The placeholders are checked against the arguments at compile time, in every language you build, and formatting needs no allocation.

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

let text = t!("with_named_args", name = "Ada", count = 3).to_string();
assert_eq!(text, "Hello Ada, you have 3 messages");
```

With arguments, `t!` returns a value implementing [`core::fmt::Display`], which formats on demand.
Without arguments, it returns the `&'static str` as written in the file, placeholders included.

## Writing Into a Buffer

Use the result with `write!`, `format_args!` or any other formatting macro.
It works with any [`core::fmt::Write`]: a display driver, a fixed-size buffer, a UART, or a `String`:

```rust
use core::fmt::Write;
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

let mut output = String::new();
write!(output, "{}", t!("with_arg", value = 42))?;
assert_eq!(output, "Value: 42");
# Ok::<(), core::fmt::Error>(())
```

Like `write!`, the writer only needs a `write_fmt` method, so `std::io::Write` works too.
With `alloc` or `std`, `.to_string()` and `format!("{}", ...)` return a `String`.

## Arguments

Arguments are named: `name = value`, or `"name" => value` like in rust-i18n.
Positional arguments are not supported, and a variable is not captured implicitly: write `name = name`.

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

Arguments are evaluated once and borrowed, as by `format_args!`.
The result can be stored in a `let` when the arguments are places (variables, fields), and temporaries live until the end of the enclosing statement:

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

let name = String::from("Ada");
let count = 3;
let text = t!("with_named_args", name = name, count = count);
assert_eq!(text.to_string(), "Hello Ada, you have 3 messages");
```

An argument not used by a translation is allowed, as a singular plural form without `{count}` does not use `count`.
`count` is a reserved option for [plural forms][guide-plurals] and also fills the `{count}` placeholder.
Formatting flags of the outer format string (width, alignment) are not applied to the whole message.

## Format Specifications

A specification after the argument applies to every placeholder of that name, unless the placeholder has its own:

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

assert_eq!(t!("with_arg", value = 42 : {:05}).to_string(), "Value: 00042");
```

## Placeholder Syntax

Translations use Rust format syntax, `{name}`, with literal braces written `{{` and `}}`.
The rust-i18n syntax `%{name}` is accepted too.
A translation containing `%{` is read in rust-i18n syntax: `%{name}` is a placeholder and other braces are literal.

The macro parses the placeholders of every translation it may select: every locale, plural form and select variant.

## `defmt`

`defmt` interns format strings at compile time and needs them as literal tokens, so translations cannot be `defmt` format strings.
Log the translation as an argument instead: `defmt::info!("{=str}", t!("greeting"))`.

## Compile Errors

The checks apply to the translation of every locale you build:

- a placeholder without an argument, e.g. a translator writing `{nom}` instead of `{name}`, reported with the key, the locale and the placeholder;
- invalid placeholder syntax, such as an unescaped `{` (write `{{`);
- a positional placeholder (`{}`), which has no name to match;
- a duplicated argument or option.

```rust,compile_fail
# embedded_i18n::i18n!("locales");
// `with_named_args` also needs `count`.
let _ = embedded_i18n::t!("with_named_args", name = "Ada").to_string();
```

Building each shipped locale in CI catches these mistakes before release.

[guide-plurals]: plurals.md
[`t!`]: translating.md
[`core::fmt::Write`]: https://doc.rust-lang.org/core/fmt/trait.Write.html
[`core::fmt::Display`]: https://doc.rust-lang.org/core/fmt/trait.Display.html
