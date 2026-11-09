const RIGHT_TO_LEFT_LANGUAGES: &[&str] = &[
    "ar", "arc", "ckb", "dv", "fa", "he", "iw", "ks", "ps", "sd", "syr", "ug", "ur", "yi",
];

const RIGHT_TO_LEFT_SCRIPTS: &[&str] = &["Arab", "Hebr"];

const fn is_separator(byte: u8) -> bool {
    byte == b'-' || byte == b'_'
}

const fn equals_ignore_case(bytes: &[u8], start: usize, end: usize, candidate: &str) -> bool {
    let candidate = candidate.as_bytes();
    if end - start != candidate.len() {
        return false;
    }
    let mut index = 0;
    while index < candidate.len() {
        if !bytes[start + index].eq_ignore_ascii_case(&candidate[index]) {
            return false;
        }
        index += 1;
    }
    true
}

const fn contains(list: &[&str], bytes: &[u8], start: usize, end: usize) -> bool {
    let mut index = 0;
    while index < list.len() {
        if equals_ignore_case(bytes, start, end, list[index]) {
            return true;
        }
        index += 1;
    }
    false
}

/// Returns whether a locale is written right to left.
///
/// Recognizes right-to-left languages (`ar`, `he`, `fa`, `ur`, `ps`, `sd`, `ug`,
/// `yi`...) and the `Arab` and `Hebr` script subtags, case-insensitively, with
/// `-` or `_` separators. Usable in `const` context.
///
/// ```
/// use embedded_i18n::is_right_to_left;
///
/// assert!(is_right_to_left("ar-EG"));
/// assert!(is_right_to_left("pa_Arab"));
/// assert!(!is_right_to_left("fr-CA"));
/// ```
pub const fn is_right_to_left(locale: &str) -> bool {
    let bytes = locale.as_bytes();
    let mut start = 0;
    let mut end = 0;
    let mut first = true;
    while end <= bytes.len() {
        if end == bytes.len() || is_separator(bytes[end]) {
            if first {
                if contains(RIGHT_TO_LEFT_LANGUAGES, bytes, start, end) {
                    return true;
                }
                first = false;
            } else if contains(RIGHT_TO_LEFT_SCRIPTS, bytes, start, end) {
                return true;
            }
            start = end + 1;
        }
        end += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn right_to_left_languages() {
        for locale in ["ar", "AR", "ar-EG", "he", "fa_IR", "ur", "yi", "ckb"] {
            assert!(is_right_to_left(locale), "{}", locale);
        }
    }

    #[test]
    fn right_to_left_scripts() {
        for locale in ["ku-Arab", "pa-arab-PK", "yi-Hebr"] {
            assert!(is_right_to_left(locale), "{}", locale);
        }
    }

    #[test]
    fn left_to_right_locales() {
        for locale in ["", "en", "fr-CA", "arn", "a", "ku", "zh-Hant-TW", "de-AR"] {
            assert!(!is_right_to_left(locale), "{}", locale);
        }
    }

    #[test]
    fn usable_in_const() {
        const { assert!(is_right_to_left("ar-SA")) };
    }
}
