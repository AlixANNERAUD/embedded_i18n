//! Packed locale identifiers and per-crate locale resolution.
//!
//! A locale tag is packed into a `u32` (15 bits of language, 17 bits of hash
//! of the rest of the tag). This must match, bit for bit, the algorithm of
//! `embedded_i18n::Locale::new` in the runtime crate.

use proc_macro2::{Literal, TokenStream as TokenStream2};
use quote::quote;

/// Identifier meaning "no locale set".
pub const NONE: u32 = 0;

/// Language part (bits 31..17) used for tags without a 1–3 letter language.
const RESERVED_LANGUAGE: u32 = 0x7FFF;

/// Packs a locale tag into its runtime identifier.
///
/// The tag is trimmed, ASCII-lowercased and `_` becomes `-`; an empty tag is
/// [`NONE`]. A leading 1–3 letter `a`–`z` language is stored in base 32 in the
/// high bits and the rest of the tag is hashed (FNV-1a) into the low 17 bits;
/// any other tag is hashed whole under the reserved language `0x7FFF`.
pub fn pack(tag: &str) -> u32 {
    let normalized: Vec<u8> = tag
        .trim_matches(|c: char| c.is_ascii_whitespace())
        .bytes()
        .map(|b| match b {
            b'_' => b'-',
            b => b.to_ascii_lowercase(),
        })
        .collect();
    if normalized.is_empty() {
        return NONE;
    }
    let (language, rest) = match normalized.iter().position(|&b| b == b'-') {
        Some(at) => (&normalized[..at], &normalized[at + 1..]),
        None => (&normalized[..], &normalized[normalized.len()..]),
    };
    let (lang, hashed) =
        if (1..=3).contains(&language.len()) && language.iter().all(u8::is_ascii_lowercase) {
            let lang = language
                .iter()
                .fold(0u32, |lang, &c| lang * 32 + u32::from(c - b'a' + 1));
            (lang, rest)
        } else {
            (RESERVED_LANGUAGE, &normalized[..])
        };
    let low = if hashed.is_empty() {
        0
    } else {
        let hash = hashed.iter().fold(0x811C_9DC5u32, |h, &b| {
            (h ^ u32::from(b)).wrapping_mul(0x0100_0193)
        });
        match (hash ^ (hash >> 17)) & 0x1_FFFF {
            0 => 1,
            low => low,
        }
    };
    (lang << 17) | low
}

/// Index of the locale a requested tag resolves to: the locale with the same
/// identifier, else a locale of the same language (preferring the language-only
/// one, then the first in configuration order), else `None`.
pub fn resolve(locales: &[String], tag: &str) -> Option<usize> {
    let id = pack(tag);
    if id == NONE {
        return None;
    }
    if let Some(index) = locales.iter().position(|locale| pack(locale) == id) {
        return Some(index);
    }
    let language = id >> 17;
    if language == RESERVED_LANGUAGE {
        return None;
    }
    language_match(locales, language)
}

/// Index of the language-only locale of `language`, else of its first locale.
fn language_match(locales: &[String], language: u32) -> Option<usize> {
    let mut first = None;
    for (index, locale) in locales.iter().enumerate() {
        let id = pack(locale);
        if id >> 17 != language {
            continue;
        }
        if id & 0x1_FFFF == 0 {
            return Some(index);
        }
        first.get_or_insert(index);
    }
    first
}

/// Fails when two available locales pack to the same identifier.
pub fn check_distinct(locales: &[String]) -> Result<(), String> {
    for (index, a) in locales.iter().enumerate() {
        let id = pack(a);
        if let Some(b) = locales[index + 1..].iter().find(|b| pack(b) == id) {
            return Err(format!(
                "available locales \"{a}\" and \"{b}\" have the same runtime identifier"
            ));
        }
    }
    Ok(())
}

