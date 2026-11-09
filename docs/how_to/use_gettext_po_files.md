# How to Use gettext PO Files and Other Formats

JSON is the default locale file format.
This guide switches a crate to GNU gettext `.po` files, which most translation tools and services understand, and then to compiled `.mo`, YAML, TOML and Fluent files.

## Enable the Backend

Each format is a Cargo feature of `embedded-i18n`.
Disable the default features to drop JSON, and enable `backend-po`:

```toml
[dependencies]
embedded-i18n = { version = "0.2", default-features = false, features = ["backend-po"] }
```

The backends only run inside the procedural macros, on the build machine: they add nothing to the firmware.
Several backends can be enabled together; for each locale, the first existing file wins (`.json`, `.yaml`, `.yml`, `.toml`, `.po`, `.mo`, then `.ftl`).

## Write the Catalogs

Put one catalog per locale in `locales/`, named after the locale: `en.po`, `fr.po`, `fr-CA.po`...

```po
msgid ""
msgstr ""
"Plural-Forms: nplurals=2; plural=(n > 1);\n"

msgid "greeting"
msgstr "Bonjour"

msgid "with_named_args"
msgstr "Bonjour {name}, vous avez {count} messages"

msgctxt "menu"
msgid "Open"
msgstr "Ouvrir"

msgid "files"
msgid_plural "files"
msgstr[0] "{count} fichier"
msgstr[1] "{count} fichiers"

msgid "replied.female"
msgstr "Elle a répondu"

msgid "replied.other"
msgstr "Iel a répondu"
```

Fuzzy, obsolete and empty entries are skipped and resolve through the fallback locales, as with `msgfmt`.
Placeholders are named and use Rust format syntax, `{name}`, rather than `printf` conversions such as `%s` or `%d`.

## Translate

The gettext functions map to [`t!`] options:

| gettext | embedded-i18n |
| --- | --- |
| `gettext("text")` | `t!("text")` |
| `pgettext("ctx", "text")` | `t!("text", context = "ctx")` |
| `ngettext("file", "files", n)` | `t!("file", count = n)`, looked up by the `msgid` |
| `npgettext("ctx", "file", "files", n)` | `t!("file", context = "ctx", count = n)` |

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales", default_locale = "fr");

assert_eq!(t!("greeting"), "Bonjour");
assert_eq!(t!("Open", context = "menu"), "Ouvrir");
assert_eq!(t!("files", count = 2).to_string(), "2 fichiers");
assert_eq!(t!("replied", select = "female"), "Elle a répondu");
```

The plural rule comes from the `Plural-Forms` header and is compiled into Rust, so any language works, including those without built-in rules.

Source strings must be translated in at least one catalog, such as an English `en.po`, or the `fallback` option must point to a catalog of the source language: a `msgid` is never used as its own translation.

## Use Compiled MO Files

To ship the catalogs your tools compile, enable `backend-mo` and put the `.mo` files, as produced by `msgfmt`, in `locales/`:

```sh
msgfmt locales/fr.po -o locales/fr.mo
```

```toml
[dependencies]
embedded-i18n = { version = "0.2", default-features = false, features = ["backend-mo"] }
```

They are read with the same rules as `.po` files, in either byte order.

## Use YAML or TOML Files

Enable `backend-yaml` or `backend-toml`, and nest keys as in JSON:

```yaml
greeting: Hallo
menu:
  settings:
    title: Einstellungen
```

The rust-i18n files written one locale per file (`en.yml`, `fr.yml`) work as is.

## Use Fluent Files

Enable `backend-fluent`.
Simple messages, terms, attributes, variables and whole-message selects are supported, and `{ $user-name }` becomes the `{user_name}` placeholder:

```ftl
-brand = Acme
hello = Hello, { $user-name }!
about = About { -brand }
```

```rust,ignore
t!("hello", user_name = name)
```

Functions such as `NUMBER()` and `DATETIME()`, parameterized terms, and selects mixed with text are reported as compile errors, since they need a runtime resolver.

## See Also

- [Locale Files][reference-locale-files] for each format, the key layout and the errors.
- [Plural Rules][reference-plural-rules] for `Plural-Forms` and the built-in rules.

[reference-locale-files]: ../reference/locale_files.md
[reference-plural-rules]: ../reference/plural_rules.md
[`t!`]: ../reference/t_macro.md
