# Pseudo-Localization

Before real translations exist, a pseudo-locale shows which strings are hardcoded, which labels are clipped by longer languages, and which glyphs are missing from the font.
Build with the special locale `pseudo`, with the default locale of [`i18n!`] or the `EMBEDDED_I18N_LOCALE` override:

```sh
EMBEDDED_I18N_LOCALE=pseudo cargo run
```

```rust
embedded_i18n::i18n!("locales", default_locale = "pseudo", fallback = "en");

let greeting = embedded_i18n::t!("greeting");
assert!(greeting.starts_with('[') && greeting.ends_with(']'));
```

Every message of the fallback locales is transformed at compile time:

```text
Open            →  [Ópéñ ~~]
Hello {name}    →  [Hélló {name} ~~]
{count}         →  [{count}]
```

- letters are replaced with accented Latin-1 letters, so text that is not translated stands out, and the font is checked for accented glyphs;
- the text is padded with `~` by about a third of its letters, the usual expansion of English text into German or French, to reveal clipping;
- the result is wrapped in brackets, so truncated text is easy to spot;
- placeholders and the `{{` / `}}` escapes are kept as is, so [formatting][guide-formatting], [plural][guide-plurals] and [select][guide-select] keep working.

The pseudo-locale is built from the fallback chain (`fallback`, `en` by default), keeps the plural rules of the locale each message comes from, and is always complete for [strict mode][guide-coverage].
It can be embedded next to real locales for [runtime selection][guide-runtime-locales]: `available_locales = ["en", "fr", "pseudo"]`.

[guide-formatting]: formatting.md
[guide-plurals]: plurals.md
[guide-select]: select.md
[guide-coverage]: coverage.md
[guide-runtime-locales]: runtime_locales.md
[`i18n!`]: configuration.md
