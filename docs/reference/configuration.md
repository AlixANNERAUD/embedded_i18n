# Configuration

[`i18n!`] configures the translation macros of a crate: where the locale files are, which locales are embedded, and how messages fall back.
Call it once at the crate root (`lib.rs` or `main.rs`), before any `mod` declaration:

```rust
embedded_i18n::i18n!("locales", fallback = "en");
# fn main() {}
```

Every option can also come from the `[package.metadata.i18n]` table of the `Cargo.toml` of the crate, in which case the call can have no argument:

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

Each crate, integration test, example and doctest that translates is a separate crate, and calls [`i18n!`] once.
A doctest with several available locales is marked `standalone_crate` and has an explicit `fn main`, with [`i18n!`] at its top level.

## Options

| `i18n!` argument | `[package.metadata.i18n]` key | Type | Default | Description |
| --- | --- | --- | --- | --- |
| first positional string | `load-path` | string | `"locales"` | Locale files directory, relative to the `Cargo.toml` of the crate |
| `fallback` | `fallback` | string or array of strings | `["en"]` | Fallback chain after the locale and its less specific forms |
| `default_locale` | `default-locale` | string | `"en"` | Locale active at startup, and the only one embedded unless `available_locales` is set |
| `available_locales` | `available-locales` | array of strings | `[default_locale]` | Locales to embed: more than one enables [runtime selection][how-to-switch-language] |
| `strict` | `strict` | boolean | `false` | Forbids falling back for messages missing from the primary locale, see [How to Ship Complete Translations][how-to-ship-complete] |
| `bidi_isolation` | `bidi-isolation` | boolean | `false` | Isolates the placeholders of right-to-left translations, see [How to Support Right-to-Left Languages][how-to-right-to-left] |

The metadata keys are those of rust-i18n where they exist.
In [`i18n!`], strings and booleans are literals, and arrays are written `["en", "fr"]` (a trailing comma is allowed).

A single locale is embedded by default, so the firmware contains the strings of `default_locale` only:

```rust
embedded_i18n::i18n!("locales", default_locale = "fr");
assert_eq!(embedded_i18n::t!("greeting"), "Bonjour");
assert_eq!(embedded_i18n::locale!(), "fr");
```

## Environment Overrides

Two environment variables are read at compile time, to build one firmware per language without editing the code:

| Variable | Overrides | Format |
| --- | --- | --- |
| `EMBEDDED_I18N_LOCALE` | `default_locale` | A locale; surrounding whitespace is trimmed, and an empty value is ignored |
| `EMBEDDED_I18N_LOCALES` | `available_locales` | Comma separated locales; entries are trimmed, empty entries dropped, and a value without any entry is ignored |

```sh
EMBEDDED_I18N_LOCALE=fr cargo build --release
```

They can be set in the `[env]` table of `.cargo/config.toml`.

The overrides apply to every crate of the build that calls [`i18n!`], dependencies included.
`EMBEDDED_I18N_LOCALE` adapts to each crate:

- without configured `available_locales`, the requested locale becomes the only one embedded;
- with configured `available_locales` (in the arguments or the metadata) that contain it, it becomes the `default_locale`;
- with configured `available_locales` that do not contain it, the `default_locale` is the available locale with the same [runtime identifier][explanation-runtime-locale-model] (`fr_ca` for `fr-CA`), else the first one of the same language (the language-only locale first), else the crate's own `default_locale`.

`EMBEDDED_I18N_LOCALES` replaces the `available_locales` of every crate as given, and `EMBEDDED_I18N_LOCALE`, or else the crate's own `default_locale`, must be one of them.
Every other setting lives in the arguments or the metadata, so it is versioned with the code.
A build script cannot see the arguments of [`i18n!`], but it can read these variables.

## Precedence

From the highest to the lowest priority, for each option:

1. the environment overrides;
2. the arguments of [`i18n!`];
3. the `[package.metadata.i18n]` table;
4. the defaults.

## Special Locales

`pseudo`, as `default_locale` or in `available_locales`, is a [pseudo-localization][how-to-pseudo-localize] of the fallback locales.
It keeps the plural rules of the locale each message comes from, and is always complete in strict mode.

## Locale Resolution

A message is looked up in a chain of locales, from the most specific to the most generic:

1. the locale being translated into (`default_locale`, or each embedded locale);
2. the same locale without its last subtag, repeatedly: `zh-Hant-TW`, `zh-Hant`, `zh`;
3. each fallback locale (`fallback`), in order, with the same subtag removal.

With `default_locale = "fr-CA"` and `fallback = ["de", "en"]`, the chain is `fr-CA`, `fr`, `de`, `en`.
A locale appearing twice is only tried once.
The first non-empty translation wins, and a message missing from every locale is a compile error listing the chain.

The primary locale of strict mode is the first part of the chain: the locale and its less specific forms (`fr-CA` and `fr`).

Locale identifiers are normalized like BCP 47 tags: `fr_ca`, `FR-CA` and `fr-CA` are the same locale, scripts are title-cased (`zh-hant` becomes `zh-Hant`) and regions upper-cased.
The file of a locale can be named with `-` or `_`, in its normalized case or in lower case: `fr-CA.json`, `fr_CA.json`, `fr-ca.json` or `fr_ca.json`.
Missing files are skipped.

