# How to Switch Language at Runtime

By default a firmware embeds one locale, and every [`t!`] call expands to a literal.
This guide embeds several locales so that the user can pick the language at runtime: from a settings menu, a DIP switch, or a configuration stored in flash.

## Embed the Locales

List the locales to embed with `available_locales` in [`i18n!`]:

```rust,standalone_crate
embedded_i18n::i18n!("locales", default_locale = "en", available_locales = ["en", "fr", "de"]);
# fn main() {}
```

or in the `[package.metadata.i18n]` table of `Cargo.toml`:

```toml
[package.metadata.i18n]
default-locale = "en"
available-locales = ["en", "fr", "de"]
```

`default_locale` is the locale active at startup, and must be one of `available_locales`.
To choose the locales when building instead, set the `EMBEDDED_I18N_LOCALES` variable (comma separated), see [Configuration][reference-configuration].

Every message must then be found for every embedded locale, through its fallback chain: the build fails and names the locales missing it otherwise.

## Select the Locale

Call [`set_locale`] with a locale tag.
Every [`t!`] call then translates into the current locale:

```rust,standalone_crate
use embedded_i18n::{set_locale, t};

embedded_i18n::i18n!("locales", default_locale = "en", available_locales = ["en", "fr"]);

fn main() {
    assert_eq!(t!("greeting"), "Hello");

    set_locale("fr");
    assert_eq!(t!("greeting"), "Bonjour");
    assert_eq!(t!("files", count = 2).to_string(), "2 fichiers");

    // A regional tag is served by the closest embedded locale.
    set_locale("fr-CA");
    assert_eq!(t!("greeting"), "Bonjour");
    assert_eq!(embedded_i18n::locale!(), "fr");

    // A locale that is not embedded is served by `default_locale`.
    set_locale("de");
    assert_eq!(t!("greeting"), "Hello");
}
```

[`set_locale`] never fails: each crate translates into its closest embedded locale, then into its `default_locale`.
[`locale!`] returns the locale the calling crate actually translates into, which can differ from the requested one.

## List the Languages in a Menu

[`available_locales!`] returns the embedded locales, in configuration order:

```rust,standalone_crate
use embedded_i18n::{set_locale, t};

embedded_i18n::i18n!("locales", default_locale = "en", available_locales = ["en", "fr"]);

fn main() {
    assert_eq!(embedded_i18n::available_locales!(), ["en", "fr"]);

    for &language in embedded_i18n::available_locales!() {
        set_locale(language);
        assert_eq!(embedded_i18n::locale!(), language);
        let greeting: &'static str = t!("greeting");
        // en: "Hello", fr: "Bonjour"
        assert!(!greeting.is_empty());
    }
}
```

Store the selected tag (or its index in this list) in flash, and pass it to [`set_locale`] at startup.

## Start in the Language of the Operating System

On a hosted target, enable the `std` feature and pass [`system_locale`] to [`set_locale`] at startup:

```rust,standalone_crate
use embedded_i18n::{set_locale, system_locale};

embedded_i18n::i18n!("locales", available_locales = ["en", "fr"]);

fn main() {
    set_locale(&system_locale().unwrap_or_default());
}
```

It reads `LC_ALL`, `LC_MESSAGES` then `LANG`, and allocates a `String` on each call, so call it once.
An empty tag leaves the `default_locale` active.
Windows and macOS graphical applications have no such variables: use the [`sys-locale`](https://crates.io/crates/sys-locale) crate there, and pass its result to [`set_locale`].

## Format Dates in the Current Language

Pass [`locale!`] to [`date_time_locale`]:

```rust,standalone_crate
use embedded_i18n::{date_time_locale, set_locale, DateTimeFormatter};

embedded_i18n::i18n!("locales", default_locale = "en", available_locales = ["en", "fr"]);

fn main() {
    set_locale("fr");
    let formatter = DateTimeFormatter::new(date_time_locale(embedded_i18n::locale!()));
    assert_eq!(formatter.display(1_791_884_800, "%d %B").to_string(), "13 octobre");
}
```

With a runtime locale, this links the date and time names of every supported language, see [Date and Time][reference-date-time].

## Keep `const` Items Working

With several locales, [`t!`] returns a runtime value, so it can no longer initialize a `const` or a `static`: the compiler reports a non-const expression.
Move such items into functions, or translate them into a fixed locale with a literal `locale = "fr"`, which is still resolved at compile time (see [How to Localize per Request][how-to-localize-per-request]).

Code calling [`set_locale`], [`locale!`] and [`available_locales!`] still builds with a single locale: [`set_locale`] has no effect then, since only the default locale exists.
A library can therefore call them whatever the configuration.

## See Also

- [Runtime API][reference-runtime-api] for the items used here.
- [The Runtime Locale Model][explanation-runtime-locale-model] for how a tag is matched to an embedded locale, and its limitations.
- [How It Works][explanation-how-it-works] for the ROM and RAM cost of embedding several locales.

[reference-configuration]: ../reference/configuration.md
[reference-date-time]: ../reference/date_time.md
[reference-runtime-api]: ../reference/runtime_api.md
[how-to-localize-per-request]: localize_per_request.md
[explanation-runtime-locale-model]: ../explanation/runtime_locale_model.md
[explanation-how-it-works]: ../explanation/how_it_works.md
[`t!`]: ../reference/t_macro.md
[`i18n!`]: ../reference/configuration.md
[`set_locale`]: ../reference/runtime_api.md
[`locale!`]: ../reference/runtime_api.md
[`available_locales!`]: ../reference/runtime_api.md
[`system_locale`]: ../reference/runtime_api.md
[`date_time_locale`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.date_time_locale.html
