# Translation Coverage

By default, a message missing from the selected locale silently resolves through the fallback locales.
That is what you want while translations are in progress, but it can ship a French firmware with English strings.

## Strict Mode

Set `strict = true` to forbid falling back, in the arguments of [`i18n!`] or in the `[package.metadata.i18n]` table of `Cargo.toml` (see [Configuration][guide-configuration]):

```toml
[package.metadata.i18n]
default-locale = "fr"
strict = true
```

Every [`t!`] call must then find its message in the primary locale, i.e. the default locale and its less specific forms (`fr-CA` and `fr` for `default_locale = "fr-CA"`), otherwise the build fails at the call site:

```rust,compile_fail
// `missing_in_french` only exists in `locales/en.json`.
embedded_i18n::i18n!("locales", default_locale = "fr", strict = true);
let _ = embedded_i18n::t!("missing_in_french");
```

```text
error: translation for "missing_in_french" is missing from the primary locale "fr"
```

Every untranslated call site is reported in one build, not only the first one.
Entries ignored by the parsers count as missing: empty translations, fuzzy PO entries, plural messages with an empty form.
With [several embedded locales][guide-runtime-locales], every embedded locale must be complete.
The [pseudo-locale][guide-pseudo-localization] is always complete.

A CI job building each shipped locale in strict mode catches untranslated strings before release.
`EMBEDDED_I18N_LOCALE` overrides `default_locale` for the build, so one job can cover every language:

```yaml
strategy:
  matrix:
    locale: [en, fr, de]
steps:
  - run: cargo build --release
    env:
      EMBEDDED_I18N_LOCALE: ${{ matrix.locale }}
```

with `strict = true` in the metadata of the crate.

## Key Extraction and Unused Keys

There is no built-in extraction of the keys used in the source code (an `xgettext` equivalent), nor a report of keys present in locale files but never used.
A procedural macro only sees its own call site, cannot know when the last call site of a crate has been expanded, and cannot emit warnings on stable Rust, so these reports would need a separate tool scanning the sources.
Strict mode covers the most harmful case, untranslated strings, at compile time.

[guide-configuration]: configuration.md
[guide-runtime-locales]: runtime_locales.md
[guide-pseudo-localization]: pseudo_localization.md
[`i18n!`]: configuration.md
[`t!`]: translating.md
