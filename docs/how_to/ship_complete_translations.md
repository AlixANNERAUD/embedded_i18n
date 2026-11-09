# How to Ship Complete Translations

By default, a message missing from the selected locale silently resolves through the fallback locales.
That is what you want while translations are in progress, but it can ship a French firmware with English strings.
This guide makes such a build fail instead.

## Turn On Strict Mode

Set `strict = true` in the `[package.metadata.i18n]` table of `Cargo.toml`, or in the arguments of [`i18n!`]:

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
error: translation for "missing_in_french" is missing from the primary locale "fr" (strict mode)
```

Every untranslated call site is reported in one build, not only the first one.
Entries ignored by the parsers count as missing: empty translations, fuzzy PO entries, plural messages with an empty form.

With [several embedded locales][how-to-switch-language], every embedded locale must be complete.
The [pseudo-locale][how-to-pseudo-localize] is always complete.

## Check Every Shipped Language in CI

Keep `strict = true` in the metadata of the crate, and build each shipped locale with the `EMBEDDED_I18N_LOCALE` override, so that one job covers every language:

```yaml
strategy:
  matrix:
    locale: [en, fr, de]
steps:
  - run: cargo build --release
    env:
      EMBEDDED_I18N_LOCALE: ${{ matrix.locale }}
```

The same builds catch placeholders that do not match their arguments, in every language.
See [How to Build One Firmware per Language][how-to-build-per-language].

## What Strict Mode Does Not Cover

Strict mode does not list the keys of the locale files that no call uses, nor extract the keys used in the sources (an `xgettext` equivalent).
See [Design Choices][explanation-design-choices] for why.

[how-to-switch-language]: switch_language_at_runtime.md
[how-to-pseudo-localize]: pseudo_localize.md
[how-to-build-per-language]: build_one_firmware_per_language.md
[explanation-design-choices]: ../explanation/design_choices.md
[`i18n!`]: ../reference/configuration.md
[`t!`]: ../reference/t_macro.md
