# How to Localize per Request

A server answering clients in their own language, or a device driving several displays in different languages, cannot rely on one global current locale.
This guide translates each message into a locale given at the call, without touching the global one.

## Embed the Locales

Embed every locale you serve with `available_locales`, as in [How to Switch Language at Runtime][how-to-switch-language]:

```rust,standalone_crate
embedded_i18n::i18n!("locales", default_locale = "en", available_locales = ["en", "fr"]);
# fn main() {}
```

## Pass the Locale to `t!`

Add `locale = ...` to [`t!`]:

```rust,standalone_crate
use embedded_i18n::{Locale, t};

embedded_i18n::i18n!("locales", default_locale = "en", available_locales = ["en", "fr"]);

fn main() {
    // A string literal is resolved at compile time: this is the French literal.
    assert_eq!(t!("greeting", locale = "fr"), "Bonjour");

    // Any `Into<Locale>` (`&str`, `Locale`) is resolved at runtime.
    let language = "fr-CA";
    assert_eq!(t!("greeting", locale = language), "Bonjour");

    const ENGLISH: Locale = Locale::new("en");
    assert_eq!(
        t!("files", count = 3, locale = ENGLISH).to_string(),
        "3 files"
    );
}
```

The locale is matched to an embedded locale as by [`set_locale`]: the same tag, else the same language, else the `default_locale`.
The global current locale is left untouched, so other code keeps translating into it.

## Handle a Request

Parse the language of the request once into a [`Locale`], a `Copy` value, and pass it to every call:

```rust,standalone_crate
use core::fmt::Write;
use embedded_i18n::{Locale, t};

embedded_i18n::i18n!("locales", default_locale = "en", available_locales = ["en", "fr"]);

fn answer(output: &mut impl Write, language: &str, name: &str) -> core::fmt::Result {
    let locale = Locale::from(language);
    write!(output, "{}", t!("with_named_args", name = name, count = 1, locale = locale))
}

fn main() {
    let mut output = String::new();
    answer(&mut output, "fr", "Ada").unwrap();
    assert_eq!(output, "Bonjour Ada, vous avez 1 messages");
}
```

The translation is chosen with a few integer comparisons, without looking up a string.

## Build With a Single Locale Too

With a single embedded locale, the `locale` expression is evaluated and ignored, so the same code builds in both modes:

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

let language = "fr";
assert_eq!(t!("greeting", locale = language), "Hello");
```

## See Also

- [The `t!` macro][reference-t] and the [runtime API][reference-runtime-api].
- [The Runtime Locale Model][explanation-runtime-locale-model] for how tags are matched.

[how-to-switch-language]: switch_language_at_runtime.md
[reference-t]: ../reference/t_macro.md
[reference-runtime-api]: ../reference/runtime_api.md
[explanation-runtime-locale-model]: ../explanation/runtime_locale_model.md
[`t!`]: ../reference/t_macro.md
[`set_locale`]: ../reference/runtime_api.md
[`Locale`]: ../reference/runtime_api.md
