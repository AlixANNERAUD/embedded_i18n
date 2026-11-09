# Unicode Ranges

Embedded fonts are usually subset to the characters a firmware can display, since a full CJK font does not fit in flash.
[`get_locale_ranges`] returns the Unicode blocks needed to render a language, to feed font conversion tools (LVGL's `lv_font_conv`, `fontTools`' `pyftsubset`, u8g2's `bdfconv`...):

```rust
use embedded_i18n::{get_locale_ranges, BASIC_LATIN, HIRAGANA, KATAKANA, CJK_UNIFIED_IDEOGRAPHS};

assert_eq!(get_locale_ranges("en"), Some(&[BASIC_LATIN][..]));
assert_eq!(
    get_locale_ranges("ja"),
    Some(&[BASIC_LATIN, HIRAGANA, KATAKANA, CJK_UNIFIED_IDEOGRAPHS][..])
);
assert_eq!(get_locale_ranges("xx"), None);
```

Supported locales: `en`, `fr`, `de`, `es`, `it`, `pt`, `nl`, `sv`, `no`, `da`, `pl`, `cs`, `sk`, `hu`, `ro`, `tr`, `ru`, `uk`, `be`, `el`, `ar`, `he`, `ja`, `zh`, `zh-CN`, `zh-TW`, `ko`, `hi`, `th` and `vi`.
The names are matched exactly (`fr`, not `fr-CA` or `FR`).

The ranges are whole Unicode blocks, as half-open `Range<u32>` constants ([`BASIC_LATIN`], [`LATIN_1_SUPPLEMENT`], [`CYRILLIC`], [`HANGUL_SYLLABLES`]...).
They cover every letter of the language, but also unused code points: to subset a font to the characters actually used, collect the characters of the locale files instead.

## Formatting for Font Tools

[`merge_contiguous_ranges_in_place`] sorts ranges and merges the overlapping and adjacent ones, and [`write_ranges`] writes them as a comma separated list of inclusive ranges, both without allocation:

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

Hiragana and Katakana are adjacent, so they are merged into a single range.
With the `alloc` feature, `merge_contiguous_ranges`, `format_range` and `format_ranges` return a `Vec` or a `String` instead.

## In a Build Script

The ranges are typically needed when converting fonts, before compiling the firmware.
With the `std` feature in `[build-dependencies]`, a build script can generate the font for the build locale:

```rust,no_run
use embedded_i18n::{format_ranges, get_locale_ranges, merge_contiguous_ranges};

fn main() {
    println!("cargo:rerun-if-env-changed=EMBEDDED_I18N_LOCALE");
    let locale = std::env::var("EMBEDDED_I18N_LOCALE").unwrap_or_else(|_| "en".to_string());
    let ranges = get_locale_ranges(&locale).expect("unsupported locale");
    let ranges = merge_contiguous_ranges(ranges.to_vec());
    let ranges = format_ranges(&ranges);
    // e.g. run `lv_font_conv --range <ranges> ...` with std::process::Command
    println!("cargo:warning=font ranges for {locale}: {ranges}");
}
```

The build script cannot see the arguments of [`i18n!`]: it reads the same `EMBEDDED_I18N_LOCALE` variable as the [override][guide-configuration], or its own setting.

[guide-configuration]: configuration.md
[`i18n!`]: configuration.md
[`get_locale_ranges`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.get_locale_ranges.html
[`merge_contiguous_ranges_in_place`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.merge_contiguous_ranges_in_place.html
[`write_ranges`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.write_ranges.html
[`BASIC_LATIN`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/constant.BASIC_LATIN.html
[`LATIN_1_SUPPLEMENT`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/constant.LATIN_1_SUPPLEMENT.html
[`CYRILLIC`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/constant.CYRILLIC.html
[`HANGUL_SYLLABLES`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/constant.HANGUL_SYLLABLES.html
