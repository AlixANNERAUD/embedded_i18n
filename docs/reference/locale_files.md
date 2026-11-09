# Locale Files

Translations live in one file per locale, in the locale files directory of the crate calling [`i18n!`] (`locales/` by default, see [Configuration][reference-configuration]): `en.json`, `fr-CA.po`, `de.yaml`...
The files are read and parsed by the procedural macros at compile time, so every format adds code to the build only, never to the firmware.

Keys are either source strings, gettext style (`"Hello, world!"`), or identifiers (`menu.settings.title`).
Both styles can be mixed.
Translations use `{name}` placeholders, or rust-i18n's `%{name}`, see [The `t!` Macro][reference-t].

## Formats and Cargo Features

Each format is enabled by a Cargo feature of `embedded-i18n`:

| Feature | Files | Parser |
| --- | --- | --- |
| `backend-json` (default) | `.json` | `serde_json` |
| `backend-yaml` | `.yaml`, `.yml` | `yaml-rust2` |
| `backend-toml` | `.toml` | `toml` |
| `backend-po` | GNU gettext `.po` | built in |
| `backend-mo` | Compiled GNU gettext `.mo` | built in |
| `backend-fluent` | Fluent `.ftl` (subset) | `fluent-syntax` |

To use another format than JSON, disable the default features:

```toml
[dependencies]
embedded-i18n = { version = "0.2", default-features = false, features = ["backend-po"] }
```

Several backends can be enabled together.
For each locale, the first existing file wins, in this order: `.json`, `.yaml`, `.yml`, `.toml`, `.po`, `.mo`, `.ftl`.
Files of the same locale are not merged.
Files containing several locales at once are not supported.

## File Names

A locale file is named after its locale, with `-` or `_`, in the normalized case of the locale or in lower case: `fr-CA.json`, `fr_CA.json`, `fr-ca.json` or `fr_ca.json`.
Missing files are skipped: the message is looked up in the next locale of the chain.

## JSON

```json
{
  "Hello, world!": "Bonjour le monde !",
  "Missing argument: {name}": "Argument manquant : {name}",
  "menu": {
    "settings": {
      "title": "Paramètres"
    }
  }
}
```

- The top level is an object.
- Nested objects are flattened to dotted keys, so the last entry is `t!("menu.settings.title")`.
- Values are strings or objects.
- A key defined twice, e.g. `"menu.settings.title"` and its nested form, is a compile error.

## YAML and TOML

YAML and TOML files are nested like JSON, with the same rules (a mapping or table at the top level, string values, no duplicate flattened keys):

```yaml
greeting: Hallo
menu:
  settings:
    title: Einstellungen
```

```toml
greeting = "Hola"

[menu.settings]
title = "Ajustes"
```

YAML integer keys are read as strings.

## GNU gettext PO and MO

```po
msgid ""
msgstr ""
"Plural-Forms: nplurals=2; plural=(n > 1);\n"

msgid "Hello, world!"
msgstr "Bonjour le monde !"

msgctxt "menu"
msgid "Open"
msgstr "Ouvrir"
```

The parser follows `msgfmt`:

- entries flagged `#, fuzzy` (alone or with other flags, such as `#, fuzzy, c-format`) are ignored;
- entries with an empty `msgstr`, and plural entries with an empty form, are untranslated;
- obsolete entries (`#~`) are ignored;
- multi-line strings and C escapes (`\n`, `\"`, `\t`...) are supported;
- the indexes of `msgstr[N]` must be contiguous from 0.

Ignored and untranslated entries resolve through the fallback locales.
The header entry (empty `msgid`) provides `Plural-Forms`, see [Plural Rules][reference-plural-rules].

Compiled catalogs (`.mo`, little or big endian, major revision 0 or 1), as produced by `msgfmt`, are read by the `backend-mo` feature with the same rules.

## Fluent

```ftl
-brand = Acme
hello = Hello, { $user-name }!
about = About { -brand }
login = Log in
    .placeholder = Email
emails = { $count ->
    [one] One email
   *[other] { $count } emails
}
```

The supported subset covers simple messages:

