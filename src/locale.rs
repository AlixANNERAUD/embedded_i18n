//! Locale identifiers, the global current locale and the `Display` wrapper of `t!`.

use core::fmt;
use core::sync::atomic::{AtomicU32, Ordering};

/// Language value of tags that are not a 1 to 3 letter language code.
const OTHER_LANGUAGE: u32 = 0x7FFF;

const FNV_OFFSET: u32 = 0x811C_9DC5;
const FNV_PRIME: u32 = 0x0100_0193;

/// Normalizes one byte of a tag: ASCII lowercase, `_` as `-`.
const fn normalize(byte: u8) -> u8 {
    match byte {
        b'_' => b'-',
        byte => byte.to_ascii_lowercase(),
    }
}

/// Hashes the normalized bytes of `bytes[start..end]` with FNV-1a, folded to
/// 17 bits and never zero.
const fn hash(bytes: &[u8], start: usize, end: usize) -> u32 {
    if start >= end {
        return 0;
    }
    let mut h = FNV_OFFSET;
    let mut index = start;
    while index < end {
        h ^= normalize(bytes[index]) as u32;
        h = h.wrapping_mul(FNV_PRIME);
        index += 1;
    }
    let low = (h ^ (h >> 17)) & 0x1_FFFF;
    if low == 0 { 1 } else { low }
}

/// Packs a locale tag into its identifier.
const fn pack(tag: &str) -> u32 {
    let bytes = tag.as_bytes();
    let mut start = 0;
    let mut end = bytes.len();
    while start < end && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    if start == end {
        return 0;
    }

    let mut separator = start;
    while separator < end && normalize(bytes[separator]) != b'-' {
        separator += 1;
    }

    let length = separator - start;
    let mut language = 0u32;
    let mut is_language = length >= 1 && length <= 3;
    let mut index = start;
    while is_language && index < separator {
        let byte = normalize(bytes[index]);
        if byte.is_ascii_lowercase() {
            language = language * 32 + (byte - b'a' + 1) as u32;
        } else {
            is_language = false;
        }
        index += 1;
    }

    if is_language {
        let rest = if separator < end { separator + 1 } else { end };
        (language << 17) | hash(bytes, rest, end)
    } else {
        (OTHER_LANGUAGE << 17) | hash(bytes, start, end)
    }
}

/// A locale, packed into a 32-bit identifier.
///
/// The identifier is computed from the locale tag by a `const fn`, so tags
/// cost nothing at runtime when converted in `const` context. The tag is
/// normalized first (surrounding ASCII whitespace trimmed, ASCII lowercase,
/// `_` as `-`), so `fr_CA`, `FR-ca` and ` fr-CA ` are the same locale.
///
/// The top 15 bits hold the language when it is 1 to 3 ASCII letters (5 bits
/// per letter, `a` is 1); the low 17 bits hold a hash of the rest of the tag
/// (region, script, variants), or `0` when there is none. Tags that do not
/// start with a 1 to 3 letter language (`pseudo`) use the language value
/// `0x7FFF` and hash the whole tag. [`Locale::NONE`] (identifier `0`) is the
/// empty tag.
///
/// ```
/// use embedded_i18n::Locale;
///
/// assert_eq!(Locale::new("fr_CA"), Locale::new("FR-ca"));
/// assert_eq!(Locale::new("en").id(), 0x015C_0000);
/// assert_eq!(Locale::NONE.id(), 0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Locale(u32);

impl Locale {
    /// The empty locale, identifier `0`: no locale was selected.
    pub const NONE: Locale = Locale(0);

    /// Packs a locale tag into a [`Locale`]. Usable in `const` context.
    pub const fn new(tag: &str) -> Locale {
        Locale(pack(tag))
    }

    /// Returns the packed 32-bit identifier.
    pub const fn id(self) -> u32 {
        self.0
    }
}

impl From<&str> for Locale {
    fn from(tag: &str) -> Locale {
        Locale::new(tag)
    }
}

static CURRENT: AtomicU32 = AtomicU32::new(0);

/// Selects the current locale used by [`t!`](crate::t) and [`locale!`](crate::locale).
///
/// The locale is stored as a [`Locale`] identifier in an atomic with a plain
/// store, so this works on targets without compare-and-swap and needs no
/// allocation. An empty tag restores the unset state.
///
/// ```
/// embedded_i18n::set_locale("fr");
/// ```
pub fn set_locale(locale: &str) {
    CURRENT.store(Locale::new(locale).id(), Ordering::Relaxed);
}

/// Returns the identifier of the current locale, `0` when it was never set.
#[doc(hidden)]
pub fn __current_locale() -> u32 {
    CURRENT.load(Ordering::Relaxed)
}

/// Formats with a closure: the value of a `t!` call with arguments.
#[doc(hidden)]
pub struct __FormatWith<F>(pub F);

impl<F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result> fmt::Display for __FormatWith<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.0)(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_reference_tags() {
        assert_eq!(Locale::new("en").id(), 0x015C_0000);
        assert_eq!(Locale::new("fr").id(), 0x01A4_0000);
        assert_eq!(Locale::new("de").id(), 0x010A_0000);
        assert_eq!(Locale::new("fr-CA").id(), 0x01A5_B85D);
        assert_eq!(Locale::new("fr-FR").id(), 0x01A4_FD60);
        assert_eq!(Locale::new("zh-Hant").id(), 0x0690_220D);
        assert_eq!(Locale::new("zh-Hans").id(), 0x0690_3E88);
        assert_eq!(Locale::new("pt-BR").id(), 0x0429_E320);
        assert_eq!(Locale::new("pseudo").id(), 0xFFFE_7D15);
        assert_eq!(Locale::new("x-a").id(), 0x0030_5B2A);
        assert_eq!(Locale::new(""), Locale::NONE);
    }

    #[test]
    fn normalizes_before_packing() {
        assert_eq!(Locale::new("FR_ca"), Locale::new("fr-CA"));
        assert_eq!(Locale::new(" fr-ca "), Locale::new("fr-CA"));
        assert_eq!(Locale::from("pt_br"), Locale::new("pt-BR"));
    }

    #[test]
    fn global_locale_round_trips() {
        // single test touching the global, so no lock is needed
        assert_eq!(__current_locale(), 0);
        set_locale("fr_CA");
        assert_eq!(__current_locale(), 0x01A5_B85D);
        set_locale("");
        assert_eq!(__current_locale(), 0);
    }

    const _: () = assert!(Locale::new("en").id() == 0x015C_0000); // const fn
}
