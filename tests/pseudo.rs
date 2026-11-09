embedded_i18n::i18n!("locales", default_locale = "pseudo");

use embedded_i18n::{locale, t};

// The pseudo locale is generated from the fallback chain (`en`, from the
// package metadata): letters are accented, the text is padded with `~` and
// bracketed, and placeholders are left untouched.

#[test]
fn pseudo_localizes_the_fallback() {
    assert_eq!(locale!(), "pseudo");
    assert_eq!(t!("greeting"), "[Hélló ~~]");
    assert_eq!(t!("farewell"), "[Góódbýé ~~~]");
}

#[test]
fn pseudo_keeps_placeholders() {
    assert_eq!(t!("with_arg", value = 42).to_string(), "[Válúé: 42 ~~]");
    assert_eq!(
        t!("with_named_args", name = "Ada", count = 3).to_string(),
        "[Hélló Ada, ýóú hávé 3 mésságés ~~~~~~~]"
    );
}

#[test]
fn pseudo_keeps_plural_forms() {
    assert_eq!(t!("files", count = 1).to_string(), "[1 fílé ~~]");
    assert_eq!(t!("files", count = 2).to_string(), "[2 fílés ~~]");
}

#[test]
fn pseudo_is_complete() {
    assert_eq!(t!("missing_in_french"), "[Óñlý íñ Éñglísh ~~~~~]");
}
