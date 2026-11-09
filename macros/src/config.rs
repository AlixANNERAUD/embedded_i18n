//! `i18n!` configuration: arguments, `[package.metadata.i18n]` and environment overrides.

use std::path::{Path, PathBuf};

use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{LitBool, LitStr, Token, braced, bracketed};

/// Highest number of available locales a crate may declare.
const MAXIMUM_LOCALES: usize = 255;

/// Directory, relative to the manifest, holding the translation files by default.
const DEFAULT_PATH: &str = "locales";

/// Locale used as fallback and as default when nothing else is configured.
const DEFAULT_LOCALE: &str = "en";

/// Options accepted by `i18n!`, in the order reported in error messages.
const OPTIONS: &str = "fallback, default_locale, available_locales, strict, bidi_isolation";

/// Resolved configuration of one `i18n!` invocation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Config {
    /// Absolute path of the directory holding the translation files.
    pub path: PathBuf,
    /// Locales tried, in order, when a message is missing from the active locale.
    pub fallback: Vec<String>,
    /// Locale active before any call to `set_locale`.
    pub default_locale: String,
    /// Every locale the crate can switch to; never empty.
    pub available_locales: Vec<String>,
    /// Whether a missing translation is a compile-time error.
    pub strict: bool,
    /// Whether interpolated values are wrapped in Unicode bidirectional isolates.
    pub bidi_isolation: bool,
}

/// The two environment overrides, read by the caller so that this module stays pure.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Environment {
    /// Overrides the default locale.
    pub locale: Option<String>,
    /// Overrides the available locales (comma separated).
    pub locales: Option<String>,
}

/// Options as written in the arguments (or in the resolved token form), before merging.
#[derive(Default)]
struct Options {
    path: Option<String>,
    fallback: Option<Vec<String>>,
    default_locale: Option<String>,
    available_locales: Option<Vec<String>>,
    strict: Option<bool>,
    bidi_isolation: Option<bool>,
}

