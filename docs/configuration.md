# Configuration

[`i18n!`] configures the translation macros of a crate.
Call it once at the crate root (`lib.rs` or `main.rs`), before any `mod` declaration:

```rust
embedded_i18n::i18n!("locales", fallback = "en");
# fn main() {}
```

Everything can also come from the `[package.metadata.i18n]` table of the `Cargo.toml` of the crate, in which case the call has no argument:

```toml
[package.metadata.i18n]
load-path = "locales"
default-locale = "fr"
fallback = ["en"]
```

```rust
embedded_i18n::i18n!();
# fn main() {}
```

## Options

| `i18n!` argument | `[package.metadata.i18n]` key | Default | Description |
| --- | --- | --- | --- |
| first positional string | `load-path` | `"locales"` | Locale files directory, relative to the `Cargo.toml` of the crate |
| `fallback = "en"` or `["en", "es"]` | `fallback` | `["en"]` | Fallback chain after the locale and its less specific forms |
| `default_locale = "fr"` | `default-locale` | `"en"` | Locale active at startup, and the only one embedded unless `available_locales` is set |
| `available_locales = ["en", "fr"]` | `available-locales` | `[default_locale]` | Locales to embed: more than one enables [runtime selection][guide-runtime-locales] |
| `strict = true` | `strict` | `false` | Forbids falling back for messages missing from the primary locale, see [Coverage][guide-coverage] |
| `bidi_isolation = true` | `bidi-isolation` | `false` | Isolates the placeholders of right-to-left translations, see [Right-to-Left Languages][guide-right-to-left] |

The metadata keys are those of rust-i18n where they exist.
A single locale is embedded by default, so the firmware contains the strings of `default_locale` only:

```rust
embedded_i18n::i18n!("locales", default_locale = "fr");
assert_eq!(embedded_i18n::t!("greeting"), "Bonjour");
assert_eq!(embedded_i18n::locale!(), "fr");
```

Mistakes in the call are compile errors: an unknown option, a value of the wrong type, more than 255 available locales, a `default_locale` missing from `available_locales`, two available locales with the same [runtime identifier][guide-runtime-locales], or an unreadable locale directory.

## Precedence

From the highest to the lowest priority:

1. the environment overrides below;
2. the arguments of [`i18n!`];
3. the `[package.metadata.i18n]` table;
4. the defaults.

## Environment Overrides

Two environment variables are read at compile time, for CI matrices building one firmware per language:

| Variable | Overrides | Description |
| --- | --- | --- |
| `EMBEDDED_I18N_LOCALE` | `default_locale` | Locale to embed, or the locale active at startup with several available locales |
| `EMBEDDED_I18N_LOCALES` | `available_locales` | Comma separated locales to embed for [runtime selection][guide-runtime-locales] |

Set them on the command line for a one-off build:

```sh
EMBEDDED_I18N_LOCALE=fr cargo build --release
```

or once in `.cargo/config.toml`, to share them with your team and CI:

```toml
[env]
EMBEDDED_I18N_LOCALE = "fr"
```

Changing a variable rebuilds the crates calling [`i18n!`].
`EMBEDDED_I18N_LOCALE=pseudo` builds a [pseudo-localization][guide-pseudo-localization] of the fallback locales.
Every other setting lives in the arguments or the metadata, so it is versioned with the code.

## Locale Resolution

A message is looked up in a chain of locales, from the most specific to the most generic:

1. the selected locale (`default_locale`);
2. the same locale without its last subtag, repeatedly: `zh-Hant-TW`, `zh-Hant`, `zh`;
3. each fallback locale (`fallback`), in order, with the same subtag removal.

With `default_locale = "fr-CA"` and `fallback = ["de", "en"]`, the chain is `fr-CA`, `fr`, `de`, `en`.
A locale appearing twice is only tried once.
The first non-empty translation wins, and a message missing from every locale is a compile error listing the chain.

Locale identifiers are normalized like BCP 47 tags: `fr_ca`, `FR-CA` and `fr-CA` are the same locale, scripts are title-cased (`zh-hant` becomes `zh-Hant`) and regions upper-cased.
The file of a locale can be named with `-` or `_`, in its normalized case or in lower case: `fr-CA.json`, `fr_CA.json`, `fr-ca.json` or `fr_ca.json`.
Missing files are skipped.

## Locale Files Directory

By default, locale files are read from `locales/`, next to the `Cargo.toml` of the crate calling [`i18n!`].
The first positional argument, or `load-path`, changes it, relative to that `Cargo.toml`.

To share one set of locale files between the crates of a workspace, point each crate to the shared directory, with `load-path` or the first argument of [`i18n!`]:

```toml
[package.metadata.i18n]
load-path = "../i18n"
```

A missing directory is a compile error naming the resolved path:

```text
error: locales directory not found: /path/to/crate/i18n
```

## Rebuild Tracking

Cargo rebuilds a crate when its sources change, but it does not know that macros read other files.
[`i18n!`] therefore tracks everything it reads, and Cargo rebuilds the crate when any of it changes.
Nothing is added to the binary, and there is nothing to call besides [`i18n!`].

| Change | Rebuild |
| --- | --- |
| Editing a loaded locale file | yes |
| Editing `Cargo.toml` | yes |
| Changing `EMBEDDED_I18N_LOCALE` or `EMBEDDED_I18N_LOCALES` | yes |
| Creating a locale file that did not exist at the previous build | no, run `cargo clean -p <crate>` or touch a source file |
| Enabling another backend feature | yes |

A locale file created after the build, such as a new `fr-CA.json` next to an existing `fr.json`, is not seen until the crate is rebuilt for another reason, because only existing files are tracked.

To also pick up new files, add a build script to the crate calling [`i18n!`]:

```rust,no_run
fn main() {
    println!("cargo:rerun-if-changed=locales");
}
```

Cargo then reruns the build script, and rebuilds the crate, whenever a file of `locales/` is added, removed or modified.
Adjust the path if you set `load-path`.

## Reading the Locale

[`locale!`] returns the name of the locale the calling crate translates into, as a `&'static str`.
It is a literal when a single locale is embedded, so it works in `const` context.
[`available_locales!`] lists the embedded locales, see [Runtime Locale Selection][guide-runtime-locales].

## Build Scripts

A build script runs before the crate is compiled and cannot see the arguments of [`i18n!`].
To pick the font subset of a locale (see [Unicode Ranges][guide-unicode-ranges]), read the same `EMBEDDED_I18N_LOCALE` variable, or your own setting.

[guide-runtime-locales]: runtime_locales.md
[guide-coverage]: coverage.md
[guide-right-to-left]: right_to_left.md
[guide-pseudo-localization]: pseudo_localization.md
[guide-unicode-ranges]: unicode_ranges.md
[`i18n!`]: configuration.md
[`locale!`]: runtime_locales.md
[`available_locales!`]: runtime_locales.md