## Locale Files Directory

By default, locale files are read from `locales/`, next to the `Cargo.toml` of the crate calling [`i18n!`].
The first positional argument, or `load-path`, changes it, relative to that `Cargo.toml`.

To share one set of locale files between the crates of a workspace, point each crate to the shared directory:

```toml
[package.metadata.i18n]
load-path = "../i18n"
```

## Rebuild Tracking

[`i18n!`] tracks the files it reads, so that Cargo rebuilds the crate when they change.
Nothing is added to the binary, and there is nothing to call besides [`i18n!`].

| Change | Rebuild |
| --- | --- |
| Editing a loaded locale file | yes |
| Editing `Cargo.toml` | yes |
| Changing `EMBEDDED_I18N_LOCALE` or `EMBEDDED_I18N_LOCALES` | yes |
| Enabling another backend feature | yes |
| Creating a locale file that did not exist at the previous build | no, run `cargo clean -p <crate>` or touch a source file |

A locale file created after the build, such as a new `fr-CA.json` next to an existing `fr.json`, is not seen until the crate is rebuilt for another reason, because only existing files are tracked.
To also pick up new files, add a build script to the crate calling [`i18n!`]:

```rust,ignore
// build.rs
fn main() {
    println!("cargo:rerun-if-changed=locales");
}
```

Cargo then reruns the build script, and rebuilds the crate, whenever a file of `locales/` is added, removed or modified.
Adjust the path if you set `load-path`.

## Generated Items

[`i18n!`] expands, at the crate root, to:

- a hidden `macro_rules!` named `embedded_i18n_requires_i18n_at_the_crate_root`, through which [`t!`], [`locale!`] and [`available_locales!`] reach the configuration;
- the rebuild tracking of the files and variables above;
- with several available locales, the crate's hidden function resolving the current locale to one of its locales.

See [How It Works][explanation-how-it-works].

## Compile Errors

Errors are reported on the [`i18n!`] call, except the errors of a misplaced call, which rustc reports on the macros using it.
A second call in a submodule makes the macros ambiguous, and a call outside the crate root with several available locales (in `fn main`, in a module, or in a doctest that is not a `standalone_crate`) defines the resolution function out of the crate root, where the macros look for it.

| Mistake | Message |
| --- | --- |
| Unknown option | ``unknown option `foo`, expected one of: fallback, default_locale, available_locales, strict, bidi_isolation`` |
| Option given twice | `` `strict` is given twice `` |
| Value of the wrong type | the parser error, such as `expected boolean literal` |
| Invalid metadata | `invalid [package.metadata.i18n] in /path/to/crate/Cargo.toml: <detail>` |
| Unknown metadata key | ``<…>: unknown key `foo`, expected one of: load-path, fallback, default-locale, available-locales, strict, bidi-isolation`` |
| Metadata value of the wrong type | ``<…>: `strict` must be a boolean``, `` `default-locale` must be a string``, `` `available-locales` must be an array of strings``, `` `fallback` must be a string or an array of strings`` |
| Empty locale name | `locale names cannot be empty` |
| `default_locale` missing from `available_locales` | `default locale "de" is not in available_locales ["en", "fr"]` |
| More than 255 available locales | `available_locales lists 256 locales, at most 255 are supported` |
| Two available locales with the same [runtime identifier][explanation-runtime-locale-model] | `available locales "fr-CA" and "fr_ca" have the same runtime identifier` |
| Missing locale directory | `locales directory not found: /path/to/crate/i18n` |
| A second call in the same crate | ``the name `EMBEDDED_I18N_I18N_IS_CALLED_ONCE_PER_CRATE` is defined multiple times`` |
| A second call in a submodule | ``error[E0659]: `embedded_i18n_requires_i18n_at_the_crate_root` is ambiguous`` |
| A call outside the crate root with several locales | ``error[E0425]: cannot find function `__embedded_i18n_resolve` in the crate root`` |

```rust,compile_fail
embedded_i18n::i18n!(available_locales = ["fr-CA", "fr_ca"], default_locale = "fr-CA");
```

```rust,compile_fail,E0659
embedded_i18n::i18n!("locales");

mod screen {
    embedded_i18n::i18n!("locales");

    pub fn title() -> &'static str {
        embedded_i18n::t!("greeting")
    }
}

fn main() {
    let _ = screen::title();
}
```

```rust,compile_fail,E0425
fn main() {
    embedded_i18n::i18n!("locales", available_locales = ["en", "fr"]);
    let _ = embedded_i18n::t!("greeting");
}
```

Errors of the locale files themselves are reported on the call too, see [Locale Files][reference-locale-files].

[reference-locale-files]: locale_files.md
[how-to-switch-language]: ../how_to/switch_language_at_runtime.md
[how-to-ship-complete]: ../how_to/ship_complete_translations.md
[how-to-right-to-left]: ../how_to/support_right_to_left_languages.md
[how-to-pseudo-localize]: ../how_to/pseudo_localize.md
[explanation-how-it-works]: ../explanation/how_it_works.md
[explanation-runtime-locale-model]: ../explanation/runtime_locale_model.md
[`i18n!`]: configuration.md
[`t!`]: t_macro.md
[`locale!`]: runtime_api.md
[`available_locales!`]: runtime_api.md
