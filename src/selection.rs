use core::fmt;
use core::sync::atomic::{AtomicU8, Ordering};

const fn embedded_locales() -> &'static str {
    match option_env!("EMBEDDED_I18N_LOCALES") {
        Some(locales) => locales,
        None => "",
    }
}

const fn entry(list: &'static str, wanted: usize) -> Option<&'static str> {
    let bytes = list.as_bytes();
    let mut start = 0;
    let mut index = 0;
    let mut current = 0;
    while index <= bytes.len() {
        if index == bytes.len() || bytes[index] == b',' {
            let (head, _) = bytes.split_at(index);
            let (_, entry) = head.split_at(start);
            let entry = entry.trim_ascii();
            if !entry.is_empty() {
                if current == wanted {
                    return match core::str::from_utf8(entry) {
                        Ok(entry) => Some(entry),
                        Err(_) => None,
                    };
                }
                current += 1;
            }
            start = index + 1;
        }
        index += 1;
    }
    None
}

const fn count(list: &'static str) -> usize {
    let mut count = 0;
    while entry(list, count).is_some() {
        count += 1;
    }
    count
}

/// Number of embedded locales: the length of `EMBEDDED_I18N_LOCALES`, or 1.
pub const LOCALE_COUNT: usize = match count(embedded_locales()) {
    0 => 1,
    count => count,
};

const _: () = assert!(LOCALE_COUNT <= u8::MAX as usize);

/// Embedded locales, in the order of `EMBEDDED_I18N_LOCALES`, or the build
/// locale alone.
pub const LOCALES: [&str; LOCALE_COUNT] = {
    let mut locales = [crate::get_locale(); LOCALE_COUNT];
    let mut index = 0;
    while index < LOCALE_COUNT {
        if let Some(locale) = entry(embedded_locales(), index) {
            locales[index] = locale;
        }
        index += 1;
    }
    locales
};

const fn same_locale(left: &str, right: &str) -> bool {
    let left = left.trim_ascii().as_bytes();
    let right = right.trim_ascii().as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        let a = if left[index] == b'_' {
            b'-'
        } else {
            left[index]
        };
        let b = if right[index] == b'_' {
            b'-'
        } else {
            right[index]
        };
        if !a.eq_ignore_ascii_case(&b) {
            return false;
        }
        index += 1;
    }
    true
}

/// Returns the index in [`LOCALES`] of a locale, compared case-insensitively
/// with `-` and `_` treated alike.
pub const fn find_locale(locale: &str) -> Option<usize> {
    let mut index = 0;
    while index < LOCALE_COUNT {
        if same_locale(LOCALES[index], locale) {
            return Some(index);
        }
        index += 1;
    }
    None
}

/// Index of the locale active at startup: `EMBEDDED_I18N_LOCALE` if embedded,
/// otherwise the first embedded locale.
pub const DEFAULT_LOCALE_INDEX: usize = match find_locale(crate::get_locale()) {
    Some(index) => index,
    None => 0,
};

static CURRENT: AtomicU8 = AtomicU8::new(DEFAULT_LOCALE_INDEX as u8);

/// Error returned when selecting a locale that is not embedded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownLocale;

impl fmt::Display for UnknownLocale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("locale is not embedded")
    }
}

/// Returns the index in [`LOCALES`] of the current locale.
#[inline]
pub fn locale_index() -> usize {
    let index = usize::from(CURRENT.load(Ordering::Relaxed));
    if index < LOCALE_COUNT { index } else { 0 }
}

/// Returns the current locale.
///
/// ```
/// assert_eq!(embedded_i18n::locale(), "en");
/// ```
pub fn locale() -> &'static str {
    LOCALES[locale_index()]
}

/// Selects the current locale by its index in [`LOCALES`].
pub fn set_locale_index(index: usize) -> Result<(), UnknownLocale> {
    if index >= LOCALE_COUNT {
        return Err(UnknownLocale);
    }
    CURRENT.store(index as u8, Ordering::Relaxed);
    Ok(())
}

/// Selects the current locale used by the translation macros.
///
/// With `EMBEDDED_I18N_LOCALES=en,fr,de`, every listed locale is embedded and
/// the macros look up the current one at runtime. The current locale is stored
/// in an atomic with plain loads and stores, so this works on targets without
/// compare-and-swap and needs no allocation. Without `EMBEDDED_I18N_LOCALES`,
/// only the build locale is accepted.
///
/// ```
/// use embedded_i18n::{set_locale, UnknownLocale};
///
/// assert_eq!(set_locale("en"), Ok(()));
/// assert_eq!(set_locale("tlh"), Err(UnknownLocale));
/// ```
pub fn set_locale(locale: &str) -> Result<(), UnknownLocale> {
    set_locale_index(find_locale(locale).ok_or(UnknownLocale)?)
}