/// Generates `__embedded_i18n_resolve`, mapping a runtime identifier to the
/// index of a configured locale: exact match, then same language, then
/// `default`.
pub fn resolver(locales: &[String], default: usize) -> TokenStream2 {
    let ids: Vec<u32> = locales.iter().map(|locale| pack(locale)).collect();
    let exact_ids = ids.iter().map(|&id| Literal::u32_suffixed(id));
    let exact_indexes = (0..ids.len()).map(Literal::usize_unsuffixed);

    let mut languages: Vec<u32> = Vec::new();
    for &id in &ids {
        let language = id >> 17;
        if language != NONE && language != RESERVED_LANGUAGE && !languages.contains(&language) {
            languages.push(language);
        }
    }
    let language_ids = languages
        .iter()
        .map(|&language| Literal::u32_suffixed(language));
    let language_indexes = languages.iter().map(|&language| {
        Literal::usize_unsuffixed(
            language_match(locales, language).expect("the language comes from a locale"),
        )
    });
    let default = Literal::usize_unsuffixed(default);

    quote! {
        #[doc(hidden)]
        #[allow(dead_code)]
        fn __embedded_i18n_resolve(id: u32) -> usize {
            match id {
                #(#exact_ids => #exact_indexes,)*
                _ => match id >> 17 {
                    #(#language_ids => #language_indexes,)*
                    _ => #default,
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn packs_like_the_runtime() {
        assert_eq!(pack("en"), 0x015C_0000);
        assert_eq!(pack("fr"), 0x01A4_0000);
        assert_eq!(pack("de"), 0x010A_0000);
        assert_eq!(pack("fr-CA"), 0x01A5_B85D);
        assert_eq!(pack("fr-FR"), 0x01A4_FD60);
        assert_eq!(pack("zh-Hant"), 0x0690_220D);
        assert_eq!(pack("zh-Hans"), 0x0690_3E88);
        assert_eq!(pack("pt-BR"), 0x0429_E320);
        assert_eq!(pack("pseudo"), 0xFFFE_7D15);
        assert_eq!(pack("x-a"), 0x0030_5B2A);
        assert_eq!(pack(""), NONE);
        assert_eq!(pack("FR_ca"), 0x01A5_B85D);
    }

    #[test]
    fn resolves_exact_then_language() {
        let locales = ["en", "fr-FR", "fr", "zh-Hant"].map(String::from);
        assert_eq!(resolve(&locales, "fr-FR"), Some(1));
        assert_eq!(resolve(&locales, "fr-CA"), Some(2)); // language-only locale preferred
        assert_eq!(resolve(&locales, "zh-Hans"), Some(3)); // documented script limitation
        assert_eq!(resolve(&locales, "de"), None);
        assert_eq!(resolve(&locales, ""), None);
        let regional = ["en", "fr-FR", "fr-CA"].map(String::from);
        assert_eq!(resolve(&regional, "fr"), Some(1)); // first locale of the language
    }

    #[test]
    fn rejects_identical_identifiers() {
        let locales = ["fr-CA", "FR_ca"].map(String::from);
        assert_eq!(
            check_distinct(&locales),
            Err(
                "available locales \"fr-CA\" and \"FR_ca\" have the same runtime identifier"
                    .to_string()
            )
        );
        let distinct = ["en", "fr", "fr-CA"].map(String::from);
        assert_eq!(check_distinct(&distinct), Ok(()));
    }

    #[test]
    fn generates_the_resolver() {
        let locales = ["en", "fr"].map(String::from);
        let expected = quote! {
            #[doc(hidden)]
            #[allow(dead_code)]
            fn __embedded_i18n_resolve(id: u32) -> usize {
                match id {
                    22806528u32 => 0,
                    27525120u32 => 1,
                    _ => match id >> 17 {
                        174u32 => 0,
                        210u32 => 1,
                        _ => 0,
                    },
                }
            }
        };
        assert_eq!(resolver(&locales, 0).to_string(), expected.to_string());
    }

    #[test]
    fn resolver_skips_the_reserved_language_and_prefers_language_only() {
        let locales = ["pseudo", "fr-FR", "fr"].map(String::from);
        let expected = quote! {
            #[doc(hidden)]
            #[allow(dead_code)]
            fn __embedded_i18n_resolve(id: u32) -> usize {
                match id {
                    4294868245u32 => 0,
                    27589984u32 => 1,
                    27525120u32 => 2,
                    _ => match id >> 17 {
                        210u32 => 2,
                        _ => 1,
                    },
                }
            }
        };
        assert_eq!(resolver(&locales, 1).to_string(), expected.to_string());
    }
}
