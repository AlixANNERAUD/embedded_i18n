embedded_i18n::i18n!("locales", default_locale = "fr", strict = true);

use embedded_i18n::{available_locales, locale, t};

// Strict mode forbids falling back for messages missing from the primary
// locale: `t!("missing_in_french")` would not compile here. These keys all
// exist in French.

#[test]
fn translates_into_the_default_locale() {
    assert_eq!(locale!(), "fr");
    assert_eq!(available_locales!(), ["fr"]);
    assert_eq!(t!("greeting"), "Bonjour");
    assert_eq!(t!("farewell"), "Au revoir");
}

#[test]
fn strict_mode_covers_arguments_plurals_context_and_select() {
    assert_eq!(t!("with_arg", value = 3).to_string(), "Valeur : 3");
    assert_eq!(t!("files", count = 1).to_string(), "1 fichier");
    assert_eq!(t!("files", count = 2).to_string(), "2 fichiers");
    assert_eq!(t!("Open", context = "menu"), "Ouvrir");
    assert_eq!(t!("replied", select = "female"), "Elle a répondu");
}

#[test]
fn strict_mode_is_const() {
    const GREETING: &str = t!("greeting");
    assert_eq!(GREETING, "Bonjour");
}
