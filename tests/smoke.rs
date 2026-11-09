embedded_i18n::i18n!("locales", available_locales = ["en", "fr"]);
use embedded_i18n::t;

#[test]
fn translates_with_the_new_api() {
    assert_eq!(t!("greeting"), "Hello");
    assert_eq!(t!("greeting", locale = "fr"), "Bonjour");
    assert_eq!(t!("files", count = 3).to_string(), "3 files");
    assert_eq!(t!("files", count = 2 + 3).to_string(), "5 files");
}
