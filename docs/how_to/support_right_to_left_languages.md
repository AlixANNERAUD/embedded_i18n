# How to Support Right-to-Left Languages

Arabic, Hebrew, Persian and Urdu are written right to left.
Two things need care on an embedded display: mirroring the layout, and inserting left-to-right values (names, numbers, file names) into right-to-left text.

## Mirror the Layout

[`is_right_to_left`] tells whether a locale is written right to left.
It is a `const fn`, and [`locale!`] is a literal when a single locale is embedded, so the layout can be chosen at compile time:

```rust
use embedded_i18n::is_right_to_left;

embedded_i18n::i18n!("locales");

const MIRRORED: bool = is_right_to_left(embedded_i18n::locale!());
assert!(!MIRRORED);

assert!(is_right_to_left("ar-EG"));
assert!(is_right_to_left("he"));
assert!(is_right_to_left("pa_Arab"));
assert!(!is_right_to_left("fr-CA"));
```

With [several embedded locales][how-to-switch-language], call it at runtime after [`set_locale`], with the same [`locale!`].
The recognized languages and scripts are listed in the [runtime API reference][reference-runtime-api].

## Isolate the Placeholders

When a left-to-right value is inserted into right-to-left text, the Unicode bidirectional algorithm can move it, or the punctuation next to it, to the wrong place.
Set `bidi_isolation = true` in [`i18n!`], or `bidi-isolation = true` in `[package.metadata.i18n]`:

```rust
embedded_i18n::i18n!("locales", bidi_isolation = true);
# fn main() {}
```

Every `{...}` placeholder of the messages coming from a right-to-left locale is then wrapped in Unicode isolates, U+2068 FIRST STRONG ISOLATE and U+2069 POP DIRECTIONAL ISOLATE, as Fluent does:

```text
"مرحبا {name}"  →  "مرحبا \u{2068}{name}\u{2069}"
```

The wrapping happens at compile time, in the translation itself: it costs two characters per placeholder and no code.
Messages coming from left-to-right locales, for instance through a fallback, are not changed.

Enable it only when your text renderer, or its font shaping library, implements the Unicode bidirectional algorithm.
Many embedded renderers do not, and would draw the isolates as missing glyphs, which is why it is off by default.

[how-to-switch-language]: switch_language_at_runtime.md
[reference-runtime-api]: ../reference/runtime_api.md
[`is_right_to_left`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.is_right_to_left.html
[`i18n!`]: ../reference/configuration.md
[`locale!`]: ../reference/runtime_api.md
[`set_locale`]: ../reference/runtime_api.md
