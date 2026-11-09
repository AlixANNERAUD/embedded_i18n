# Right-to-Left Languages

Arabic, Hebrew, Persian and Urdu are written right to left.
Two things need care on an embedded display: mirroring the layout, and interpolating left-to-right values (names, numbers, file names) into right-to-left text.

## Text Direction

[`is_right_to_left`] tells whether a locale is written right to left.
It is a `const fn`, and [`locale!`] is a literal when a single locale is embedded, so a layout can be mirrored at compile time:

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

Right-to-left languages (`ar`, `he`, `fa`, `ur`, `ps`, `sd`, `ug`, `yi`, `ckb`, ...) and the `Arab` and `Hebr` script subtags are recognized, case-insensitively, with `-` or `_` separators.

## Bidi Isolation

When a left-to-right value is inserted into right-to-left text, the Unicode bidirectional algorithm can move it, or the punctuation next to it, to the wrong place.
Set `bidi_isolation = true` in [`i18n!`], or `bidi-isolation = true` in `[package.metadata.i18n]`, to wrap every `{...}` placeholder of messages coming from a right-to-left locale in Unicode isolates, U+2068 FIRST STRONG ISOLATE and U+2069 POP DIRECTIONAL ISOLATE, as Fluent does:

```text
"مرحبا {name}"  →  "مرحبا \u{2068}{name}\u{2069}"
```

```rust
embedded_i18n::i18n!("locales", bidi_isolation = true);
# fn main() {}
```

The wrapping happens at compile time, in the translation itself, so it costs two characters per placeholder and no code.
Messages coming from left-to-right locales, for instance through a fallback, are not changed.

It is off by default: many embedded text renderers do not implement the Unicode bidirectional algorithm and would draw the isolates as missing glyphs.
Enable it when your renderer (or the font shaping library) handles them.

[`is_right_to_left`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.is_right_to_left.html
[`i18n!`]: configuration.md
[`locale!`]: runtime_locales.md