impl Options {
    /// Parse `name = value` pairs until the stream is exhausted.
    ///
    /// `path` is only accepted when `allow_path` is set (the resolved form).
    fn parse_pairs(input: ParseStream, allow_path: bool) -> syn::Result<Options> {
        let mut options = Options::default();
        while !input.is_empty() {
            let name = Ident::parse_any(input)?;
            let text = name.to_string();
            match text.as_str() {
                "path" if allow_path => {
                    let value = Self::take(&mut options.path, &name, || {
                        input.parse::<Token![=]>()?;
                        Ok(input.parse::<LitStr>()?.value())
                    })?;
                    options.path = Some(value);
                }
                "fallback" => {
                    let value = Self::take(&mut options.fallback, &name, || {
                        input.parse::<Token![=]>()?;
                        if input.peek(LitStr) {
                            Ok(vec![input.parse::<LitStr>()?.value()])
                        } else {
                            parse_string_list(input)
                        }
                    })?;
                    options.fallback = Some(value);
                }
                "default_locale" => {
                    let value = Self::take(&mut options.default_locale, &name, || {
                        input.parse::<Token![=]>()?;
                        Ok(input.parse::<LitStr>()?.value())
                    })?;
                    options.default_locale = Some(value);
                }
                "available_locales" => {
                    let value = Self::take(&mut options.available_locales, &name, || {
                        input.parse::<Token![=]>()?;
                        parse_string_list(input)
                    })?;
                    options.available_locales = Some(value);
                }
                "strict" => {
                    let value = Self::take(&mut options.strict, &name, || {
                        input.parse::<Token![=]>()?;
                        Ok(input.parse::<LitBool>()?.value)
                    })?;
                    options.strict = Some(value);
                }
                "bidi_isolation" => {
                    let value = Self::take(&mut options.bidi_isolation, &name, || {
                        input.parse::<Token![=]>()?;
                        Ok(input.parse::<LitBool>()?.value)
                    })?;
                    options.bidi_isolation = Some(value);
                }
                _ => {
                    return Err(syn::Error::new(
                        name.span(),
                        format!("unknown option `{text}`, expected one of: {OPTIONS}"),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(options)
    }

    /// Reject a second occurrence of `name`, otherwise parse its value.
    fn take<T>(
        slot: &mut Option<T>,
        name: &Ident,
        parse: impl FnOnce() -> syn::Result<T>,
    ) -> syn::Result<T> {
        if slot.is_some() {
            return Err(syn::Error::new(
                name.span(),
                format!("`{name}` is given twice"),
            ));
        }
        parse()
    }
}

/// Parse `["a", "b", ...]` (trailing comma allowed).
fn parse_string_list(input: ParseStream) -> syn::Result<Vec<String>> {
    let content;
    bracketed!(content in input);
    let items = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;
    Ok(items.iter().map(LitStr::value).collect())
}

/// The arguments of `i18n!`: an optional leading path literal, then options.
struct Arguments {
    path: Option<String>,
    options: Options,
}

impl Parse for Arguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut path = None;
        if input.peek(LitStr) {
            path = Some(input.parse::<LitStr>()?.value());
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        let options = Options::parse_pairs(input, false)?;
        Ok(Arguments { path, options })
    }
}

/// Values read from `[package.metadata.i18n]`.
#[derive(Default)]
struct Metadata {
    load_path: Option<String>,
    fallback: Option<Vec<String>>,
    default_locale: Option<String>,
    available_locales: Option<Vec<String>>,
    strict: Option<bool>,
    bidi_isolation: Option<bool>,
}

impl Metadata {
    /// Read the metadata of the manifest in `manifest_dir`; a missing file or section gives defaults.
    fn read(manifest_dir: &Path) -> syn::Result<Metadata> {
        let manifest = Config::manifest_file(manifest_dir);
        let invalid = |detail: String| {
            syn::Error::new(
                Span::call_site(),
                format!(
                    "invalid [package.metadata.i18n] in {}: {detail}",
                    manifest.display()
                ),
            )
        };
        let text = match std::fs::read_to_string(&manifest) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Metadata::default());
            }
            Err(error) => return Err(invalid(error.to_string())),
        };
        let table: ::toml::Table =
            ::toml::from_str(&text).map_err(|error| invalid(error.to_string()))?;
        let Some(section) = table
            .get("package")
            .and_then(|package| package.get("metadata"))
            .and_then(|metadata| metadata.get("i18n"))
        else {
            return Ok(Metadata::default());
        };
        let section = section
            .as_table()
            .ok_or_else(|| invalid("expected a table".to_string()))?;

        let mut metadata = Metadata::default();
        for (key, value) in section {
            match key.as_str() {
                "load-path" => {
                    metadata.load_path = Some(metadata_string(key, value).map_err(invalid)?)
                }
                "fallback" => {
                    metadata.fallback = Some(metadata_string_or_list(key, value).map_err(invalid)?)
                }
                "default-locale" => {
                    metadata.default_locale = Some(metadata_string(key, value).map_err(invalid)?);
                }
                "available-locales" => {
                    metadata.available_locales = Some(metadata_list(key, value).map_err(invalid)?);
                }
                "strict" => metadata.strict = Some(metadata_bool(key, value).map_err(invalid)?),
                "bidi-isolation" => {
                    metadata.bidi_isolation = Some(metadata_bool(key, value).map_err(invalid)?);
                }
                _ => {
                    return Err(invalid(format!(
                        "unknown key `{key}`, expected one of: load-path, fallback, default-locale, available-locales, strict, bidi-isolation"
                    )));
                }
            }
        }
        Ok(metadata)
    }
}

fn metadata_string(key: &str, value: &::toml::Value) -> Result<String, String> {
    value
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| format!("`{key}` must be a string"))
}

fn metadata_bool(key: &str, value: &::toml::Value) -> Result<bool, String> {
    value
        .as_bool()
        .ok_or_else(|| format!("`{key}` must be a boolean"))
}

fn metadata_list(key: &str, value: &::toml::Value) -> Result<Vec<String>, String> {
    let error = || format!("`{key}` must be an array of strings");
    value
        .as_array()
        .ok_or_else(error)?
        .iter()
        .map(|item| item.as_str().map(str::to_string).ok_or_else(error))
        .collect()
}

