#![cfg(feature = "backend-json")]

embedded_i18n::i18n!(
    "tests/locales/rtl",
    default_locale = "ar",
    fallback = "ar",
    bidi_isolation = true
);

use embedded_i18n::t;

// Values interpolated into a right-to-left translation are wrapped in U+2068
// FIRST STRONG ISOLATE and U+2069 POP DIRECTIONAL ISOLATE, with both
// placeholder syntaxes.

#[test]
fn isolates_rust_i18n_placeholders() {
    assert_eq!(
        t!("hello", name = "Ada").to_string(),
        "مرحبا \u{2068}Ada\u{2069}"
    );
}

#[test]
fn keeps_plain_braces_of_rust_i18n_syntax_as_text() {
    assert_eq!(
        t!("braces", name = "Ada").to_string(),
        "\u{2068}Ada\u{2069} {raw}"
    );
}

#[test]
fn isolates_rust_placeholders() {
    assert_eq!(
        t!("rust", name = "Ada").to_string(),
        "مرحبا \u{2068}Ada\u{2069}"
    );
}

#[test]
fn isolates_plural_counts() {
    assert_eq!(
        t!("files", count = 3).to_string(),
        "\u{2068}3\u{2069} ملفات"
    );
}
