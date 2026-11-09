embedded_i18n::i18n!("locales", available_locales = ["en", "fr"]);

use core::fmt::Write;

use embedded_i18n::{Locale, available_locales, locale, set_locale, t};

struct Buffer {
    bytes: [u8; 64],
    length: usize,
}

impl Write for Buffer {
    fn write_str(&mut self, value: &str) -> core::fmt::Result {
        let end = self.length + value.len();
        if end > self.bytes.len() {
            return Err(core::fmt::Error);
        }
        self.bytes[self.length..end].copy_from_slice(value.as_bytes());
        self.length = end;
        Ok(())
    }
}

fn written(write: impl FnOnce(&mut Buffer) -> core::fmt::Result) -> Buffer {
    let mut buffer = Buffer {
        bytes: [0; 64],
        length: 0,
    };
    write(&mut buffer).unwrap();
    buffer
}

// The only test of this file that touches the global locale: tests run in
// parallel threads and the locale is shared by the whole program.
#[test]
fn switches_locale_at_runtime() {
    assert_eq!(available_locales!(), &["en", "fr"]);

    assert_eq!(locale!(), "en");
    assert_eq!(t!("greeting"), "Hello");

    set_locale("fr");
    assert_eq!(locale!(), "fr");
    assert_eq!(t!("greeting"), "Bonjour");
    assert_eq!(t!(c"greeting").to_bytes(), b"Bonjour");
    assert_eq!(t!("missing_in_french"), "Only in English");
    assert_eq!(t!("files", count = 0).to_string(), "0 fichier");
    assert_eq!(t!("replied", select = "female"), "Elle a répondu");
    let buffer = written(|buffer| write!(buffer, "{}", t!("with_arg", value = 42)));
    assert_eq!(&buffer.bytes[..buffer.length], "Valeur : 42".as_bytes());
    let buffer = written(|buffer| write!(buffer, "{}", t!("with_arg", value = 7)));
    assert_eq!(&buffer.bytes[..buffer.length], "Valeur : 7".as_bytes());

    // An unknown locale resolves to the default one.
    set_locale("de");
    assert_eq!(locale!(), "en");

    set_locale("en");
    assert_eq!(t!("greeting"), "Hello");
    assert_eq!(t!("files", count = 0).to_string(), "0 files");

    // Resolution of regional, unknown, empty and upper case tags.
    set_locale("fr-BE");
    assert_eq!(t!("greeting"), "Bonjour");
    assert_eq!(locale!(), "fr");
    set_locale("de");
    assert_eq!(t!("greeting"), "Hello");
    assert_eq!(locale!(), "en");
    set_locale("");
    assert_eq!(t!("greeting"), "Hello");
    set_locale("FR");
    assert_eq!(t!("files", count = 1).to_string(), "1 fichier");
    assert_eq!(available_locales!(), &["en", "fr"]);
}

#[test]
fn explicit_locales_ignore_the_global_one() {
    let some_string = String::from("fr");
    assert_eq!(t!("greeting", locale = "fr"), "Bonjour");
    assert_eq!(t!("greeting", locale = Locale::new("en")), "Hello");
    assert_eq!(t!("greeting", locale = some_string.as_str()), "Bonjour");
    assert_eq!(t!("greeting", locale = "fr-CA"), "Bonjour");
    assert_eq!(t!("greeting", locale = "de"), "Hello");
    assert_eq!(
        t!("files", count = 2, locale = "fr").to_string(),
        "2 fichiers"
    );
}

#[test]
fn explicit_locales_in_const_context() {
    const FRENCH: &str = t!("greeting", locale = "fr");
    assert_eq!(FRENCH, "Bonjour");
}