fn metadata_string_or_list(key: &str, value: &::toml::Value) -> Result<Vec<String>, String> {
    match value.as_str() {
        Some(single) => Ok(vec![single.to_string()]),
        None => metadata_list(key, value)
            .map_err(|_| format!("`{key}` must be a string or an array of strings")),
    }
}

impl Environment {
    /// The locale override, ignoring an empty value.
    fn locale(&self) -> Option<String> {
        self.locale
            .as_deref()
            .map(str::trim)
            .filter(|locale| !locale.is_empty())
            .map(str::to_string)
    }

    /// The locales override: comma separated, entries trimmed, empty entries dropped.
    /// A value without any entry is treated as unset.
    fn locales(&self) -> Option<Vec<String>> {
        let list: Vec<String> = self
            .locales
            .as_deref()?
            .split(',')
            .map(str::trim)
            .filter(|locale| !locale.is_empty())
            .map(str::to_string)
            .collect();
        (!list.is_empty()).then_some(list)
    }
}

impl Config {
    /// Resolve the configuration of an `i18n!` call.
    ///
    /// Precedence: environment, then arguments, then `[package.metadata.i18n]`, then defaults.
    pub fn from_i18n_args(
        args: TokenStream2,
        manifest_dir: &Path,
        environment: &Environment,
    ) -> syn::Result<Config> {
        let arguments: Arguments = syn::parse2(args)?;
        let metadata = Metadata::read(manifest_dir)?;
        let options = arguments.options;

        let load_path = arguments
            .path
            .or(metadata.load_path)
            .unwrap_or_else(|| DEFAULT_PATH.to_string());
        let default_locale = environment
            .locale()
            .or(options.default_locale)
            .or(metadata.default_locale)
            .unwrap_or_else(|| DEFAULT_LOCALE.to_string());
        let available_locales = environment
            .locales()
            .or(options.available_locales)
            .or(metadata.available_locales)
            .unwrap_or_else(|| vec![default_locale.clone()]);
        let fallback = options
            .fallback
            .or(metadata.fallback)
            .unwrap_or_else(|| vec![DEFAULT_LOCALE.to_string()]);

        let config = Config {
            path: manifest_dir.join(load_path),
            fallback,
            default_locale,
            available_locales,
            strict: options.strict.or(metadata.strict).unwrap_or(false),
            bidi_isolation: options
                .bidi_isolation
                .or(metadata.bidi_isolation)
                .unwrap_or(false),
        };
        config.validate()?;
        Ok(config)
    }

    /// Check the invariants of a merged configuration.
    fn validate(&self) -> syn::Result<()> {
        let count = self.available_locales.len();
        if count > MAXIMUM_LOCALES {
            return Err(syn::Error::new(
                Span::call_site(),
                format!(
                    "available_locales lists {count} locales, at most {MAXIMUM_LOCALES} are supported"
                ),
            ));
        }
        if !self.available_locales.contains(&self.default_locale) {
            let list = self
                .available_locales
                .iter()
                .map(|locale| format!("{locale:?}"))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(syn::Error::new(
                Span::call_site(),
                format!(
                    "default locale \"{}\" is not in available_locales [{list}]",
                    self.default_locale
                ),
            ));
        }
        Ok(())
    }

