# Unicode Ranges

The Unicode blocks needed to render each language, to subset fonts.

## API

| Item | Description |
| --- | --- |
| [`get_locale_ranges`]`(&str)` | The ranges of a language, as `Option<&'static [Range<u32>]>`; `None` for an unsupported locale |
| [`merge_contiguous_ranges_in_place`]`(&mut [Range<u32>])` | Sorts the ranges and merges the overlapping and adjacent ones, without allocation; returns the number of merged ranges, stored at the start of the slice |
| `write_range(&mut impl Write, &Range<u32>)` | Writes a range as `start-end` (inclusive end), or `start` for a single code point |
| [`write_ranges`]`(&mut impl Write, ranges)` | Writes ranges as a comma separated list of inclusive ranges, as accepted by font subsetting tools |
| `merge_contiguous_ranges(Vec<Range<u32>>)` | With the `alloc` feature, the merged ranges as a `Vec` |
| `format_range(&Range<u32>)`, `format_ranges(ranges)` | With the `alloc` feature, the output of `write_range` and [`write_ranges`] as a `String` |

Numbers are written in decimal.

```rust
use embedded_i18n::{get_locale_ranges, merge_contiguous_ranges_in_place, write_ranges};

let mut ranges = [0..0, 0..0, 0..0, 0..0];
ranges.clone_from_slice(get_locale_ranges("ja").unwrap());
let length = merge_contiguous_ranges_in_place(&mut ranges);

let mut output = String::new();
write_ranges(&mut output, &ranges[..length])?;
assert_eq!(output, "32-126,12352-12543,19968-40959");
# Ok::<(), core::fmt::Error>(())
```

## Supported Locales

`en`, `fr`, `de`, `es`, `it`, `pt`, `pt-BR`, `nl`, `sv`, `no`, `da`, `pl`, `cs`, `sk`, `hu`, `ro`, `tr`, `ru`, `uk`, `be`, `el`, `ar`, `he`, `ja`, `zh`, `zh-CN`, `zh-TW`, `ko`, `hi`, `th` and `vi`.

The names are matched exactly (`fr`, not `fr-CA` or `FR`).

## Range Constants

The ranges are whole Unicode blocks, as half-open `Range<u32>` constants:

| Constant | Code points |
| --- | --- |
| [`BASIC_LATIN`] | `0x20..0x7F` (printable ASCII) |
| [`LATIN_1_SUPPLEMENT`] | `0xA0..0x100` |
| `LATIN_EXTENDED_A` | `0x100..0x180` |
| `LATIN_EXTENDED_B` | `0x180..0x250` |
| `LATIN_EXTENDED_ADDITIONAL` | `0x1E00..0x1F00` |
| `VIETNAMESE_EXTENSIONS` | `0x1EA0..0x1F00` |
| `GREEK` | `0x370..0x400` |
| `GREEK_EXTENDED` | `0x1F00..0x2000` |
| [`CYRILLIC`] | `0x400..0x500` |
| `CYRILLIC_SUPPLEMENT` | `0x500..0x530` |
| `HEBREW` | `0x590..0x600` |
| `ARABIC` | `0x600..0x700` |
| `ARABIC_SUPPLEMENT` | `0x750..0x780` |
| `DEVANAGARI` | `0x900..0x980` |
| `THAI` | `0xE00..0xE80` |
| `HIRAGANA` | `0x3040..0x30A0` |
| `KATAKANA` | `0x30A0..0x3100` |
| `CJK_UNIFIED_IDEOGRAPHS` | `0x4E00..0xA000` |
| [`HANGUL_SYLLABLES`] | `0xAC00..0xD7B0` |

They cover every letter of the language, but also code points it does not use.

```rust
use embedded_i18n::{get_locale_ranges, BASIC_LATIN, HIRAGANA, KATAKANA, CJK_UNIFIED_IDEOGRAPHS};

assert_eq!(get_locale_ranges("en"), Some(&[BASIC_LATIN][..]));
assert_eq!(
    get_locale_ranges("ja"),
    Some(&[BASIC_LATIN, HIRAGANA, KATAKANA, CJK_UNIFIED_IDEOGRAPHS][..])
);
assert_eq!(get_locale_ranges("xx"), None);
```

[`get_locale_ranges`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.get_locale_ranges.html
[`merge_contiguous_ranges_in_place`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.merge_contiguous_ranges_in_place.html
[`write_ranges`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.write_ranges.html
[`BASIC_LATIN`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/constant.BASIC_LATIN.html
[`LATIN_1_SUPPLEMENT`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/constant.LATIN_1_SUPPLEMENT.html
[`CYRILLIC`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/constant.CYRILLIC.html
[`HANGUL_SYLLABLES`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/constant.HANGUL_SYLLABLES.html
