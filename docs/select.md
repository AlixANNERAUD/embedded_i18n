# Select Variants

Some messages depend on a runtime value other than a count: a gender, a device state, an enum variant.
Pass `select = value` to [`t!`] to choose between the variants of a message from any value implementing `AsRef<str>`:

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

assert_eq!(t!("replied", select = "female"), "She replied");
assert_eq!(t!("replied", select = "male"), "He replied");
assert_eq!(t!("replied", select = "unknown"), "They replied");
```

The expansion is a `match` on string literals returning a `&'static str` (or a `&'static CStr` with a C string key), with no allocation and no table lookup.
The `other` variant is required and used for every value without a variant of its own.

Selecting from an enum only needs a string for each variant:

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

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

`select` keeps the `&'static str` type, so the result can be written into any [`core::fmt::Write`] as is:

```rust
use core::fmt::Write;
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

let mut output = String::new();
output.write_str(t!("replied", select = "male"))?;
assert_eq!(output, "He replied");
# Ok::<(), core::fmt::Error>(())
```

Variants can use [named arguments][guide-formatting] next to `select`, in which case the result implements `Display`.
Messages accept a [`context`][guide-context].

## In Locale Files

Variants are nested keys, with a mandatory `other`:

```json
{
  "replied": {
    "male": "Il a répondu",
    "female": "Elle a répondu",
    "other": "Iel a répondu"
  }
}
```

In PO and MO files, variants are flat keys:

```po
msgid "replied.female"
msgstr "Elle a répondu"

msgid "replied.other"
msgstr "Iel a répondu"
```

In Fluent, a select expression making up the whole message:

```ftl
replied = { $gender ->
    [male] He replied
    [female] She replied
   *[other] They replied
}
```

All variants come from the first locale of the chain defining a non-empty `other` variant for the message: variants are never mixed between locales.

## Compile Errors

- a message without an `other` variant in any locale of the chain;
- a placeholder of a variant without an argument.

[guide-context]: context.md
[guide-formatting]: formatting.md
[`t!`]: translating.md
[`core::fmt::Write`]: https://doc.rust-lang.org/core/fmt/trait.Write.html