    /// The fully resolved form, as handed to `__expand!`:
    /// `{ path = "..", fallback = [..], default_locale = "..", available_locales = [..], strict = .., bidi_isolation = .. }`.
    pub fn to_tokens(&self) -> TokenStream2 {
        let path = self.path.to_string_lossy().into_owned();
        let fallback = &self.fallback;
        let default_locale = &self.default_locale;
        let available_locales = &self.available_locales;
        let strict = self.strict;
        let bidi_isolation = self.bidi_isolation;
        quote! {
            {
                path = #path,
                fallback = [#(#fallback),*],
                default_locale = #default_locale,
                available_locales = [#(#available_locales),*],
                strict = #strict,
                bidi_isolation = #bidi_isolation
            }
        }
    }

    /// Parse exactly what [`Config::to_tokens`] emits, without merging anything.
    pub fn from_resolved(tokens: TokenStream2) -> syn::Result<Config> {
        let parser = |input: ParseStream| -> syn::Result<Options> {
            let content;
            braced!(content in input);
            Options::parse_pairs(&content, true)
        };
        let options = parser.parse2(tokens)?;
        let missing = |name: &str| syn::Error::new(Span::call_site(), format!("missing `{name}`"));
        let config = Config {
            path: PathBuf::from(options.path.ok_or_else(|| missing("path"))?),
            fallback: options.fallback.ok_or_else(|| missing("fallback"))?,
            default_locale: options
                .default_locale
                .ok_or_else(|| missing("default_locale"))?,
            available_locales: options
                .available_locales
                .ok_or_else(|| missing("available_locales"))?,
            strict: options.strict.ok_or_else(|| missing("strict"))?,
            bidi_isolation: options
                .bidi_isolation
                .ok_or_else(|| missing("bidi_isolation"))?,
        };
        config.validate()?;
        Ok(config)
    }

    /// Whether the crate has more than one locale to choose from.
    pub fn is_multiple(&self) -> bool {
        self.available_locales.len() > 1
    }

    /// The manifest of the crate being compiled.
    pub fn manifest_file(manifest_dir: &Path) -> PathBuf {
        manifest_dir.join("Cargo.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    fn no_environment() -> Environment {
        Environment::default()
    }

    /// A manifest directory holding `manifest` as its `Cargo.toml`.
    fn manifest_with(manifest: &str) -> tempfile::TempDir {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("Cargo.toml"), manifest).unwrap();
        directory
    }

    fn resolve(args: TokenStream2, directory: &Path) -> syn::Result<Config> {
        Config::from_i18n_args(args, directory, &no_environment())
    }

    fn error_of(args: TokenStream2) -> String {
        let directory = tempfile::tempdir().unwrap();
        resolve(args, directory.path()).unwrap_err().to_string()
    }

    #[test]
    fn defaults_without_arguments_or_metadata() {
        let directory = tempfile::tempdir().unwrap();
        let config = resolve(quote! {}, directory.path()).unwrap();
        assert_eq!(config.path, directory.path().join("locales"));
        assert_eq!(config.fallback, strings(&["en"]));
        assert_eq!(config.default_locale, "en");
        assert_eq!(config.available_locales, strings(&["en"]));
        assert!(!config.strict);
        assert!(!config.bidi_isolation);
    }

    #[test]
    fn reads_rust_i18n_style_arguments() {
        let directory = tempfile::tempdir().unwrap();
        let config = resolve(
            quote! { "i18n", fallback = ["en", "es"], default_locale = "fr", available_locales = ["en", "fr"], strict = true },
            directory.path(),
        )
        .unwrap();
        assert_eq!(config.path, directory.path().join("i18n"));
        assert_eq!(config.fallback, strings(&["en", "es"]));
        assert_eq!(config.default_locale, "fr");
        assert_eq!(config.available_locales, strings(&["en", "fr"]));
        assert!(config.strict);
        assert!(!config.bidi_isolation);
    }

    #[test]
    fn fallback_accepts_a_single_string() {
        let directory = tempfile::tempdir().unwrap();
        let config = resolve(quote! { fallback = "de", }, directory.path()).unwrap();
        assert_eq!(config.fallback, strings(&["de"]));
    }

    #[test]
    fn reads_package_metadata() {
        let directory = manifest_with(
            r#"
[package]
name = "demo"

[package.metadata.i18n]
load-path = "l"
default-locale = "fr"
available-locales = ["en", "fr"]
fallback = "en"
bidi-isolation = true
"#,
        );
        let config = resolve(quote! {}, directory.path()).unwrap();
        assert_eq!(config.path, directory.path().join("l"));
        assert_eq!(config.fallback, strings(&["en"]));
        assert_eq!(config.default_locale, "fr");
        assert_eq!(config.available_locales, strings(&["en", "fr"]));
        assert!(!config.strict);
        assert!(config.bidi_isolation);
    }

    #[test]
    fn rejects_invalid_metadata() {
        let directory = manifest_with("[package.metadata.i18n]\nstrict = \"yes\"\n");
        let error = resolve(quote! {}, directory.path())
            .unwrap_err()
            .to_string();
        assert_eq!(
            error,
            format!(
                "invalid [package.metadata.i18n] in {}: `strict` must be a boolean",
                directory.path().join("Cargo.toml").display()
            )
        );
    }

    #[test]
    fn arguments_override_metadata() {
        let directory = manifest_with(
            "[package.metadata.i18n]\ndefault-locale = \"de\"\navailable-locales = [\"de\"]\n",
        );
        let config = resolve(
            quote! { default_locale = "fr", available_locales = ["fr"] },
            directory.path(),
        )
        .unwrap();
        assert_eq!(config.default_locale, "fr");
        assert_eq!(config.available_locales, strings(&["fr"]));
    }

    #[test]
    fn environment_overrides_everything() {
        let directory = manifest_with(
            "[package.metadata.i18n]\ndefault-locale = \"de\"\navailable-locales = [\"de\"]\n",
        );
        let environment = Environment {
            locale: Some("fr".to_string()),
            locales: Some(" en, fr ,".to_string()),
        };
        let config = Config::from_i18n_args(
            quote! { default_locale = "es", available_locales = ["es"] },
            directory.path(),
            &environment,
        )
        .unwrap();
        assert_eq!(config.default_locale, "fr");
        assert_eq!(config.available_locales, strings(&["en", "fr"]));
    }

    #[test]
    fn round_trips_through_tokens() {
        let directory = tempfile::tempdir().unwrap();
        let config = resolve(
            quote! { "i18n", fallback = ["en", "es"], default_locale = "fr", available_locales = ["en", "fr"], strict = true, bidi_isolation = true },
            directory.path(),
        )
        .unwrap();
        assert_eq!(Config::from_resolved(config.to_tokens()).unwrap(), config);

        let defaults = resolve(quote! {}, directory.path()).unwrap();
        assert_eq!(
            Config::from_resolved(defaults.to_tokens()).unwrap(),
            defaults
        );
    }

    #[test]
    fn rejects_unknown_and_duplicate_options() {
        assert_eq!(
            error_of(quote! { colour = "red" }),
            "unknown option `colour`, expected one of: fallback, default_locale, available_locales, strict, bidi_isolation"
        );
        assert_eq!(
            error_of(quote! { strict = true, strict = false }),
            "`strict` is given twice"
        );
        assert_eq!(
            error_of(quote! { fallback = "en", fallback = "fr" }),
            "`fallback` is given twice"
        );
    }

    #[test]
    fn rejects_default_outside_available() {
        assert_eq!(
            error_of(quote! { default_locale = "de", available_locales = ["en", "fr"] }),
            "default locale \"de\" is not in available_locales [\"en\", \"fr\"]"
        );
    }

    #[test]
    fn rejects_more_than_255_locales() {
        let locales: Vec<String> = (0..256).map(|index| format!("l{index}")).collect();
        assert_eq!(
            error_of(quote! { default_locale = "l0", available_locales = [#(#locales),*] }),
            "available_locales lists 256 locales, at most 255 are supported"
        );

        let accepted: Vec<String> = (0..255).map(|index| format!("l{index}")).collect();
        let directory = tempfile::tempdir().unwrap();
        let config = resolve(
            quote! { default_locale = "l0", available_locales = [#(#accepted),*] },
            directory.path(),
        )
        .unwrap();
        assert_eq!(config.available_locales.len(), 255);
    }

    #[test]
    fn is_multiple_only_with_two_or_more_locales() {
        let directory = tempfile::tempdir().unwrap();
        let single = resolve(quote! {}, directory.path()).unwrap();
        assert!(!single.is_multiple());
        let several = resolve(
            quote! { available_locales = ["en", "fr"] },
            directory.path(),
        )
        .unwrap();
        assert!(several.is_multiple());
    }

    #[test]
    fn manifest_file_is_cargo_toml_in_the_directory() {
        assert_eq!(
            Config::manifest_file(Path::new("/crate")),
            PathBuf::from("/crate/Cargo.toml")
        );
    }
}
