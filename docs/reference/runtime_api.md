# Runtime API

The items to select and read the locale at runtime.
They are useful when several locales are embedded (`available_locales`, see [Configuration][reference-configuration]), and build with a single locale too.

| Item | Kind | Description |
| --- | --- | --- |
| [`set_locale`]`(&str)` | function | Sets the global current locale |
| [`Locale`] | struct | `Copy` handle of a locale tag, packed into a `u32` |
| [`locale!`]`()` | macro | Name of the locale the calling crate translates into, a `&'static str` |
| [`available_locales!`]`()` | macro | Embedded locales of the calling crate, a `&'static [&'static str]` |
| [`system_locale`]`()` | function, `std` feature | Locale of the operating system, an `Option<String>` |
| [`is_right_to_left`]`(&str)` | `const fn` | Whether a locale is written right to left |
| `t!("key", locale = ...)` | [`t!`] option | Translates into a given locale, see [The `t!` Macro][reference-t] |

## `set_locale`

```text
pub fn set_locale(locale: &str)
```

- Stores the [`Locale`] identifier of the tag in a global `AtomicU32`, with a plain store: no compare-and-swap, no allocation, no lock.
- Returns `()`, as in rust-i18n, and never fails: each crate translates into its closest embedded locale, see [Resolution](#resolution).
- An empty tag restores the unset state, in which every crate uses its `default_locale`.
- The current locale is shared by every crate of the program.
- With a single embedded locale, it has no effect on [`t!`].

## `Locale`

```rust
use embedded_i18n::Locale;

const FRENCH_CANADA: Locale = Locale::new("fr-CA");

assert_eq!(Locale::new("fr_CA"), Locale::new("FR-ca"));
assert_eq!(Locale::from("fr-CA"), FRENCH_CANADA);
assert_eq!(Locale::new("en").id(), 0x015C_0000);
assert_eq!(Locale::NONE.id(), 0);
```

| Item | Description |
| --- | --- |
| `Locale::new(&str)` | `const fn` packing a tag into its identifier |
| `From<&str>` | the same, for `Into<Locale>` arguments |
| `.id()` | the `u32` identifier |
| `Locale::NONE` | identifier `0`, the empty tag: no locale set |
| `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `Hash` | derived |

Tags are normalized before packing: surrounding ASCII whitespace trimmed, ASCII case ignored, `_` equivalent to `-`.
The layout of the identifier:

| Bits | Content |
| --- | --- |
| 31 to 17 | The language subtag, when it is 1 to 3 ASCII letters, 5 bits per letter (`a` is 1), `0` for none |
| 16 to 0 | A hash of the rest of the tag (script, region, variants), `0` when there is none and never `0` otherwise |

A tag that does not start with a 1 to 3 letter language subtag (`pseudo`) uses the language value `0x7FFF` and hashes the whole tag into the low bits: it only matches exactly.
See [The Runtime Locale Model][explanation-runtime-locale-model] for the reasons and the limitations.

## `locale!`

Returns the name of the locale the calling crate translates into, as a `&'static str`.

- With a single embedded locale, it is the `default_locale` literal, usable in `const` items.
- With several, it is the embedded locale the current locale resolves to, which can differ from the tag given to [`set_locale`] (`fr-CA` can resolve to `fr`).

```rust
embedded_i18n::i18n!();

assert_eq!(embedded_i18n::locale!(), "en");
```

## `available_locales!`

Returns the locales embedded in the calling crate, as a `&'static [&'static str]`, in configuration order.
With a single locale, it is `[default_locale]`.

```rust
embedded_i18n::i18n!();

assert_eq!(embedded_i18n::available_locales!(), ["en"]);
```

## `system_locale`

```text
#[cfg(feature = "std")]
pub fn system_locale() -> Option<String>
```

- Reads `LC_ALL`, `LC_MESSAGES`, then `LANG`; the first variable holding a usable value wins.
- Skips empty, `C` and `POSIX` values (and `C.UTF-8`).
- Normalizes the value to a tag: `fr_FR.UTF-8@euro` becomes `fr-FR`.
- Returns `None` when no variable is usable.
- Allocates a `String` on each call.
- Reads the POSIX environment only: Windows and macOS graphical applications should use the [`sys-locale`](https://crates.io/crates/sys-locale) crate instead.

## `is_right_to_left`

A `const fn` returning whether a locale is written right to left.
It recognizes, case-insensitively and with `-` or `_` separators, the right-to-left languages `ar`, `arc`, `ckb`, `dv`, `fa`, `he`, `iw`, `ks`, `ps`, `sd`, `syr`, `ug`, `ur` and `yi`, and the `Arab` and `Hebr` script subtags (`pa_Arab`).

```rust
use embedded_i18n::is_right_to_left;

assert!(is_right_to_left("ar-EG"));
assert!(is_right_to_left("pa_Arab"));
assert!(!is_right_to_left("fr-CA"));
```

## Resolution

With several embedded locales, each crate maps the current locale, or the `locale = ...` of a [`t!`] call, to one of its embedded locales, in this order:

1. an exact identifier match;
2. the same language: the locale that is the language alone (`fr`) if embedded, otherwise the first embedded locale of that language (`fr-CA`);
3. the `default_locale`.

The unset state (`Locale::NONE`) resolves to the `default_locale`.
A literal `locale = "fr"` is resolved the same way, at compile time.

```rust,standalone_crate
use embedded_i18n::{set_locale, t};

embedded_i18n::i18n!("locales", default_locale = "en", available_locales = ["en", "fr"]);

fn main() {
    set_locale("fr-CA");
    assert_eq!(embedded_i18n::locale!(), "fr");
    assert_eq!(t!("greeting"), "Bonjour");

    set_locale("de");
    assert_eq!(embedded_i18n::locale!(), "en");

    set_locale("");
    assert_eq!(embedded_i18n::locale!(), "en");
}
```

## Limits

- Up to 255 locales can be embedded in a crate.
- Two embedded locales with the same identifier are a compile error on [`i18n!`].
- With several embedded locales, [`t!`] returns runtime values, which cannot initialize `const` or `static` items.

[reference-configuration]: configuration.md
[reference-t]: t_macro.md
[explanation-runtime-locale-model]: ../explanation/runtime_locale_model.md
[`t!`]: t_macro.md
[`i18n!`]: configuration.md
[`set_locale`]: runtime_api.md
[`Locale`]: runtime_api.md
[`locale!`]: runtime_api.md
[`available_locales!`]: runtime_api.md
[`system_locale`]: runtime_api.md
[`is_right_to_left`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.is_right_to_left.html
