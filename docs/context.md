# Message Context

The same source string can need different translations: `Open` as a menu action is `Ouvrir` in French, but `Open` as a state is `Ouvert`.
A message context disambiguates them.
Pass `context = "..."` to [`t!`]:

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

assert_eq!(t!("Open", context = "menu"), "Open (menu)");
assert_eq!(t!("Open", context = "state"), "Open (state)");
```

The option can also come first, as in `t!(context = "menu", "Open")`.
The result is still a `&'static str`.
`context` works with C string keys, and combines with [plural forms][guide-plurals]:

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

assert_eq!(t!("files", context = "menu", count = 2).to_string(), "Open files");
```

## In Locale Files

GNU gettext PO and MO files use `msgctxt`:

```po
msgctxt "menu"
msgid "Open"
msgstr "Ouvrir"

msgctxt "state"
msgid "Open"
msgstr "Ouvert"
```

JSON, YAML, TOML and Fluent files use a namespace named after the context, i.e. the key `menu.Open`:

```json
{
  "menu": { "Open": "Ouvrir" },
  "state": { "Open": "Ouvert" }
}
```

A message with a context never falls back to the same key without context: if `menu.Open` is missing from every locale of the chain, `t!("Open", context = "menu")` is a compile error, even if `Open` exists.

[guide-plurals]: plurals.md
[`t!`]: translating.md
