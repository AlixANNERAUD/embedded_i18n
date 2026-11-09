# Getting Started

In this tutorial, you build a small status screen that greets a user and counts their messages and files.
You first show it in English, then build it in French, and finally switch between both languages while the program runs.

Along the way you write locale files, configure the crate with [`i18n!`], and translate with [`t!`]: a plain string, a message with arguments, and a plural.
The code only uses `core` and needs no allocator, so it runs unchanged in a `#![no_std]` firmware, where it would write into a display driver.
In this tutorial it runs on your computer, so that you can see the output in a terminal.

You need Rust 1.85 or later.

## 1. Create the Project

Create a new binary crate:

```sh
cargo new status-screen
cd status-screen
```

Add `embedded-i18n` to the `[dependencies]` of `Cargo.toml`:

```toml
[dependencies]
embedded-i18n = "0.2"
```

The default features read JSON locale files, which is what this tutorial uses.

## 2. Write the English Locale File

Translations live in a `locales/` directory next to `Cargo.toml`, with one file per language.
Create `locales/en.json`:

```json
{
  "greeting": "Hello",
  "with_named_args": "Hello {name}, you have {count} messages",
  "files": {
    "one": "{count} file",
    "other": "{count} files"
  }
}
```

Each entry maps a key to its translation:

- `greeting` is a plain string;
- `with_named_args` has two placeholders, `{name}` and `{count}`, that the program fills in;
- `files` has two plural forms: `one` for a single file, and `other` for any other number.

## 3. Translate a String

Replace `src/main.rs` with:

```rust
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

fn main() {
    println!("{}", t!("greeting"));
#   assert_eq!(t!("greeting"), "Hello");
}
```

Run it:

```sh
cargo run
```

```text
Hello
```

[`i18n!`] tells the crate where its locale files are.
Call it once, at the top of `main.rs` (or `lib.rs`), before any `mod` declaration.
[`t!`] then replaces the key with its translation while the program compiles: the binary contains the string `"Hello"`, and no locale file is read at runtime.

Now make a typo in the key, `t!("greting")`, and run `cargo run` again.
The build fails, naming the key and the locales it was looked up in:

```text
error: translation for "greting" not found in locales "en" (/path/to/status-screen/locales)
```

Missing translations are compile errors, so they never reach a device.
Fix the typo before going on.

## 4. Fill In Arguments and Pick a Plural Form

A real screen shows values computed at runtime.
Write the screen in a `render` function that takes any [`core::fmt::Write`] destination:

```rust
use core::fmt::Write;
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

fn render(screen: &mut impl Write, name: &str, messages: u32, files: u32) -> core::fmt::Result {
    writeln!(screen, "{}", t!("with_named_args", name = name, count = messages))?;
    writeln!(screen, "{}", t!("files", count = files))
}

fn main() {
    let mut screen = String::new();
    render(&mut screen, "Ada", 3, 1).unwrap();
    print!("{screen}");
#   assert_eq!(screen, "Hello Ada, you have 3 messages\n1 file\n");
}
```

```text
Hello Ada, you have 3 messages
1 file
```

With arguments, [`t!`] returns a value that formats the message when it is written, without allocating.
`name = name` fills the `{name}` placeholder, and `count = ...` fills `{count}`.
For `files`, `count` also picks the plural form: change the last argument of `render` to `2` and the second line becomes `2 files`.

The placeholders are checked while compiling.
Remove `count = messages` and run again: the build fails, because the English translation uses `{count}`:

```text
error: translation of "with_named_args" in locale "en" uses the placeholder {count}, which is not an argument of t!
```

Put the argument back before going on.

## 5. Drop the Allocator

`String` needs an allocator, which a firmware usually does not have.
`render` accepts anything implementing [`core::fmt::Write`], so give it a fixed-size buffer instead:

```rust
use core::fmt::{self, Write};
use embedded_i18n::t;

embedded_i18n::i18n!("locales");

fn render(screen: &mut impl Write, name: &str, messages: u32, files: u32) -> fmt::Result {
    writeln!(screen, "{}", t!("with_named_args", name = name, count = messages))?;
    writeln!(screen, "{}", t!("files", count = files))
}

/// A screen buffer of 64 bytes, without allocation.
struct Buffer {
    bytes: [u8; 64],
    length: usize,
}

impl Write for Buffer {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.length + text.len();
        self.bytes.get_mut(self.length..end).ok_or(fmt::Error)?.copy_from_slice(text.as_bytes());
        self.length = end;
        Ok(())
    }
}

fn main() {
    let mut buffer = Buffer { bytes: [0; 64], length: 0 };
    render(&mut buffer, "Ada", 3, 2).unwrap();
    let text = core::str::from_utf8(&buffer.bytes[..buffer.length]).unwrap();
    print!("{text}");
#   assert_eq!(text, "Hello Ada, you have 3 messages\n2 files\n");
}
```