- text, message references and term references (`{ -brand }` becomes `Acme`);
- string and number literals;
- attributes, as `<message>.<attribute>` keys (`login.placeholder`);
- variables, which become named format placeholders: `{ $user-name }` becomes `{user_name}`, for the named arguments of [`t!`];
- a select expression making up a whole message, which becomes one key per variant (`emails.one`, `emails.other`), with the default variant also available as `emails`.

Select expressions mixed with text, functions (`NUMBER()`, `DATETIME()`) and parameterized terms are not supported and are reported as compile errors.

## Plural Forms, Select Variants and Contexts

| | JSON, YAML, TOML | PO, MO | Fluent |
| --- | --- | --- | --- |
| Plural forms of `files` | nested keys named after the CLDR categories: `files.one`, `files.other` | `msgid "files"`, `msgid_plural`, `msgstr[N]` | a whole-message select on `$count`, with CLDR category names |
| Select variants of `replied` | nested keys: `replied.female`, `replied.other` | flat keys: `msgid "replied.female"`, `msgid "replied.other"` | a whole-message select, with the variant names |
| `Open` in context `menu` | the key `menu.Open` (a `menu` namespace) | `msgctxt "menu"` | the key `menu.Open` |

Plural forms in JSON, YAML and Fluent:

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

```ftl
files = { $count ->
    [one] { $count } file
   *[other] { $count } files
}
```

Plural forms in PO files, with the rule of the `Plural-Forms` header:

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

Select variants, with the mandatory `other`:

```json
{
  "replied": {
    "male": "Il a répondu",
    "female": "Elle a répondu",
    "other": "Iel a répondu"
  }
}
```

```po
msgid "replied.female"
msgstr "Elle a répondu"

msgid "replied.other"
msgstr "Iel a répondu"
```

```ftl
replied = { $gender ->
    [male] He replied
    [female] She replied
   *[other] They replied
}
```

Contexts:

```json
{
  "menu": { "Open": "Ouvrir" },
  "state": { "Open": "Ouvert" }
}
```

```po
msgctxt "menu"
msgid "Open"
msgstr "Ouvrir"

msgctxt "state"
msgid "Open"
msgstr "Ouvert"
```

## Compile Errors

Errors of a locale file are prefixed with its path, and reported on the [`i18n!`] call and on the [`t!`] calls reading it.

| Mistake | Message |
| --- | --- |
| Missing locales directory | `locales directory not found: /path/to/crate/locales` |
| Unreadable file | `/path/to/en.json: Failed to read file: ...` |
| JSON, YAML or TOML syntax error | `/path/to/en.json: JSON parse error: ...`, `YAML parse error: ...`, `TOML parse error: ...` |
| Top level that is not an object | `JSON parse error: expected an object at the top level`, `YAML parse error: expected a mapping at the top level` |
| Value that is not a string or a nested table | `JSON parse error: expected a string or an object for key "menu"`, `YAML parse error: expected a string or a mapping for key "menu"`, `TOML parse error: expected a string or a table for key "menu"` |
| YAML key that is not a string | `YAML parse error: expected a string key under "menu"` |
| Key defined twice | `duplicate key "menu.settings.title"` |
| PO syntax error | `PO parse error at line 12: ...` |
| Non-contiguous `msgstr[N]` | `PO parse error: msgstr indexes of "files" are not contiguous` |
| Invalid MO file | `MO parse error: invalid magic number`, `MO parse error: unsupported revision` |
| Fluent syntax error | `Fluent parse error: ...` |
| Unsupported Fluent construct | `Fluent error: select expression mixed with other text in "key" is not supported` |
| Unknown Fluent reference | `Fluent error: unknown message "name" in "key"`, `Fluent error: unknown term "name" in "key"` |
| Fluent reference cycle | `Fluent error: reference cycle in "key"` |
| Invalid `Plural-Forms` | see [Plural Rules][reference-plural-rules] |

[reference-configuration]: configuration.md
[reference-t]: t_macro.md
[reference-plural-rules]: plural_rules.md
[`t!`]: t_macro.md
[`i18n!`]: configuration.md
