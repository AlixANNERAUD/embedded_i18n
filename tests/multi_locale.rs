use core::fmt::Write;

use embedded_i18n::{
    DEFAULT_LOCALE_INDEX, LOCALES, locale, locale_index, match_locale, set_locale,
    set_locale_index, translate, translate_args, translate_plural, translate_select,
    translate_write,
};

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

#[test]
fn switches_locale_at_runtime() {
    if LOCALES[..] != ["en", "fr"] {
        return;
    }

    assert_eq!(locale(), "en");
    assert_eq!(translate!("greeting"), "Hello");

    set_locale("fr").unwrap();
    assert_eq!(locale_index(), 1);
    assert_eq!(locale(), "fr");
    assert_eq!(translate!("greeting"), "Bonjour");
    assert_eq!(translate!(c"greeting").to_bytes(), b"Bonjour");
    assert_eq!(translate!("missing_in_french"), "Only in English");
    assert_eq!(translate_plural!("files", 0), "{count} fichier");
    assert_eq!(translate_select!("replied", "female"), "Elle a répondu");
    let buffer = written(|buffer| translate_write!(buffer, "with_arg", 42));
    assert_eq!(&buffer.bytes[..buffer.length], "Valeur : 42".as_bytes());
    let buffer = written(|buffer| write!(buffer, "{}", translate_args!("with_arg", 7)));
    assert_eq!(&buffer.bytes[..buffer.length], "Valeur : 7".as_bytes());

    assert!(set_locale("de").is_err());
    assert!(set_locale_index(2).is_err());
    assert_eq!(locale(), "fr");

    set_locale_index(0).unwrap();
    assert_eq!(translate!("greeting"), "Hello");
    assert_eq!(translate_plural!("files", 0), "{count} files");
}

#[test]
fn translates_into_an_explicit_locale() {
    if LOCALES[..] != ["en", "fr"] {
        return;
    }
    // The current locale may be changed by another test meanwhile: an explicit locale ignores it.
    let french = match_locale("fr-CA").unwrap();
    let english = match_locale("en-US").unwrap();

    assert_eq!(translate!(locale = french, "greeting"), "Bonjour");
    assert_eq!(translate!(locale = english, "greeting"), "Hello");
    assert_eq!(
        translate!(locale = french, c"greeting").to_bytes(),
        b"Bonjour"
    );
    assert_eq!(
        translate_plural!(locale = french, "files", 0),
        "{count} fichier"
    );
    assert_eq!(
        translate_select!(locale = french, "replied", "female"),
        "Elle a répondu"
    );
    let buffer = written(|buffer| translate_write!(buffer, locale = french, "with_arg", 42));
    assert_eq!(&buffer.bytes[..buffer.length], "Valeur : 42".as_bytes());
    let buffer = written(|buffer| {
        write!(
            buffer,
            "{}",
            translate_args!(locale = english, "with_arg", 7)
        )
    });
    assert_eq!(&buffer.bytes[..buffer.length], "Value: 7".as_bytes());
    assert_eq!(
        translate!(locale = LOCALES.len(), "greeting"),
        translate!(locale = DEFAULT_LOCALE_INDEX, "greeting"),
        "an unknown index selects the default locale"
    );
}
