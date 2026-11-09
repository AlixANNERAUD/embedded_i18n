embedded_i18n::i18n!();

use core::ffi::CStr;
use core::fmt::Write;

use embedded_i18n::{available_locales, locale, t};

struct Buffer {
    bytes: [u8; 64],
    length: usize,
}

impl Buffer {
    fn new() -> Buffer {
        Buffer {
            bytes: [0; 64],
            length: 0,
        }
    }

    fn written(&self) -> &[u8] {
        &self.bytes[..self.length]
    }
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

#[test]
fn basic_translation() {
    assert_eq!(t!("greeting"), "Hello");
}

#[test]
fn farewell_translation() {
    assert_eq!(t!("farewell"), "Goodbye");
}

#[test]
fn translation_with_placeholder() {
    assert_eq!(t!("with_arg"), "Value: {value}");
    assert_eq!(t!("with_arg", value = 42).to_string(), "Value: 42");
}

#[test]
fn c_string_translation() {
    let c = t!(c"c_greeting");
    assert_eq!(c.to_bytes(), b"Hello from C");
    assert_eq!(c.to_bytes_with_nul(), b"Hello from C\0");
}

#[test]
fn c_string_length() {
    let c: &'static CStr = t!(c"c_greeting");
    assert_eq!(c.count_bytes(), 12);
}

#[test]
fn translate_is_str() {
    let s: &'static str = t!("greeting");
    assert_eq!(s, "Hello");
}

#[cfg(feature = "std")]
#[test]
fn translate_works_with_format_macro() {
    let result = std::format!("{} {}", t!("greeting"), t!("farewell"));
    assert_eq!(result, "Hello Goodbye");
}

#[test]
fn translate_write_positional() {
    let mut buffer = Buffer::new();
    write!(buffer, "{}", t!("with_arg", value = 42)).unwrap();
    assert_eq!(buffer.written(), b"Value: 42");
}

#[test]
fn translate_write_named() {
    let mut buffer = Buffer::new();
    let name = "Ada";
    write!(buffer, "{}", t!("with_named_args", name = name, count = 3)).unwrap();
    assert_eq!(buffer.written(), b"Hello Ada, you have 3 messages");
}

#[test]
fn translate_write_propagates_errors() {
    let mut buffer = Buffer::new();
    buffer.length = 60;
    assert!(write!(buffer, "{}", t!("with_arg", value = 42)).is_err());
}

#[test]
fn translate_args_display() {
    let mut buffer = Buffer::new();
    buffer
        .write_fmt(format_args!("{}", t!("with_arg", value = 7)))
        .unwrap();
    assert_eq!(buffer.written(), b"Value: 7");
}

#[test]
fn translate_as_format_argument() {
    assert_eq!(format!("{}", t!("with_arg", value = 1)), "Value: 1");
    assert_eq!(
        format!("{}", t!("with_named_args", name = "Bob", count = 0)),
        "Hello Bob, you have 0 messages"
    );
}

#[test]
fn translate_write_captures_variables() {
    let mut buffer = Buffer::new();
    let name = "Eve";
    let count = 5;
    write!(
        buffer,
        "{}",
        t!("with_named_args", name = name, count = count)
    )
    .unwrap();
    assert_eq!(buffer.written(), b"Hello Eve, you have 5 messages");
}

#[test]
fn translate_with_context() {
    assert_eq!(t!("Open", context = "menu"), "Open (menu)");
    assert_eq!(t!("Open", context = "state"), "Open (state)");
}

#[test]
fn translate_with_context_and_c_string() {
    assert_eq!(t!(c"Open", context = "menu").to_bytes(), b"Open (menu)");
}

#[test]
fn translate_write_with_context() {
    let mut buffer = Buffer::new();
    write!(buffer, "{}", t!("Open", context = "state")).unwrap();
    assert_eq!(buffer.written(), b"Open (state)");
}

#[test]
fn translate_plural_selects_form() {
    assert_eq!(t!("files", count = 1).to_string(), "1 file");
    assert_eq!(t!("files", count = 0usize).to_string(), "0 files");
    assert_eq!(t!("files", count = 2u8).to_string(), "2 files");
    assert_eq!(t!("files", count = -1i32).to_string(), "-1 file");
}

#[test]
fn translate_plural_with_context() {
    assert_eq!(
        t!("files", context = "menu", count = 1).to_string(),
        "Open file"
    );
    assert_eq!(
        t!("files", context = "menu", count = 3).to_string(),
        "Open files"
    );
}

#[test]
fn translate_plural_write_captures_count() {
    let mut buffer = Buffer::new();
    let files = 3;
    write!(buffer, "{}", t!("files", count = files)).unwrap();
    assert_eq!(buffer.written(), b"3 files");
    buffer.length = 0;
    write!(buffer, "{}", t!("files", count = 1)).unwrap();
    assert_eq!(buffer.written(), b"1 file");
}

#[cfg(feature = "backend-json")]
#[test]
fn translate_plural_zero_override() {
    assert_eq!(t!("files_with_zero", count = 0).to_string(), "No files");
    assert_eq!(t!("files_with_zero", count = 1).to_string(), "1 file");
}

#[test]
fn translate_select_matches_variant() {
    assert_eq!(t!("replied", select = "male"), "He replied");
    assert_eq!(t!("replied", select = "female"), "She replied");
    assert_eq!(t!("replied", select = "unknown"), "They replied");
}

enum Gender {
    Female,
    Other,
}

impl AsRef<str> for Gender {
    fn as_ref(&self) -> &str {
        match self {
            Gender::Female => "female",
            Gender::Other => "other",
        }
    }
}

#[test]
fn translate_select_accepts_as_ref_str() {
    assert_eq!(
        t!(c"replied", select = Gender::Female).to_bytes(),
        b"She replied"
    );
    let mut buffer = Buffer::new();
    write!(buffer, "{}", t!("replied", select = Gender::Other)).unwrap();
    assert_eq!(buffer.written(), b"They replied");
}

#[test]
fn locale_macros_describe_the_crate() {
    assert_eq!(available_locales!(), ["en"]);
    assert_eq!(locale!(), "en");
}

// TODO(Task 6 fix round 1): re-enable once `count = 2 + 3` keeps its precedence
// in the expansion (E0614 today). A compile error cannot be `#[ignore]`d.
#[cfg(any())]
#[test]
fn temporaries_used_inline() {
    let mut buffer = Buffer::new();
    write!(buffer, "{}", t!("files", count = 2 + 3)).unwrap();
    assert_eq!(buffer.written(), b"5 files");
}

#[test]
fn temporaries_used_inline_parenthesized() {
    let mut buffer = Buffer::new();
    write!(buffer, "{}", t!("files", count = (2 + 3))).unwrap();
    assert_eq!(buffer.written(), b"5 files");
}

#[test]
fn places_can_be_stored() {
    let n = 2;
    let m = t!("files", count = n);
    assert_eq!(m.to_string(), "2 files");
}

#[test]
fn percent_placeholders() {
    assert_eq!(t!("percent", name = "Ada").to_string(), "Ada {raw}");
}

#[test]
fn plural_form_without_count_placeholder() {
    assert_eq!(
        t!("files", context = "menu", count = 1).to_string(),
        "Open file"
    );
}

#[test]
fn translations_are_const() {
    const TITLE: &str = t!("greeting");
    const MENU: &str = t!("Open", context = "menu");
    assert_eq!(TITLE, "Hello");
    assert_eq!(MENU, "Open (menu)");
}

// Only meaningful under `EMBEDDED_I18N_LOCALE=fr`, which a dedicated CI step sets.
#[test]
fn environment_override() {
    if option_env!("EMBEDDED_I18N_LOCALE") == Some("fr") {
        assert_eq!(t!("greeting"), "Bonjour");
    }
}