```text
Hello Ada, you have 3 messages
2 files
```

`render` did not change: everything it does comes from `core`.
In a firmware, `Buffer` would be your display driver, or a fixed-capacity string from a crate such as `heapless`.

## 6. Build in French

Create `locales/fr.json`, with the same keys:

```json
{
  "greeting": "Bonjour",
  "with_named_args": "Bonjour {name}, vous avez {count} messages",
  "files": {
    "one": "{count} fichier",
    "other": "{count} fichiers"
  }
}
```

Then tell [`i18n!`] to translate into French with `default_locale`:

```rust
use core::fmt::Write;
use embedded_i18n::t;

embedded_i18n::i18n!("locales", default_locale = "fr");

fn render(screen: &mut impl Write, name: &str, messages: u32, files: u32) -> core::fmt::Result {
    writeln!(screen, "{}", t!("with_named_args", name = name, count = messages))?;
    writeln!(screen, "{}", t!("files", count = files))
}

fn main() {
    let mut screen = String::new();
    render(&mut screen, "Ada", 3, 1).unwrap();
    print!("{screen}");
#   assert_eq!(screen, "Bonjour Ada, vous avez 3 messages\n1 fichier\n");
}
```

```text
Bonjour Ada, vous avez 3 messages
1 fichier
```

The binary now contains the French strings only.
French plural rules differ from English ones: try a count of `0`, which is `0 fichier` in French but `0 files` in English.

A key missing from `fr.json` falls back to `en.json`, the default fallback language.

You can also leave the code untouched and pick the language when building, which is handy to build one firmware per language.
Put `embedded_i18n::i18n!("locales");` back, and run:

```sh
EMBEDDED_I18N_LOCALE=fr cargo run
```

## 7. Switch Language at Runtime

Some devices let the user pick the language in a settings menu.
List the locales to embed with `available_locales`, and select one with [`set_locale`]:

```rust,standalone_crate
use core::fmt::Write;
use embedded_i18n::{set_locale, t};

embedded_i18n::i18n!("locales", default_locale = "en", available_locales = ["en", "fr"]);

fn render(screen: &mut impl Write, name: &str, messages: u32, files: u32) -> core::fmt::Result {
    writeln!(screen, "{}", t!("with_named_args", name = name, count = messages))?;
    writeln!(screen, "{}", t!("files", count = files))
}

fn main() {
    let mut screen = String::new();
    for &language in embedded_i18n::available_locales!() {
        set_locale(language);
        render(&mut screen, "Ada", 3, 1).unwrap();
    }
    print!("{screen}");
    assert_eq!(
        screen,
        "Hello Ada, you have 3 messages\n1 file\nBonjour Ada, vous avez 3 messages\n1 fichier\n"
    );
}
```

```text
Hello Ada, you have 3 messages
1 file
Bonjour Ada, vous avez 3 messages
1 fichier
```

`render` is still the same function.
The binary now contains both languages, and each [`t!`] picks the translation of the current locale.
The current locale takes 4 bytes of RAM, and setting it needs no allocation.

## What You Built

You wrote locale files in two languages, translated a string, a message with arguments and a plural, saw the compiler catch a wrong key and a missing argument, rendered the screen without an allocator, and switched languages at runtime.

From here:

- the [how-to guides][guide-how-to] solve specific tasks, such as [building one firmware per language][how-to-build-per-language], [using gettext PO files][how-to-use-po-files] or [shipping complete translations][how-to-ship-complete];
- the [`t!` reference][reference-t] and the [configuration reference][reference-configuration] list every option;
- [How It Works][explanation-how-it-works] explains what the macros generate and what it costs.

[guide-how-to]: ../../README.md#how-to-guides
[how-to-build-per-language]: ../how_to/build_one_firmware_per_language.md
[how-to-use-po-files]: ../how_to/use_gettext_po_files.md
[how-to-ship-complete]: ../how_to/ship_complete_translations.md
[reference-t]: ../reference/t_macro.md
[reference-configuration]: ../reference/configuration.md
[explanation-how-it-works]: ../explanation/how_it_works.md
[`i18n!`]: ../reference/configuration.md
[`t!`]: ../reference/t_macro.md
[`set_locale`]: ../reference/runtime_api.md
[`core::fmt::Write`]: https://doc.rust-lang.org/core/fmt/trait.Write.html
