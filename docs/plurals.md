# Plural Forms

`1 file`, `2 files`: English has two plural forms, but Polish and Russian have three or four, Arabic six, and Japanese one.
Pass `count = n` to [`t!`] to pick the form matching a runtime count with the plural rules of the locale the message comes from:

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

assert_eq!(t!("files", count = 1).to_string(), "1 file");
assert_eq!(t!("files", count = 5u32).to_string(), "5 files");
assert_eq!(t!("files_with_zero", count = 0).to_string(), "No files");
```

Only the selection runs at runtime: the rule is compiled into an `if` / `match` on the count.
The result implements [`core::fmt::Display`] and formats on demand, so forms can use the `{count}` placeholder without allocating.
Write it into any [`core::fmt::Write`] (a display driver, a fixed-size buffer, a `String`):

```rust
use core::fmt::Write;
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

let mut output = String::new();
write!(output, "{}", t!("files", count = 3))?;
assert_eq!(output, "3 files");
# Ok::<(), core::fmt::Error>(())
```

The count can be any integer type.
Counts are integers only: negative counts use their absolute value, and fractional counts (`1.5 files`) are not supported.

`count` is also an ordinary named argument: if the message has no plural forms, `t!("key", count = n)` only fills `{count}`.
Other named arguments can follow the count (`t!("files", count = n, folder = name)`).
A form that does not use an argument, such as a singular form without `{count}`, is fine, but a placeholder without an argument is a compile error.
See [Formatting][guide-formatting].
Messages accept a [`context`][guide-context], and a C string key cannot be used with `count`.

## JSON, YAML, TOML and Fluent

Forms are nested keys named after the [CLDR plural categories](https://cldr.unicode.org/index/cldr-spec/plural-rules): `zero`, `one`, `two`, `few`, `many` and `other`.

```json
{
  "files": {
    "one": "{count} fichier",
    "other": "{count} fichiers"
  }
}
```

```yaml
files:
  one: "{count} файл"
  few: "{count} файла"
  many: "{count} файлов"
  other: "{count} файла"
```

In Fluent, a select expression making up the whole message:

```ftl
files = { $count ->
    [one] { $count } file
   *[other] { $count } files
}
```

Every category the language uses for integers is required, plus `other`, and a missing one is a compile error naming it:

```text
error: plural translation for "files" in locale "ru" is missing the few, many forms
```

- `zero` is optional in languages without a `zero` category: when present, it is used for a count of 0 (`"zero": "No files"`).
- The `many` form of French, Spanish, Italian, Portuguese and Catalan, used for millions (`1 000 000 de fichiers`), is optional and falls back to `other`.

Integer plural rules are built in for these languages:

| Rule | Languages |
| --- | --- |
| `other` only | `id`, `ja`, `km`, `ko`, `lo`, `ms`, `my`, `th`, `vi`, `zh` |
| `one` (1), `other` | `bg`, `da`, `de`, `el`, `en`, `et`, `fi`, `hu`, `nb`, `nl`, `nn`, `no`, `sv`, `tr` |
| `one` (0, 1), `other` | `bn`, `hi` |
| `one` (1), `many` (millions), `other` | `ca`, `es`, `it`, `pt-PT` |
| `one` (0, 1), `many` (millions), `other` | `fr`, `pt` |
| `one`, `few`, `many`, `other` | `be`, `pl`, `ru`, `uk` |
| `one`, `few`, `other` | `cs`, `ro`, `sk` |
| `one`, `two`, `other` | `he` |
| `zero`, `one`, `two`, `few`, `many`, `other` | `ar` |

Region subtags use the rules of their language (`fr-CA` uses `fr`), except `pt-PT`.
A message coming from a locale without built-in rules is a compile error suggesting a gettext file, whose rule is read from the file itself.

## GNU gettext PO and MO

Plural messages use `msgid_plural` and `msgstr[N]`, and the C expression of the `Plural-Forms` header is compiled to Rust, so any language works:

```po
msgid ""
msgstr ""
"Plural-Forms: nplurals=3; plural=(n%10==1 && n%100!=11 ? 0 : n%10>=2 && n%10<=4 && (n%100<10 || n%100>=20) ? 1 : 2);\n"

msgid "file"
msgid_plural "files"
msgstr[0] "{count} файл"
msgstr[1] "{count} файла"
msgstr[2] "{count} файлов"
```

The message is looked up by its `msgid` (`t!("file", count = n)`).
Without a header, the gettext default `plural=(n != 1)` applies, and an index past the last form uses the first one.
As with `msgfmt`, a plural entry with an empty form is untranslated and resolves through the fallback locales.
`t!` on a plural message without `count` returns its first form.

## Compile Errors

- a message without plural forms in any locale of the chain;
- a missing required category;
- a message coming from a locale without built-in rules, in a JSON, YAML, TOML or Fluent file;
- an invalid `Plural-Forms` expression;
- a placeholder of a form without an argument, such as `{folder}` without `folder = ...`.

[guide-context]: context.md
[guide-formatting]: formatting.md
[`t!`]: translating.md
[`core::fmt::Display`]: https://doc.rust-lang.org/core/fmt/trait.Display.html
[`core::fmt::Write`]: https://doc.rust-lang.org/core/fmt/trait.Write.html
