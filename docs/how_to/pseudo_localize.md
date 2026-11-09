# How to Pseudo-Localize

Before real translations exist, a pseudo-locale shows which strings are hardcoded, which labels are clipped by longer languages, and which glyphs are missing from the font.
This guide builds a firmware in the special locale `pseudo`.

## Build in the Pseudo-Locale

Use the `EMBEDDED_I18N_LOCALE` override, without changing the code:

```sh
EMBEDDED_I18N_LOCALE=pseudo cargo run
```

A crate that lists its `available_locales` without `pseudo` ignores this override and keeps its own default locale, so only crates with a single embedded locale, or with `pseudo` in their list, are pseudo-localized.

or set `pseudo` as the default locale of [`i18n!`]:

```rust
embedded_i18n::i18n!("locales", default_locale = "pseudo", fallback = "en");

let greeting = embedded_i18n::t!("greeting");
assert!(greeting.starts_with('[') && greeting.ends_with(']'));
```

Every message of the fallback locales (`fallback`, `en` by default) is transformed at compile time:

```text
Open            →  [Ópéñ ~~]
Hello {name}    →  [Hélló {name} ~~]
{count}         →  [{count}]
```

## Read the Result

- Letters are replaced with accented Latin-1 letters: text that stays plain on screen is not translated, and missing accented glyphs show the font needs more characters.
- The text is padded with `~` by about a third of its letters, the usual expansion of English text into German or French: a label that loses its `~` or its closing bracket is clipped.
- The result is wrapped in brackets, so truncated text is easy to spot.

Placeholders and the `{{` / `}}` escapes are kept as is, so arguments, plural forms and select variants keep working.
Each message keeps the plural rules of the locale it comes from.

## Ship It Next to Real Locales

The pseudo-locale can be embedded next to real locales for [runtime selection][how-to-switch-language], for instance behind a hidden menu entry:

```rust,standalone_crate
embedded_i18n::i18n!("locales", available_locales = ["en", "fr", "pseudo"]);

fn main() {
    embedded_i18n::set_locale("pseudo");
    assert!(embedded_i18n::t!("greeting").starts_with('['));
}
```

The pseudo-locale is always complete, so it passes [strict mode][how-to-ship-complete].

[how-to-switch-language]: switch_language_at_runtime.md
[how-to-ship-complete]: ship_complete_translations.md
[`i18n!`]: ../reference/configuration.md
