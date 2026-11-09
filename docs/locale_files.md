# Locale Files

Translations live in one file per locale, in a `locales/` directory next to the `Cargo.toml` of the crate calling [`i18n!`] (see [Configuration][guide-configuration] to change it): `en.json`, `fr-CA.po`, `de.yaml`...
The files are read and parsed by the procedural macros at compile time, so every format adds code to the build only, never to the firmware.

Keys are either source strings, gettext style (`"Hello, world!"`), or identifiers (`menu.settings.title`).
Both styles can be mixed.
Translations use `{name}` placeholders, or rust-i18n's `%{name}`, see [Formatting][guide-formatting].

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

Nested objects are flattened to dotted keys, so the last entry is `t!("menu.settings.title")`.
Values must be strings or objects.
A key defined twice, e.g. `"menu.settings.title"` and its nested form, is a compile error.

## YAML and TOML

YAML and TOML files are nested like JSON:

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
- multi-line strings and C escapes (`\n`, `\"`, `\t`...) are supported.

Ignored and untranslated entries resolve through the fallback locales.
`msgctxt` is the [message context][guide-context], and `msgid_plural` / `msgstr[N]` with the `Plural-Forms` header are the [plural forms][guide-plurals].

Compiled catalogs (`.mo`, little or big endian), as produced by `msgfmt`, are read by the `backend-mo` feature with the same rules.

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
- a select expression making up a whole message, which becomes one key per variant (`emails.one`, `emails.other`), with the default variant also available as `emails`. This is how [plural forms][guide-plurals] and [select variants][guide-select] are written in Fluent.

Select expressions mixed with text, functions (`NUMBER()`, `DATETIME()`) and parameterized terms are not supported and are reported as compile errors.

## Compile Errors

- a locale file that does not parse, with the parser message and the file path;
- a JSON, YAML or TOML value that is not a string or a nested table;
- a key defined twice in the same file;
- a missing locales directory, naming the resolved path.

[guide-configuration]: configuration.md
[guide-context]: context.md
[guide-plurals]: plurals.md
[guide-select]: select.md
[guide-formatting]: formatting.md
[`t!`]: translating.md
[`i18n!`]: configuration.md
