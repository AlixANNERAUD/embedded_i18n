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

## rust-i18n

The API of `embedded-i18n` follows rust-i18n: `i18n!` and `t!` have the same names and most of the same syntax, and the YAML, JSON and TOML files can be reused with the matching backend features, one file per locale (`en.yml`, `fr.yml`).
The differences come from where the work happens: rust-i18n looks translations up at runtime and returns `Cow<str>`, while `embedded-i18n` resolves them at compile time, returns `&'static str` or `Display` values, reports missing translations as compile errors, and embeds one locale by default.
See [Design Choices][explanation-design-choices] for the reasons, and [How to Migrate From rust-i18n][how-to-migrate-rust-i18n] to move a crate.

Coming from an earlier version of this crate? See [How to Migrate From 0.1][how-to-migrate-0-1].

## gettext

`embedded-i18n` reads `.po` and `.mo` files directly, with `msgctxt`, plural forms, `Plural-Forms` headers and fuzzy entries handled like `msgfmt`, so the usual gettext tools and translation services keep working.
The differences: placeholders are named and use Rust format syntax (`{name}`) rather than `printf` (`%s`, `%d`), and a `msgid` is not used as its own translation.
See [How to Use gettext PO Files][how-to-use-po-files] for the mapping of `gettext`, `pgettext` and `ngettext`.

## Fluent

`embedded-i18n` reads a subset of Fluent: simple messages, terms, attributes, variables and whole-message selects.
Functions (`NUMBER()`, `DATETIME()`), parameterized terms and selects mixed with text need a runtime resolver and are not supported.
Fluent variables become Rust placeholders: `{ $user-name }` becomes `{user_name}`.
See [Locale Files][reference-locale-files].

[explanation-design-choices]: design_choices.md
[how-to-migrate-rust-i18n]: ../how_to/migrate_from_rust_i18n.md
[how-to-migrate-0-1]: ../how_to/migrate_from_0_1.md
[how-to-use-po-files]: ../how_to/use_gettext_po_files.md
[reference-locale-files]: ../reference/locale_files.md
