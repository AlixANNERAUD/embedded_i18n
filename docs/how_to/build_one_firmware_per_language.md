# How to Build One Firmware per Language

A firmware embedding one language only contains the strings it displays.
This guide builds the same code once per language, from the command line, from `.cargo/config.toml` and from a CI matrix, without editing the sources.

## Override the Locale for One Build

The `EMBEDDED_I18N_LOCALE` environment variable overrides the `default_locale` of [`i18n!`] at compile time:

```sh
EMBEDDED_I18N_LOCALE=fr cargo build --release
```

Changing the variable rebuilds the crates calling [`i18n!`], so successive builds with different values are always up to date.
The special value `pseudo` builds a [pseudo-localization][how-to-pseudo-localize].

To embed several locales for [runtime selection][how-to-switch-language] instead, set `EMBEDDED_I18N_LOCALES`, comma separated:

```sh
EMBEDDED_I18N_LOCALE=fr EMBEDDED_I18N_LOCALES=en,fr,de cargo build --release
```

`EMBEDDED_I18N_LOCALE` must then be one of the listed locales.

## Set the Locale for Your Team

Set the variable once in `.cargo/config.toml`, to share it with your team and CI:

```toml
[env]
EMBEDDED_I18N_LOCALE = "fr"
```

To make the language part of the code instead, set it in `Cargo.toml` (or in the arguments of [`i18n!`]), and keep the variable for one-off overrides:

```toml
[package.metadata.i18n]
default-locale = "fr"
```

The variables win over the arguments, which win over `[package.metadata.i18n]`, see [Configuration][reference-configuration].

## Build Every Language in CI

A matrix job builds one firmware per language:

```yaml
strategy:
  matrix:
    locale: [en, fr, de]
steps:
  - run: cargo build --release
    env:
      EMBEDDED_I18N_LOCALE: ${{ matrix.locale }}
```

Each build checks the placeholders of its language, so a translator writing `{nom}` instead of `{name}` fails the job of that language.
Add `strict = true` to the metadata of the crate to also fail on untranslated strings, see [How to Ship Complete Translations][how-to-ship-complete].

## Use the Locale in a Build Script

A build script runs before the crate is compiled and cannot see the arguments of [`i18n!`].
Read the same variable to prepare language-specific assets, such as the font subset of the locale (see [How to Subset Fonts][how-to-subset-fonts]):

```rust,no_run
fn main() {
    println!("cargo:rerun-if-env-changed=EMBEDDED_I18N_LOCALE");
    let locale = std::env::var("EMBEDDED_I18N_LOCALE").unwrap_or_else(|_| "en".to_string());
    println!("cargo:warning=building assets for {locale}");
}
```

When the language comes from `Cargo.toml` rather than the variable, give the build script its own setting, such as a variable with the same default.

## Name the Firmware After Its Language

In the firmware, [`locale!`] is the name of the embedded locale, a literal usable in `const` items:

```rust
embedded_i18n::i18n!("locales", default_locale = "fr");

const LANGUAGE: &str = embedded_i18n::locale!();
assert_eq!(LANGUAGE, "fr");
assert_eq!(embedded_i18n::t!("greeting"), "Bonjour");
```

[how-to-pseudo-localize]: pseudo_localize.md
[how-to-switch-language]: switch_language_at_runtime.md
[how-to-ship-complete]: ship_complete_translations.md
[how-to-subset-fonts]: subset_fonts.md
[reference-configuration]: ../reference/configuration.md
[`i18n!`]: ../reference/configuration.md
[`locale!`]: ../reference/runtime_api.md