/// Returns the index in [`LOCALES`] of the embedded locale that best serves a
/// requested one, such as a language tag from an `Accept-Language` header or a
/// user setting: the locale itself, then its parents (`fr-CA` for `fr-CA-x`,
/// then `fr`), then any embedded locale of the same language (`fr-FR` for
/// `fr`).
///
/// Pass the result to the `locale` option of the translation macros to
/// translate into it without changing the current locale, for example once per
/// request in a server.
///
/// ```
/// assert_eq!(embedded_i18n::match_locale("en-GB"), Some(0));
/// assert_eq!(embedded_i18n::match_locale("tlh"), None);
/// ```
pub fn match_locale(requested: &str) -> Option<usize> {
    best_match(&LOCALES, requested)
}

fn best_match(locales: &[&str], requested: &str) -> Option<usize> {
    let requested = requested.trim_ascii();
    if requested.is_empty() {
        return None;
    }
    let position = |wanted: &str| {
        locales
            .iter()
            .position(|locale| same_locale(locale, wanted))
    };
    let mut candidate = requested;
    loop {
        if let Some(index) = position(candidate) {
            return Some(index);
        }
        match candidate.rfind(['-', '_']) {
            Some(end) => candidate = &candidate[..end],
            None => break,
        }
    }
    locales
        .iter()
        .position(|locale| same_locale(language(locale), candidate))
}

fn language(locale: &str) -> &str {
    match locale.find(['-', '_']) {
        Some(end) => &locale[..end],
        None => locale,
    }
}

/// Clamps a locale index given to the `locale` option of the macros: an index
/// outside [`LOCALES`] selects the default locale.
#[doc(hidden)]
#[inline]
pub const fn __locale_index(index: usize) -> usize {
    if index < LOCALE_COUNT {
        index
    } else {
        DEFAULT_LOCALE_INDEX
    }
}

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
    fn parses_locale_lists() {
        assert_eq!(entry(" en, fr_CA ,,de ", 0), Some("en"));
        assert_eq!(entry(" en, fr_CA ,,de ", 1), Some("fr_CA"));
        assert_eq!(entry(" en, fr_CA ,,de ", 2), Some("de"));
        assert_eq!(entry(" en, fr_CA ,,de ", 3), None);
        assert_eq!(count(" en, fr_CA ,,de "), 3);
        assert_eq!(count(""), 0);
        assert_eq!(count(" , "), 0);
    }

    #[test]
    fn compares_locales_loosely() {
        assert!(same_locale("fr_CA", "FR-ca"));
        assert!(!same_locale("fr", "fr-CA"));
    }

    #[test]
    fn matches_requested_locales() {
        let locales = ["en", "fr-FR", "pt_BR", "zh-Hant"];
        assert_eq!(best_match(&locales, "fr-fr"), Some(1));
        assert_eq!(best_match(&locales, "en-GB"), Some(0));
        assert_eq!(best_match(&locales, "en-Latn-US"), Some(0));
        assert_eq!(best_match(&locales, "fr"), Some(1));
        assert_eq!(best_match(&locales, "fr-CA"), Some(1));
        assert_eq!(best_match(&locales, "pt-BR"), Some(2));
        assert_eq!(best_match(&locales, " zh-Hant-TW "), Some(3));
        assert_eq!(best_match(&locales, "de"), None);
        assert_eq!(best_match(&locales, ""), None);
        assert_eq!(best_match(&locales, "*"), None);
    }

    #[test]
    fn clamps_explicit_indexes() {
        assert_eq!(__locale_index(0), 0);
        assert_eq!(__locale_index(LOCALE_COUNT), DEFAULT_LOCALE_INDEX);
    }

    #[test]
    fn single_locale_by_default() {
        if LOCALE_COUNT != 1 {
            return;
        }
        assert_eq!(LOCALES[..], ["en"]);
        assert_eq!(locale_index(), 0);
        assert_eq!(locale(), "en");
        assert_eq!(set_locale("EN"), Ok(()));
        assert_eq!(set_locale("fr"), Err(UnknownLocale));
        assert_eq!(set_locale_index(1), Err(UnknownLocale));
        assert_eq!(locale(), "en");
    }
}
