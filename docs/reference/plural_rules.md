# Plural Rules

`1 file`, `2 files`: English has two plural forms, but Polish and Russian have three or four, Arabic six, and Japanese one.
[`t!`]`("key", count = n)` picks the form matching `n` with the plural rules of the locale the message comes from (see [`count`][reference-t]).
The rules come from CLDR for JSON, YAML, TOML and Fluent files, and from the `Plural-Forms` header for PO and MO files.

## CLDR Categories

Forms are named after the [CLDR plural categories](https://cldr.unicode.org/index/cldr-spec/plural-rules): `zero`, `one`, `two`, `few`, `many` and `other`.
The layout of the forms in each file format is described in [Locale Files][reference-locale-files].

- Every category the language uses for integers is required, plus `other`.
- `zero` is optional in languages without a `zero` category: when present, it is used for a count of 0 (`"zero": "No files"`).
- The `many` form of French, Spanish, Italian, Portuguese and Catalan, used for millions (`1 000 000 de fichiers`), is optional and falls back to `other`.

## Built-In Rules

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
A plural message coming from a locale without built-in rules, in a JSON, YAML, TOML or Fluent file, is a compile error suggesting a gettext file.

The rules apply to integers: negative counts use their absolute value, and fractional counts are not supported.
The [pseudo-locale][how-to-pseudo-localize] keeps the rules of the locale each message comes from.

## gettext `Plural-Forms`

In PO and MO files, plural messages use `msgid_plural` and `msgstr[N]`, and the C expression of the `Plural-Forms` header is compiled to Rust, so any language works:

```po
msgid ""
msgstr ""
"Plural-Forms: nplurals=3; plural=(n%10==1 && n%100!=11 ? 0 : n%10>=2 && n%10<=4 && (n%100<10 || n%100>=20) ? 1 : 2);\n"
```

- The message is looked up by its `msgid`: `t!("file", count = n)`.
- Without a header, the gettext default `plural=(n != 1)` applies.
- An index past the last form uses the first one.
- As with `msgfmt`, a plural entry with an empty form is untranslated and resolves through the fallback locales.
- [`t!`] on a plural message without `count` returns its first form.
- The expression supports the C operators of gettext: arithmetic (`+ - * / %`, a division by zero giving 0), comparisons, `&&`, `||`, `!`, the ternary `? :` and parentheses, on the variable `n`.

## Compile Errors

| Mistake | Message |
| --- | --- |
| Missing required category | `plural translation for "files" in locale "ru" is missing the few, many forms` |
| Locale without built-in rules | `no plural rules for locale "xx", use a gettext PO or MO file with a Plural-Forms header` |
| `Plural-Forms` without `plural=` | `missing plural= in Plural-Forms "nplurals=2;"` |
| Invalid `Plural-Forms` expression | `invalid plural expression "..."`, `unexpected character '$' in plural expression "..."`, `number too large in "..."` |
| Placeholder of a form without an argument, such as `{folder}` without `folder = ...` | see [The `t!` Macro][reference-t] |

[reference-t]: t_macro.md
[reference-locale-files]: locale_files.md
[how-to-pseudo-localize]: ../how_to/pseudo_localize.md
[`t!`]: t_macro.md
