# Comparison With Other Crates

`embedded-i18n` resolves translations at compile time and embeds only the strings of the languages you build.
That fits firmware, where flash, RAM and the allocator are scarce and the set of languages is known before flashing.
Runtime libraries fit applications that load language packs at runtime or need the full expressiveness of Fluent.

| | embedded-i18n | [rust-i18n](https://crates.io/crates/rust-i18n) | [Fluent](https://crates.io/crates/fluent) | [gettext-rs](https://crates.io/crates/gettext-rs) |
| --- | --- | --- | --- | --- |
| `no_std`, no allocator | ✅ | ❌ | ❌ | ❌ |
| Translations resolved | At compile time, as literals | Embedded at compile time, looked up at runtime | Parsed and resolved at runtime | Loaded from `.mo` files at runtime |
| Missing translation | Compile error | Runtime fallback | Runtime fallback or error | Runtime fallback to the source string |
| Placeholders checked at compile time | ✅ | ❌ | ❌ | ❌ |
| Plural forms | CLDR rules, gettext `Plural-Forms` | Not built in | CLDR rules | gettext `Plural-Forms` |
| Message context / select variants | ✅ / ✅ | ❌ / ❌ | Message ids / ✅ | ✅ / ❌ |
| Runtime locale switching | Optional (`available_locales`), locales chosen at build time | ✅ | ✅ | ✅ |
| File formats | JSON, YAML, TOML, PO, MO, Fluent (subset) | YAML, JSON, TOML | Fluent | MO |
| Localized dates | Built in, without `libc` | ❌ | ❌ | ❌ |

The other crates evolve: check their documentation for the current state.

## Coming From rust-i18n

The API follows rust-i18n: `i18n!` and `t!` have the same names and most of the same syntax, and the YAML, JSON and TOML files can be reused with the matching backend features, one file per locale (`en.yml`, `fr.yml`).
The differences (returning `&'static str` or `Display` instead of `Cow<str>`, compile errors instead of runtime fallbacks, one embedded locale by default) are listed in [Migrating from rust-i18n][guide-migrating-rust-i18n].

Coming from an earlier version of this crate? See [Migrating from 0.1][guide-migrating-0-1].

## Coming From gettext

- `.po` and `.mo` files are read directly by the `backend-po` and `backend-mo` features, with `msgctxt`, plural forms, `Plural-Forms` headers and fuzzy entries handled like `msgfmt`.
- `gettext("text")` becomes [`t!`]`("text")`, `pgettext("ctx", "text")` becomes `t!("text", context = "ctx")`, and `ngettext("file", "files", n)` becomes `t!("file", count = n)`.
- Placeholders are named and use Rust format syntax (`{name}`) rather than `printf` (`%s`, `%d`).
- Source strings must be translated in at least one locale file (an English catalog, or the `fallback` option pointing to the source language), since keys are not used as their own translation.

## Coming From Fluent

- Simple messages, terms, attributes, variables and whole-message selects are supported (see [Locale Files][guide-locale-files]).
- Functions (`NUMBER()`, `DATETIME()`), parameterized terms and selects mixed with text are not, as they need a runtime resolver.
- `{ $user-name }` becomes the Rust placeholder `{user_name}`.

[guide-locale-files]: locale_files.md
[guide-migrating-rust-i18n]: migrating_from_rust_i18n.md
[guide-migrating-0-1]: migrating_from_0_1.md
[`t!`]: translating.md
