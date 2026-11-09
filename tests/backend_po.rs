#![cfg(all(feature = "backend-po", not(feature = "backend-json")))]

embedded_i18n::i18n!("locales", available_locales = ["en", "fr"]);

use embedded_i18n::{locale, t};

// With only `backend-po`, `locales/*.po` are the translations read.

#[test]
fn reads_po_files() {
    assert_eq!(t!("greeting", locale = "en"), "Hello");
    assert_eq!(t!("greeting", locale = "fr"), "Bonjour");
    assert_eq!(locale!(), "en");
}

#[test]
fn po_arguments() {
    assert_eq!(
        t!("with_arg", value = 42, locale = "fr").to_string(),
        "Valeur : 42"
    );
    assert_eq!(
        t!("with_named_args", name = "Ada", count = 3, locale = "en").to_string(),
        "Hello Ada, you have 3 messages"
    );
}

#[test]
fn po_context() {
    assert_eq!(t!("Open", context = "menu", locale = "en"), "Open (menu)");
    assert_eq!(t!("Open", context = "state", locale = "fr"), "Ouvert");
}

#[test]
fn po_plural_forms_follow_the_header() {
    // English: nplurals=2; plural=(n != 1)
    assert_eq!(t!("files", count = 0, locale = "en").to_string(), "0 files");
    assert_eq!(t!("files", count = 1, locale = "en").to_string(), "1 file");
    // French: nplurals=2; plural=(n > 1)
    assert_eq!(
        t!("files", count = 0, locale = "fr").to_string(),
        "0 fichier"
    );
    assert_eq!(
        t!("files", count = 2, locale = "fr").to_string(),
        "2 fichiers"
    );
    assert_eq!(
        t!("files", context = "menu", count = 1, locale = "fr").to_string(),
        "Fichier ouvert"
    );
}

#[test]
fn po_select_variants() {
    assert_eq!(
        t!("replied", select = "female", locale = "fr"),
        "Elle a répondu"
    );
    assert_eq!(
        t!("replied", select = "unknown", locale = "en"),
        "They replied"
    );
}

#[test]
fn po_falls_back_to_the_other_locale() {
    assert_eq!(t!("missing_in_french", locale = "fr"), "Only in English");
}
