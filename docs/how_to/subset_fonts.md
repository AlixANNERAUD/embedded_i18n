# How to Subset Fonts

Embedded fonts are usually subset to the characters a firmware can display, since a full CJK font does not fit in flash.
This guide gets the Unicode ranges of a language and passes them to a font conversion tool, such as LVGL's `lv_font_conv`, `fontTools`' `pyftsubset` or u8g2's `bdfconv`.

## Get the Ranges of a Language

[`get_locale_ranges`] returns the Unicode blocks needed to render a language:

```rust
use embedded_i18n::{get_locale_ranges, BASIC_LATIN, HIRAGANA, KATAKANA, CJK_UNIFIED_IDEOGRAPHS};

assert_eq!(get_locale_ranges("en"), Some(&[BASIC_LATIN][..]));
assert_eq!(
    get_locale_ranges("ja"),
    Some(&[BASIC_LATIN, HIRAGANA, KATAKANA, CJK_UNIFIED_IDEOGRAPHS][..])
);
assert_eq!(get_locale_ranges("xx"), None);
```

The names are matched exactly: pass `fr`, not `fr-CA` or `FR`.

## Write Them for a Font Tool

Merge the ranges with [`merge_contiguous_ranges_in_place`], then write them with [`write_ranges`], as a comma separated list of inclusive ranges, without allocation:

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

## Generate the Font in a Build Script

The ranges are needed when converting fonts, before compiling the firmware.
Add `embedded-i18n` with the `std` feature to `[build-dependencies]`, and read the build locale in the build script:

```toml
[build-dependencies]
embedded-i18n = { version = "0.2", features = ["std"] }
```

```rust,ignore
// build.rs
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

The build script cannot see the arguments of [`i18n!`]: it reads the same `EMBEDDED_I18N_LOCALE` variable as the [build override][how-to-build-per-language], or its own setting.

## Keep the Font Small

The ranges are whole Unicode blocks: they cover every letter of the language, but also code points it never uses.
To subset a font to the characters actually displayed, collect the characters of your locale files instead, and pass them to the tool as text.

See [Unicode Ranges][reference-unicode-ranges] for the supported languages and the range constants.

[how-to-build-per-language]: build_one_firmware_per_language.md
[reference-unicode-ranges]: ../reference/unicode_ranges.md
[`i18n!`]: ../reference/configuration.md
[`get_locale_ranges`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.get_locale_ranges.html
[`merge_contiguous_ranges_in_place`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.merge_contiguous_ranges_in_place.html
[`write_ranges`]: https://docs.rs/embedded-i18n/latest/embedded_i18n/fn.write_ranges.html
