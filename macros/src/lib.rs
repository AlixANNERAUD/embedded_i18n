//! Procedural macros of [embedded-i18n](https://docs.rs/embedded-i18n).
//!
//! This crate is an implementation detail and is not meant to be used
//! directly: depend on `embedded-i18n`, which re-exports and documents the
//! macros, and provides the runtime items their expansions refer to. Its API
//! and versioning follow `embedded-i18n`.
//!
//! The macros read the locale files of the calling crate at compile time,
//! configured by the `EMBEDDED_I18N_*` environment variables, and expand to
//! string literals or to selections between them. The locale file formats are
//! enabled by the `backend-*` features, which `embedded-i18n` forwards.
#![cfg_attr(doc, deny(missing_docs))]

use std::collections::HashMap;
use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Lit, LitCStr, LitStr};

use plural::{Category, Expression};

#[allow(dead_code)] // wired in by the t! expansion
mod call;
#[allow(dead_code)] // wired in by the i18n! expansion
mod config;
#[cfg(feature = "backend-fluent")]
mod fluent;
#[cfg(feature = "backend-json")]
mod json;
mod locale;
#[allow(dead_code)] // wired in by the i18n! expansion
mod locale_id;
#[cfg(feature = "backend-mo")]
mod mo;
#[cfg_attr(
    not(any(feature = "backend-po", feature = "backend-mo")),
    allow(dead_code)
)]
mod plural;
#[allow(dead_code)] // wired in by the t! expansion
mod placeholder;
#[cfg(feature = "backend-po")]
mod po;
#[cfg(feature = "backend-toml")]
mod toml;
mod transform;
#[cfg(any(
    feature = "backend-json",
    feature = "backend-yaml",
    feature = "backend-toml"
))]
mod tree;
#[cfg(feature = "backend-yaml")]
mod yaml;

pub(crate) const CONTEXT_SEPARATOR: char = '\u{4}';

#[derive(Default)]
pub(crate) struct Translations {
    pub messages: HashMap<String, String>,
    pub plurals: HashMap<String, Vec<String>>,
    pub plural_rule: Option<plural::Expression>,
}

impl Translations {
    fn map_values(&mut self, transform: impl Fn(&str) -> String) {
        for value in self.messages.values_mut() {
            *value = transform(value);
        }
        for forms in self.plurals.values_mut() {
            for form in forms {
                *form = transform(form);
            }
        }
    }
}

impl From<HashMap<String, String>> for Translations {
    fn from(messages: HashMap<String, String>) -> Self {
        Translations {
            messages,
            ..Translations::default()
        }
    }
}

type Loader = fn(&Path) -> Result<Translations, String>;

const BACKENDS: &[(&[&str], Loader)] = &[
    #[cfg(feature = "backend-json")]
    (json::EXTENSIONS, |path| {
        json::load(path).map(Translations::from)
    }),
    #[cfg(feature = "backend-yaml")]
    (yaml::EXTENSIONS, |path| {
        yaml::load(path).map(Translations::from)
    }),
    #[cfg(feature = "backend-toml")]
    (toml::EXTENSIONS, |path| {
        toml::load(path).map(Translations::from)
    }),
    #[cfg(feature = "backend-po")]
    (po::EXTENSIONS, po::load),
    #[cfg(feature = "backend-mo")]
    (mo::EXTENSIONS, mo::load),
    #[cfg(feature = "backend-fluent")]
    (fluent::EXTENSIONS, |path| {
        fluent::load(path).map(Translations::from)
    }),
];

struct Locale {
    name: String,
    translations: Translations,
}

enum Plural<'a> {
    Categories {
        locale: &'a str,
        forms: Vec<(Category, &'a str)>,
    },
    Indexed {
        rule: Option<&'a Expression>,
        forms: &'a [String],
    },
}

struct Catalog {
    path: PathBuf,
    chain: Vec<String>,
    primary: Vec<String>,
    strict: bool,
    files: Vec<PathBuf>,
    locales: Vec<Locale>,
}

impl Catalog {
    fn scope(&self, strict: bool) -> impl Iterator<Item = &Locale> {
        self.locales
            .iter()
            .filter(move |locale| !strict || self.primary.contains(&locale.name))
    }

    fn not_found(&self, span: Span, subject: String, found: bool) -> syn::Error {
        let list = |locales: &[String]| {
            locales
                .iter()
                .map(|locale| format!("{:?}", locale))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let message = if self.strict && found {
            format!(
                "{} is missing from the primary locale {} (EMBEDDED_I18N_STRICT is set)",
                subject,
                list(&self.primary)
            )
        } else {
            format!(
                "{} not found in locales {} ({})",
                subject,
                list(&self.chain),
                self.path.display()
            )
        };
        syn::Error::new(span, message)
    }

    fn lookup(&self, keys: &[String], strict: bool) -> Option<&str> {
        self.find(keys, strict, |value| !value.is_empty())
            .or_else(|| self.find(keys, strict, |_| true))
    }

    fn plural(&self, keys: &[String], strict: bool) -> Option<Plural<'_>> {
        self.scope(strict).find_map(|locale| {
            keys.iter().find_map(|key| {
                if let Some(forms) = locale.translations.plurals.get(key) {
                    return Some(Plural::Indexed {
                        rule: locale.translations.plural_rule.as_ref(),
                        forms,
                    });
                }
                let forms: Vec<(Category, &str)> = Category::ALL
                    .iter()
                    .filter_map(|category| {
                        locale
                            .translations
                            .messages
                            .get(&format!("{}.{}", key, category.name()))
                            .filter(|value| !value.is_empty())
                            .map(|value| (*category, value.as_str()))
                    })
                    .collect();
                (!forms.is_empty()).then_some(Plural::Categories {
                    locale: &locale.name,
                    forms,
                })
            })
        })
    }

    fn select(&self, keys: &[String], strict: bool) -> Option<Vec<(&str, &str)>> {
        self.scope(strict).find_map(|locale| {
            keys.iter().find_map(|key| {
                let prefix = format!("{}.", key);
                let has_other = locale
                    .translations
                    .messages
                    .get(&format!("{}other", prefix))
                    .is_some_and(|value| !value.is_empty());
                if !has_other {
                    return None;
                }
                let mut variants: Vec<(&str, &str)> = locale
                    .translations
                    .messages
                    .iter()
                    .filter_map(|(candidate, value)| {
                        let variant = candidate.strip_prefix(&prefix)?;
                        (!variant.is_empty() && !variant.contains('.') && !value.is_empty())
                            .then_some((variant, value.as_str()))
                    })
                    .collect();
                variants.sort();
                Some(variants)
            })
        })
    }

    fn find(&self, keys: &[String], strict: bool, accept: impl Fn(&str) -> bool) -> Option<&str> {
        self.scope(strict).find_map(|locale| {
            keys.iter().find_map(|key| {
                locale
                    .translations
                    .messages
                    .get(key)
                    .map(String::as_str)
                    .filter(|value| accept(value))
            })
        })
    }
}

struct Catalogs {
    multiple: bool,
    catalogs: Vec<Result<Catalog, String>>,
}

const MAXIMUM_LOCALES: usize = 255;

static CATALOGS: LazyLock<Catalogs> = LazyLock::new(|| {
    let embedded = std::env::var("EMBEDDED_I18N_LOCALES").unwrap_or_default();
    let embedded = embedded_locales(&embedded);
    let locale = std::env::var("EMBEDDED_I18N_LOCALE").unwrap_or_else(|_| "en".to_string());
    let load = |locale: &str| -> Result<Catalog, String> {
        let manifest_directory = std::env::var("CARGO_MANIFEST_DIR")
            .map_err(|_| "CARGO_MANIFEST_DIR is not set".to_string())?;
        let fallback = std::env::var("EMBEDDED_I18N_FALLBACK").unwrap_or_else(|_| "en".to_string());
        let path = locales_directory(
            Path::new(&manifest_directory),
            std::env::var("EMBEDDED_I18N_PATH").ok().as_deref(),
        );
        let options = Options {
            strict: is_enabled(std::env::var("EMBEDDED_I18N_STRICT").ok().as_deref()),
            isolate: is_enabled(
                std::env::var("EMBEDDED_I18N_BIDI_ISOLATION")
                    .ok()
                    .as_deref(),
            ),
        };
        load_catalog(&path, locale, &fallback, options)
    };
    if embedded.is_empty() {
        return Catalogs {
            multiple: false,
            catalogs: vec![load(&locale)],
        };
    }
    if embedded.len() > MAXIMUM_LOCALES {
        return Catalogs {
            multiple: true,
            catalogs: vec![Err(format!(
                "EMBEDDED_I18N_LOCALES lists {} locales, at most {} are supported",
                embedded.len(),
                MAXIMUM_LOCALES
            ))],
        };
    }
    Catalogs {
        multiple: true,
        catalogs: embedded.into_iter().map(load).collect(),
    }
});

fn embedded_locales(list: &str) -> Vec<&str> {
    list.split(',')
        .map(str::trim_ascii)
        .filter(|locale| !locale.is_empty())
        .collect()
}

type Expansion = fn(TokenStream2, &Result<Catalog, String>) -> syn::Result<TokenStream2>;

fn locale_index() -> TokenStream2 {
    quote! { ::embedded_i18n::locale_index() }
}

fn localized(
    input: TokenStream2,
    catalogs: &Catalogs,
    expand: Expansion,
    combine: fn(Vec<TokenStream2>) -> TokenStream2,
) -> syn::Result<TokenStream2> {
    if !catalogs.multiple {
        return expand(input, &catalogs.catalogs[0]);
    }
    let mut values = Vec::new();
    let mut errors: Option<syn::Error> = None;
    for catalog in &catalogs.catalogs {
        match expand(input.clone(), catalog) {
            Ok(value) => values.push(value),
            Err(error) => match &mut errors {
                Some(errors) => errors.combine(error),
                None => errors = Some(error),
            },
        }
    }
    match errors {
        Some(errors) => Err(errors),
        None if values.len() == 1 => Ok(values.remove(0)),
        None => Ok(combine(values)),
    }
}

fn combine_array(values: Vec<TokenStream2>) -> TokenStream2 {
    let index = locale_index();
    quote! { (&[#(#values),*])[#index] }
}

fn combine_match(mut values: Vec<TokenStream2>) -> TokenStream2 {
    let index = locale_index();
    let last = values.pop();
    let indexes = 0..values.len();
    quote! {
        match #index {
            #(#indexes => #values,)*
            _ => #last,
        }
    }
}

fn expand_args_localized(input: TokenStream2, catalogs: &Catalogs) -> syn::Result<TokenStream2> {
    if !catalogs.multiple {
        return expand_args(input, &catalogs.catalogs[0]);
    }
    let formatter = proc_macro2::Ident::new("formatter", Span::mixed_site());
    let body = localized(
        quote! { #formatter, #input },
        catalogs,
        expand_write,
        combine_match,
    )?;
    Ok(quote! {
        ::core::format_args!(
            "{}",
            ::embedded_i18n::__FormatWith(|#formatter: &mut ::core::fmt::Formatter<'_>| #body)
        )
    })
}

#[derive(Clone, Copy, Default)]
struct Options {
    strict: bool,
    isolate: bool,
}

const PSEUDO_LOCALE: &str = "pseudo";

fn is_enabled(value: Option<&str>) -> bool {
    value.is_some_and(|value| {
        !matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "" | "0" | "false" | "no" | "off"
        )
    })
}

fn locales_directory(manifest_directory: &Path, configured: Option<&str>) -> PathBuf {
    match configured {
        Some(path) if !path.is_empty() => manifest_directory.join(path),
        _ => manifest_directory.join("locales"),
    }
}

fn load_catalog(
    path: &Path,
    locale: &str,
    fallback: &str,
    options: Options,
) -> Result<Catalog, String> {
    if !path.is_dir() {
        return Err(format!("locales directory not found: {}", path.display()));
    }

    let chain = locale::chain(locale, fallback);
    let pseudo = chain.first().is_some_and(|first| first == PSEUDO_LOCALE);
    let mut files = Vec::new();
    let mut locales = Vec::new();

    for locale in &chain {
        if let Some((file, mut translations)) = load_locale(path, locale)? {
            if !files.contains(&file) {
                files.push(file);
                if options.isolate && transform::is_right_to_left(locale) {
                    translations.map_values(transform::isolate_placeholders);
                }
                if pseudo {
                    translations.map_values(transform::pseudo);
                }
                locales.push(Locale {
                    name: locale.clone(),
                    translations,
                });
            }
        }
    }

    Ok(Catalog {
        path: path.to_path_buf(),
        primary: if pseudo {
            chain.clone()
        } else {
            locale::chain(locale, "")
        },
        chain,
        strict: options.strict,
        files,
        locales,
    })
}

fn load_locale(path: &Path, locale: &str) -> Result<Option<(PathBuf, Translations)>, String> {
    for stem in locale::file_stems(locale) {
        for (extensions, load) in BACKENDS {
            for extension in *extensions {
                let file = path.join(format!("{}.{}", stem, extension));
                if file.is_file() {
                    return load(&file)
                        .map(|entries| Some((file.clone(), entries)))
                        .map_err(|e| format!("{}: {}", file.display(), e));
                }
            }
        }
    }

    Ok(None)
}

enum Key {
    Str(LitStr),
    CStr(LitCStr),
}

impl Key {
    fn span(&self) -> Span {
        match self {
            Key::Str(literal) => literal.span(),
            Key::CStr(literal) => literal.span(),
        }
    }

    fn value(&self) -> syn::Result<String> {
        match self {
            Key::Str(literal) => Ok(literal.value()),
            Key::CStr(literal) => literal
                .value()
                .into_string()
                .map_err(|_| syn::Error::new(literal.span(), "C string key must be valid UTF-8")),
        }
    }
}

impl Parse for Key {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match input.parse::<Lit>()? {
            Lit::Str(literal) => Ok(Key::Str(literal)),
            Lit::CStr(literal) => Ok(Key::CStr(literal)),
            other => Err(syn::Error::new(
                other.span(),
                "expected a string literal or a C string literal",
            )),
        }
    }
}

struct Message {
    context: Option<LitStr>,
    key: Key,
}

impl Message {
    fn keys(&self, name: &str) -> Vec<String> {
        match &self.context {
            Some(context) => vec![
                format!("{}{}{}", context.value(), CONTEXT_SEPARATOR, name),
                format!("{}.{}", context.value(), name),
            ],
            None => vec![name.to_string()],
        }
    }

    fn describe(&self, name: &str) -> String {
        match &self.context {
            Some(context) => format!("{:?} in context {:?}", name, context.value()),
            None => format!("{:?}", name),
        }
    }
}

impl Parse for Message {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let context = if input.peek(syn::Ident) && input.peek2(syn::Token![=]) {
            let option: syn::Ident = input.parse()?;
            if option != "context" {
                return Err(syn::Error::new(
                    option.span(),
                    format!("unknown option `{}`, expected `context`", option),
                ));
            }
            input.parse::<syn::Token![=]>()?;
            let context = input.parse()?;
            input.parse::<syn::Token![,]>()?;
            Some(context)
        } else {
            None
        };
        Ok(Message {
            context,
            key: input.parse()?,
        })
    }
}

struct Input {
    message: Message,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let message = input.parse()?;
        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
        }
        if !input.is_empty() {
            return Err(input.error("unexpected argument"));
        }
        Ok(Input { message })
    }
}

fn translation(message: &Message, catalog: &Result<Catalog, String>) -> syn::Result<TokenStream2> {
    let key = &message.key;
    let span = key.span();
    let catalog = catalog
        .as_ref()
        .map_err(|error| syn::Error::new(span, error))?;
    let name = key.value()?;

    let keys = message.keys(&name);
    let value = catalog.lookup(&keys, catalog.strict).ok_or_else(|| {
        catalog.not_found(
            span,
            format!("translation for {}", message.describe(&name)),
            catalog.lookup(&keys, false).is_some(),
        )
    })?;

    literal(key, value, &name)
}

fn literal(key: &Key, value: &str, name: &str) -> syn::Result<TokenStream2> {
    let span = key.span();
    match key {
        Key::Str(_) => {
            let value = LitStr::new(value, span);
            Ok(quote! { #value })
        }
        Key::CStr(_) => {
            let value = CString::new(value).map_err(|_| {
                syn::Error::new(
                    span,
                    format!("translation for {:?} contains a nul byte", name),
                )
            })?;
            let value = LitCStr::new(&value, span);
            Ok(quote! { #value })
        }
    }
}

fn plural_selection(
    message: &Message,
    catalog: &Result<Catalog, String>,
    arm: impl Fn(TokenStream2) -> TokenStream2,
) -> syn::Result<TokenStream2> {
    let key = &message.key;
    let span = key.span();
    let catalog = catalog
        .as_ref()
        .map_err(|error| syn::Error::new(span, error))?;
    let name = key.value()?;
    let n = proc_macro2::Ident::new("n", Span::mixed_site());

    let keys = message.keys(&name);
    let plural = catalog.plural(&keys, catalog.strict).ok_or_else(|| {
        catalog.not_found(
            span,
            format!("plural translation for {}", message.describe(&name)),
            catalog.plural(&keys, false).is_some(),
        )
    })?;

    match plural {
        Plural::Indexed { rule, forms } => {
            let germanic = Expression::germanic();
            let index = rule.unwrap_or(&germanic).tokens(&n);
            let arms = forms
                .iter()
                .map(|form| literal(key, form, &name).map(&arm))
                .collect::<syn::Result<Vec<_>>>()?;
            let first = &arms[0];
            let indexes = (0..arms.len() as u64).collect::<Vec<_>>();
            Ok(quote! {
                match #index {
                    #(#indexes => #arms,)*
                    _ => #first,
                }
            })
        }
        Plural::Categories { locale, forms } => {
            let rules = plural::rules(locale).ok_or_else(|| {
                syn::Error::new(
                    span,
                    format!(
                        "no plural rules for locale {:?}, use a gettext PO or MO file with a Plural-Forms header",
                        locale
                    ),
                )
            })?;
            let form = |category: Category| {
                forms
                    .iter()
                    .find(|(candidate, _)| *candidate == category)
                    .map(|(_, value)| *value)
            };
            let missing: Vec<&str> = rules
                .required()
                .filter(|category| form(*category).is_none())
                .map(Category::name)
                .collect();
            if !missing.is_empty() {
                return Err(syn::Error::new(
                    span,
                    format!(
                        "plural translation for {} in locale {:?} is missing the {} form{}",
                        message.describe(&name),
                        locale,
                        missing.join(", "),
                        if missing.len() > 1 { "s" } else { "" }
                    ),
                ));
            }
            let mut branches = Vec::new();
            if let Some(zero) = form(Category::Zero) {
                if !rules
                    .categories
                    .iter()
                    .any(|(category, _)| *category == Category::Zero)
                {
                    branches.push((quote! { #n == 0 }, zero));
                }
            }
            for (category, condition) in rules.categories {
                if let Some(value) = form(*category) {
                    branches.push((condition.tokens(&n), value));
                }
            }
            let other = arm(literal(
                key,
                form(Category::Other).unwrap_or_default(),
                &name,
            )?);
            let conditions = branches.iter().map(|(condition, _)| condition);
            let values = branches
                .iter()
                .map(|(_, value)| literal(key, value, &name).map(&arm))
                .collect::<syn::Result<Vec<_>>>()?;
            Ok(quote! {
                #(if #conditions { #values } else)* { #other }
            })
        }
    }
}

fn select_selection(
    message: &Message,
    value: &syn::Expr,
    catalog: &Result<Catalog, String>,
    arm: impl Fn(TokenStream2) -> TokenStream2,
) -> syn::Result<TokenStream2> {
    let key = &message.key;
    let span = key.span();
    let catalog = catalog
        .as_ref()
        .map_err(|error| syn::Error::new(span, error))?;
    let name = key.value()?;

    let keys = message.keys(&name);
    let variants = catalog.select(&keys, catalog.strict).ok_or_else(|| {
        catalog.not_found(
            span,
            format!(
                "select translation for {} with an `other` variant",
                message.describe(&name)
            ),
            catalog.select(&keys, false).is_some(),
        )
    })?;

    let mut other = None;
    let mut patterns = Vec::new();
    let mut values = Vec::new();
    for (variant, text) in variants {
        let value = arm(literal(key, text, &name)?);
        if variant == "other" {
            other = Some(value);
        } else {
            patterns.push(variant);
            values.push(value);
        }
    }

    if patterns.is_empty() {
        return Ok(quote! {
            {
                let _ = &#value;
                #other
            }
        });
    }

    Ok(quote! {
        match ::core::convert::AsRef::<str>::as_ref(&#value) {
            #(#patterns => #values,)*
            _ => #other,
        }
    })
}

struct SelectInput {
    message: Message,
    value: syn::Expr,
}

impl Parse for SelectInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let PluralInput { message, count } = input.parse()?;
        Ok(SelectInput {
            message,
            value: count,
        })
    }
}

struct SelectWriteInput {
    writer: syn::Expr,
    message: Message,
    value: syn::Expr,
    arguments: TokenStream2,
}

impl Parse for SelectWriteInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let PluralWriteInput {
            writer,
            message,
            count,
            arguments,
        } = input.parse()?;
        Ok(SelectWriteInput {
            writer,
            message,
            value: count,
            arguments,
        })
    }
}

fn expand_select(
    input: TokenStream2,
    catalog: &Result<Catalog, String>,
) -> syn::Result<TokenStream2> {
    let input: SelectInput = syn::parse2(input)?;
    select_selection(&input.message, &input.value, catalog, |literal| literal)
}

fn expand_select_write(
    input: TokenStream2,
    catalog: &Result<Catalog, String>,
) -> syn::Result<TokenStream2> {
    let input: SelectWriteInput = syn::parse2(input)?;
    let writer = &input.writer;
    let arguments = &input.arguments;
    select_selection(&input.message, &input.value, catalog, |literal| {
        quote! { ::core::write!(#writer, #literal, #arguments) }
    })
}

struct PluralInput {
    message: Message,
    count: syn::Expr,
}

impl Parse for PluralInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let message = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let count = input.parse()?;
        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
        }
        if !input.is_empty() {
            return Err(input.error("unexpected argument"));
        }
        Ok(PluralInput { message, count })
    }
}

struct PluralWriteInput {
    writer: syn::Expr,
    message: Message,
    count: syn::Expr,
    arguments: TokenStream2,
}

impl Parse for PluralWriteInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let writer = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let message: Message = input.parse()?;
        if let Key::CStr(literal) = &message.key {
            return Err(syn::Error::new(
                literal.span(),
                "expected a string literal, C strings cannot be used as format strings",
            ));
        }
        input.parse::<syn::Token![,]>()?;
        let count = input.parse()?;
        if !input.is_empty() {
            input.parse::<syn::Token![,]>()?;
        }
        Ok(PluralWriteInput {
            writer,
            message,
            count,
            arguments: input.parse()?,
        })
    }
}

fn plural_block(count: &syn::Expr, selection: TokenStream2) -> TokenStream2 {
    let n = proc_macro2::Ident::new("n", Span::mixed_site());
    quote! {
        {
            let count = #count;
            let #n: u64 = (count as i64).unsigned_abs();
            #selection
        }
    }
}

fn expand_plural(
    input: TokenStream2,
    catalog: &Result<Catalog, String>,
) -> syn::Result<TokenStream2> {
    let input: PluralInput = syn::parse2(input)?;
    let selection = plural_selection(&input.message, catalog, |literal| literal)?;
    Ok(plural_block(&input.count, selection))
}

fn expand_plural_write(
    input: TokenStream2,
    catalog: &Result<Catalog, String>,
) -> syn::Result<TokenStream2> {
    let input: PluralWriteInput = syn::parse2(input)?;
    let writer = &input.writer;
    let arguments = &input.arguments;
    let selection = plural_selection(&input.message, catalog, |literal| {
        quote! { ::core::write!(#writer, #literal, #arguments) }
    })?;
    Ok(plural_block(&input.count, selection))
}

fn expand(input: TokenStream2, catalog: &Result<Catalog, String>) -> syn::Result<TokenStream2> {
    let input: Input = syn::parse2(input)?;
    translation(&input.message, catalog)
}

struct FormatInput {
    message: Message,
    arguments: TokenStream2,
}

impl Parse for FormatInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let message: Message = input.parse()?;
        if let Key::CStr(literal) = &message.key {
            return Err(syn::Error::new(
                literal.span(),
                "expected a string literal, C strings cannot be used as format strings",
            ));
        }
        if !input.is_empty() {
            input.parse::<syn::Token![,]>()?;
        }
        Ok(FormatInput {
            message,
            arguments: input.parse()?,
        })
    }
}

struct WriteInput {
    writer: syn::Expr,
    format: FormatInput,
}

impl Parse for WriteInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let writer = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        Ok(WriteInput {
            writer,
            format: input.parse()?,
        })
    }
}

fn expand_args(
    input: TokenStream2,
    catalog: &Result<Catalog, String>,
) -> syn::Result<TokenStream2> {
    let input: FormatInput = syn::parse2(input)?;
    let format = translation(&input.message, catalog)?;
    let arguments = input.arguments;
    Ok(quote! { ::core::format_args!(#format, #arguments) })
}

fn expand_write(
    input: TokenStream2,
    catalog: &Result<Catalog, String>,
) -> syn::Result<TokenStream2> {
    let input: WriteInput = syn::parse2(input)?;
    let writer = input.writer;
    let format = translation(&input.format.message, catalog)?;
    let arguments = input.format.arguments;
    Ok(quote! { ::core::write!(#writer, #format, #arguments) })
}

fn expand_track_locales(input: TokenStream2, catalogs: &Catalogs) -> syn::Result<TokenStream2> {
    if !input.is_empty() {
        return Err(syn::Error::new_spanned(input, "unexpected argument"));
    }
    let mut files: Vec<String> = Vec::new();
    for catalog in &catalogs.catalogs {
        let catalog = catalog
            .as_ref()
            .map_err(|error| syn::Error::new(Span::call_site(), error))?;
        for file in &catalog.files {
            let file = file.to_string_lossy().into_owned();
            if !files.contains(&file) {
                files.push(file);
            }
        }
    }
    Ok(quote! {
        const _: () = {
            #(const _: &[u8] = include_bytes!(#files);)*
        };
    })
}

#[cfg_attr(
    doc,
    doc = "Makes the calling crate depend on the locale files it reads.\n\nSee [`embedded_i18n::track_locales!`](https://docs.rs/embedded-i18n/latest/embedded_i18n/macro.track_locales.html)."
)]
#[proc_macro]
pub fn track_locales(input: TokenStream) -> TokenStream {
    expand_track_locales(input.into(), &CATALOGS)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg_attr(
    doc,
    doc = "Replaces a message key with its translation, at compile time.\n\nSee [`embedded_i18n::translate!`](https://docs.rs/embedded-i18n/latest/embedded_i18n/macro.translate.html)."
)]
#[proc_macro]
pub fn translate(input: TokenStream) -> TokenStream {
    localized(input.into(), &CATALOGS, expand, combine_array)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg_attr(
    doc,
    doc = "Selects the plural form of a message matching a count.\n\nSee [`embedded_i18n::translate_plural!`](https://docs.rs/embedded-i18n/latest/embedded_i18n/macro.translate_plural.html)."
)]
#[proc_macro]
pub fn translate_plural(input: TokenStream) -> TokenStream {
    localized(input.into(), &CATALOGS, expand_plural, combine_match)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg_attr(
    doc,
    doc = "Writes the plural form of a message matching a count.\n\nSee [`embedded_i18n::translate_plural_write!`](https://docs.rs/embedded-i18n/latest/embedded_i18n/macro.translate_plural_write.html)."
)]
#[proc_macro]
pub fn translate_plural_write(input: TokenStream) -> TokenStream {
    localized(input.into(), &CATALOGS, expand_plural_write, combine_match)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg_attr(
    doc,
    doc = "Selects a variant of a message from a value.\n\nSee [`embedded_i18n::translate_select!`](https://docs.rs/embedded-i18n/latest/embedded_i18n/macro.translate_select.html)."
)]
#[proc_macro]
pub fn translate_select(input: TokenStream) -> TokenStream {
    localized(input.into(), &CATALOGS, expand_select, combine_match)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg_attr(
    doc,
    doc = "Writes the variant of a message selected from a value.\n\nSee [`embedded_i18n::translate_select_write!`](https://docs.rs/embedded-i18n/latest/embedded_i18n/macro.translate_select_write.html)."
)]
#[proc_macro]
pub fn translate_select_write(input: TokenStream) -> TokenStream {
    localized(input.into(), &CATALOGS, expand_select_write, combine_match)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg_attr(
    doc,
    doc = "Builds format arguments from a translated format string.\n\nSee [`embedded_i18n::translate_args!`](https://docs.rs/embedded-i18n/latest/embedded_i18n/macro.translate_args.html)."
)]
#[proc_macro]
pub fn translate_args(input: TokenStream) -> TokenStream {
    expand_args_localized(input.into(), &CATALOGS)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg_attr(
    doc,
    doc = "Writes a translated, formatted message.\n\nSee [`embedded_i18n::translate_write!`](https://docs.rs/embedded-i18n/latest/embedded_i18n/macro.translate_write.html)."
)]
#[proc_macro]
pub fn translate_write(input: TokenStream) -> TokenStream {
    localized(input.into(), &CATALOGS, expand_write, combine_match)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog(entries: &[(&str, &str)]) -> Result<Catalog, String> {
        Ok(Catalog {
            path: PathBuf::from("locales"),
            chain: vec!["fr".to_string(), "en".to_string()],
            primary: vec!["fr".to_string()],
            strict: false,
            files: vec![
                PathBuf::from("/locales/fr.json"),
                PathBuf::from("/locales/en.json"),
            ],
            locales: vec![Locale {
                name: "fr".to_string(),
                translations: Translations::from(
                    entries
                        .iter()
                        .map(|(key, value)| (key.to_string(), value.to_string()))
                        .collect::<HashMap<_, _>>(),
                ),
            }],
        })
    }

    #[cfg(any(
        feature = "backend-json",
        feature = "backend-yaml",
        feature = "backend-toml",
        feature = "backend-fluent"
    ))]
    fn get<'a>(catalog: &'a Catalog, key: &str) -> &'a str {
        catalog.lookup(&[key.to_string()], false).unwrap()
    }

    fn single(catalog: Result<Catalog, String>) -> Catalogs {
        Catalogs {
            multiple: false,
            catalogs: vec![catalog],
        }
    }

    fn error(input: TokenStream2, catalog: &Result<Catalog, String>) -> String {
        match expand(input, catalog) {
            Ok(tokens) => panic!("expected an error, got {}", tokens),
            Err(error) => error.to_string(),
        }
    }

    #[test]
    fn expands_string_literal() {
        let tokens = expand(quote! { "greeting" }, &catalog(&[("greeting", "Bonjour")])).unwrap();
        assert_eq!(tokens.to_string(), quote! { "Bonjour" }.to_string());
    }

    #[test]
    fn expands_c_string_literal() {
        let tokens = expand(quote! { c"greeting" }, &catalog(&[("greeting", "Bonjour")])).unwrap();
        assert_eq!(tokens.to_string(), "c\"Bonjour\"");
    }

    #[test]
    fn accepts_trailing_comma() {
        assert!(expand(quote! { "greeting", }, &catalog(&[("greeting", "Bonjour")])).is_ok());
    }

    #[test]
    fn unescapes_key() {
        let tokens = expand(
            quote! { "say \"hi\"\n" },
            &catalog(&[("say \"hi\"\n", "dis \"salut\"")]),
        )
        .unwrap();
        let value: LitStr = syn::parse2(tokens).unwrap();
        assert_eq!(value.value(), "dis \"salut\"");
    }

    #[test]
    fn escapes_value() {
        let tokens = expand(quote! { "quote" }, &catalog(&[("quote", "a \"b\" \\ c")])).unwrap();
        let value: LitStr = syn::parse2(tokens).unwrap();
        assert_eq!(value.value(), "a \"b\" \\ c");
    }

    #[test]
    fn rejects_identifier() {
        assert!(error(quote! { greeting }, &catalog(&[])).contains("expected"));
    }

    #[test]
    fn rejects_non_string_literal() {
        assert_eq!(
            error(quote! { 42 }, &catalog(&[])),
            "expected a string literal or a C string literal"
        );
    }

    #[test]
    fn rejects_extra_argument() {
        assert_eq!(
            error(quote! { "greeting" "other" }, &catalog(&[])),
            "unexpected argument"
        );
    }

    #[test]
    fn rejects_empty_input() {
        assert!(!error(quote! {}, &catalog(&[])).is_empty());
    }

    #[test]
    fn reports_missing_key() {
        assert_eq!(
            error(quote! { "missing" }, &catalog(&[])),
            "translation for \"missing\" not found in locales \"fr\", \"en\" (locales)"
        );
    }

    #[test]
    fn reports_catalog_error() {
        assert_eq!(
            error(quote! { "greeting" }, &Err("broken".to_string())),
            "broken"
        );
    }

    #[test]
    fn reports_nul_byte_in_c_string() {
        assert!(error(quote! { c"nul" }, &catalog(&[("nul", "a\0b")])).contains("nul byte"));
    }

    #[test]
    fn reports_missing_locales_directory() {
        let directory = tempfile::tempdir().unwrap();
        let error = load_catalog(
            &directory.path().join("missing"),
            "en",
            "en",
            Options::default(),
        )
        .err()
        .unwrap();
        assert!(error.starts_with("locales directory not found"));
    }

    #[cfg(feature = "backend-json")]
    #[test]
    fn reports_invalid_locale_file() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("en.json"), "{").unwrap();
        let error = load_catalog(directory.path(), "en", "en", Options::default())
            .err()
            .unwrap();
        assert!(error.contains("en.json"));
    }

    #[cfg(feature = "backend-json")]
    #[test]
    fn primary_locale_overrides_fallback() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join("fr.json"),
            r#"{"greeting": "Bonjour", "empty": ""}"#,
        )
        .unwrap();
        std::fs::write(
            directory.path().join("en.json"),
            r#"{"greeting": "Hello", "empty": "Empty", "only": "Only"}"#,
        )
        .unwrap();
        let catalog = load_catalog(directory.path(), "fr", "en", Options::default()).unwrap();
        assert_eq!(get(&catalog, "greeting"), "Bonjour");
        assert_eq!(get(&catalog, "empty"), "Empty");
        assert_eq!(get(&catalog, "only"), "Only");
    }

    #[test]
    fn track_locales_includes_loaded_files() {
        let tokens = expand_track_locales(quote! {}, &single(catalog(&[]))).unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! {
                const _: () = {
                    const _: &[u8] = include_bytes!("/locales/fr.json");
                    const _: &[u8] = include_bytes!("/locales/en.json");
                };
            }
            .to_string()
        );
    }

    #[test]
    fn track_locales_rejects_arguments() {
        let error = expand_track_locales(quote! { "locales" }, &single(catalog(&[])))
            .err()
            .unwrap();
        assert_eq!(error.to_string(), "unexpected argument");
    }

    #[cfg(feature = "backend-json")]
    #[test]
    fn catalog_lists_loaded_files_once() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("fr.json"), "{}").unwrap();
        std::fs::write(directory.path().join("en.json"), "{}").unwrap();
        let catalog = load_catalog(directory.path(), "fr", "en", Options::default()).unwrap();
        assert_eq!(
            catalog.files,
            vec![
                directory.path().join("fr.json"),
                directory.path().join("en.json")
            ]
        );
        let catalog = load_catalog(directory.path(), "en", "en", Options::default()).unwrap();
        assert_eq!(catalog.files, vec![directory.path().join("en.json")]);
    }

    #[test]
    fn args_expands_to_format_args() {
        let tokens = expand_args(
            quote! { "greeting", name = user, 42 },
            &catalog(&[("greeting", "Bonjour {name} {}")]),
        )
        .unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! { ::core::format_args!("Bonjour {name} {}", name = user, 42) }.to_string()
        );
    }

    #[test]
    fn args_without_arguments() {
        let tokens =
            expand_args(quote! { "greeting" }, &catalog(&[("greeting", "Bonjour")])).unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! { ::core::format_args!("Bonjour",) }.to_string()
        );
    }

    #[test]
    fn args_rejects_c_string() {
        assert!(
            expand_args(quote! { c"greeting" }, &catalog(&[("greeting", "Bonjour")]))
                .err()
                .unwrap()
                .to_string()
                .contains("C strings cannot be used as format strings")
        );
    }

    #[test]
    fn args_requires_comma() {
        assert!(
            expand_args(
                quote! { "greeting" 42 },
                &catalog(&[("greeting", "Bonjour")])
            )
            .is_err()
        );
    }

    #[test]
    fn write_expands_to_write() {
        let tokens = expand_write(
            quote! { &mut output, "greeting", 42 },
            &catalog(&[("greeting", "Bonjour {}")]),
        )
        .unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! { ::core::write!(&mut output, "Bonjour {}", 42) }.to_string()
        );
    }

    #[test]
    fn write_requires_key() {
        assert!(expand_write(quote! { output }, &catalog(&[])).is_err());
    }

    #[test]
    fn write_reports_missing_key() {
        assert!(
            expand_write(quote! { output, "missing" }, &catalog(&[]))
                .err()
                .unwrap()
                .to_string()
                .starts_with("translation for \"missing\" not found")
        );
    }

    #[test]
    fn locales_directory_defaults_to_locales() {
        assert_eq!(
            locales_directory(Path::new("/crate"), None),
            PathBuf::from("/crate/locales")
        );
        assert_eq!(
            locales_directory(Path::new("/crate"), Some("")),
            PathBuf::from("/crate/locales")
        );
    }

    #[test]
    fn locales_directory_relative_to_manifest() {
        assert_eq!(
            locales_directory(Path::new("/workspace/app"), Some("../i18n")),
            PathBuf::from("/workspace/app/../i18n")
        );
    }

    #[test]
    fn locales_directory_absolute() {
        assert_eq!(
            locales_directory(Path::new("/workspace/app"), Some("/shared/i18n")),
            PathBuf::from("/shared/i18n")
        );
    }

    #[cfg(feature = "backend-json")]
    #[test]
    fn region_falls_back_to_language_then_fallbacks() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join("fr_CA.json"),
            r#"{"color": "Couleur (CA)"}"#,
        )
        .unwrap();
        std::fs::write(
            directory.path().join("fr.json"),
            r#"{"color": "Couleur", "door": "Porte", "empty": ""}"#,
        )
        .unwrap();
        std::fs::write(directory.path().join("de.json"), r#"{"bus": "Bus (de)"}"#).unwrap();
        std::fs::write(
            directory.path().join("en.json"),
            r#"{"color": "Color", "door": "Door", "bus": "Bus", "train": "Train", "empty": ""}"#,
        )
        .unwrap();
        let catalog = load_catalog(directory.path(), "fr-ca", "de,en", Options::default()).unwrap();
        assert_eq!(catalog.chain, ["fr-CA", "fr", "de", "en"]);
        assert_eq!(get(&catalog, "color"), "Couleur (CA)");
        assert_eq!(get(&catalog, "door"), "Porte");
        assert_eq!(get(&catalog, "bus"), "Bus (de)");
        assert_eq!(get(&catalog, "train"), "Train");
        assert_eq!(get(&catalog, "empty"), "");
        assert_eq!(catalog.files.len(), 4);
    }

    #[cfg(feature = "backend-json")]
    #[test]
    fn locale_file_names_are_matched_case_insensitively() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("zh-cn.json"), r#"{"hello": "你好"}"#).unwrap();
        let catalog = load_catalog(directory.path(), "zh_CN", "en", Options::default()).unwrap();
        assert_eq!(get(&catalog, "hello"), "你好");
    }

    #[cfg(feature = "backend-json")]
    #[test]
    fn nested_json_keys_are_flattened() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join("en.json"),
            r#"{"menu": {"settings": {"title": "Settings"}}}"#,
        )
        .unwrap();
        let catalog = load_catalog(directory.path(), "en", "en", Options::default()).unwrap();
        assert_eq!(get(&catalog, "menu.settings.title"), "Settings");
    }

    #[cfg(feature = "backend-yaml")]
    #[test]
    fn loads_yaml_and_yml_files() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("fr.yml"), "menu:\n  open: Ouvrir\n").unwrap();
        std::fs::write(directory.path().join("en.yaml"), "menu:\n  open: Open\n").unwrap();
        let catalog = load_catalog(directory.path(), "fr", "en", Options::default()).unwrap();
        assert_eq!(get(&catalog, "menu.open"), "Ouvrir");
        assert_eq!(catalog.files.len(), 2);
    }

    #[cfg(feature = "backend-toml")]
    #[test]
    fn loads_toml_files() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join("en.toml"),
            "[menu]\nopen = \"Open\"\n",
        )
        .unwrap();
        let catalog = load_catalog(directory.path(), "en", "en", Options::default()).unwrap();
        assert_eq!(get(&catalog, "menu.open"), "Open");
    }

    #[cfg(feature = "backend-fluent")]
    #[test]
    fn loads_fluent_files() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join("en.ftl"),
            "hello = Hello, { $name }!\n",
        )
        .unwrap();
        let catalog = load_catalog(directory.path(), "en", "en", Options::default()).unwrap();
        assert_eq!(get(&catalog, "hello"), "Hello, {name}!");
    }

    #[cfg(feature = "backend-mo")]
    #[test]
    fn reports_invalid_mo_file() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("en.mo"), [0u8; 28]).unwrap();
        let error = load_catalog(directory.path(), "en", "en", Options::default())
            .err()
            .unwrap();
        assert!(error.contains("en.mo"));
    }

    #[cfg(all(feature = "backend-json", feature = "backend-po"))]
    #[test]
    fn json_takes_precedence_over_po() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("en.json"), r#"{"a": "json"}"#).unwrap();
        std::fs::write(
            directory.path().join("en.po"),
            "msgid \"a\"\nmsgstr \"po\"\n",
        )
        .unwrap();
        let catalog = load_catalog(directory.path(), "en", "en", Options::default()).unwrap();
        assert_eq!(get(&catalog, "a"), "json");
    }

    #[test]
    fn context_uses_gettext_key_then_namespace() {
        let tokens = expand(
            quote! { context = "menu", "Open" },
            &catalog(&[("menu\u{4}Open", "Ouvrir"), ("Open", "Ouvert")]),
        )
        .unwrap();
        assert_eq!(tokens.to_string(), quote! { "Ouvrir" }.to_string());
        let tokens = expand(
            quote! { context = "menu", "Open" },
            &catalog(&[("menu.Open", "Ouvrir"), ("Open", "Ouvert")]),
        )
        .unwrap();
        assert_eq!(tokens.to_string(), quote! { "Ouvrir" }.to_string());
    }

    #[test]
    fn context_does_not_fall_back_to_key_without_context() {
        assert_eq!(
            error(
                quote! { context = "menu", "Open" },
                &catalog(&[("Open", "Ouvert")])
            ),
            "translation for \"Open\" in context \"menu\" not found in locales \"fr\", \"en\" (locales)"
        );
    }

    #[test]
    fn rejects_unknown_option() {
        assert_eq!(
            error(quote! { scope = "menu", "Open" }, &catalog(&[])),
            "unknown option `scope`, expected `context`"
        );
    }

    #[test]
    fn context_requires_string_literal_and_key() {
        assert!(expand(quote! { context = menu, "Open" }, &catalog(&[])).is_err());
        assert!(expand(quote! { context = "menu" }, &catalog(&[])).is_err());
    }

    #[test]
    fn args_and_write_accept_context() {
        let catalog = catalog(&[("menu\u{4}Open", "Ouvrir {}")]);
        let tokens = expand_args(quote! { context = "menu", "Open", 1 }, &catalog).unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! { ::core::format_args!("Ouvrir {}", 1) }.to_string()
        );
        let tokens =
            expand_write(quote! { output, context = "menu", "Open", 1 }, &catalog).unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! { ::core::write!(output, "Ouvrir {}", 1) }.to_string()
        );
    }

    #[test]
    fn earlier_locale_wins_across_key_forms() {
        let catalog = Ok(Catalog {
            path: PathBuf::from("locales"),
            chain: vec!["fr".to_string(), "en".to_string()],
            primary: vec!["fr".to_string()],
            strict: false,
            files: Vec::new(),
            locales: vec![
                Locale {
                    name: "fr".to_string(),
                    translations: Translations::from(HashMap::from([(
                        "menu.Open".to_string(),
                        "Ouvrir".to_string(),
                    )])),
                },
                Locale {
                    name: "en".to_string(),
                    translations: Translations::from(HashMap::from([(
                        "menu\u{4}Open".to_string(),
                        "Open".to_string(),
                    )])),
                },
            ],
        });
        let tokens = expand(quote! { context = "menu", "Open" }, &catalog).unwrap();
        assert_eq!(tokens.to_string(), quote! { "Ouvrir" }.to_string());
    }

    fn locale_catalog(name: &str, translations: Translations) -> Result<Catalog, String> {
        Ok(Catalog {
            path: PathBuf::from("locales"),
            chain: vec![name.to_string()],
            primary: vec![name.to_string()],
            strict: false,
            files: Vec::new(),
            locales: vec![Locale {
                name: name.to_string(),
                translations,
            }],
        })
    }

    fn messages(entries: &[(&str, &str)]) -> Translations {
        Translations::from(
            entries
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect::<HashMap<_, _>>(),
        )
    }

    #[test]
    fn plural_uses_cldr_categories() {
        let catalog = locale_catalog(
            "fr",
            messages(&[("files.one", "fichier"), ("files.other", "fichiers")]),
        );
        let tokens = expand_plural(quote! { "files", total }, &catalog)
            .unwrap()
            .to_string();
        assert!(tokens.contains("let count = total"));
        assert!(tokens.contains("\"fichier\""));
        assert!(tokens.contains("\"fichiers\""));
        assert!(!tokens.contains("1_000_000"));
    }

    #[test]
    fn plural_reports_missing_categories() {
        let catalog = locale_catalog(
            "ru",
            messages(&[("files.one", "файл"), ("files.other", "файла")]),
        );
        assert_eq!(
            expand_plural(quote! { "files", 2 }, &catalog)
                .err()
                .unwrap()
                .to_string(),
            "plural translation for \"files\" in locale \"ru\" is missing the few, many forms"
        );
    }

    #[test]
    fn plural_requires_known_language() {
        let catalog = locale_catalog("tlh", messages(&[("files.other", "files")]));
        assert!(
            expand_plural(quote! { "files", 2 }, &catalog)
                .err()
                .unwrap()
                .to_string()
                .starts_with("no plural rules for locale \"tlh\"")
        );
    }

    #[test]
    fn plural_uses_gettext_forms_and_rule() {
        let mut translations = messages(&[]);
        translations.plurals.insert(
            "files".to_string(),
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        );
        translations.plural_rule = Some(Expression::parse("n == 1 ? 0 : n == 2 ? 1 : 2").unwrap());
        let tokens = expand_plural(quote! { "files", 3 }, &locale_catalog("xx", translations))
            .unwrap()
            .to_string();
        assert!(tokens.contains("match"));
        assert!(tokens.contains("0u64 => \"a\""));
        assert!(tokens.contains("2u64 => \"c\""));
        assert!(tokens.contains("_ => \"a\""));
    }

    #[test]
    fn plural_without_rule_uses_germanic_default() {
        let mut translations = messages(&[]);
        translations
            .plurals
            .insert("files".to_string(), vec!["a".to_string(), "b".to_string()]);
        let tokens = expand_plural(quote! { "files", 3 }, &locale_catalog("xx", translations))
            .unwrap()
            .to_string();
        assert!(tokens.contains("!= (1u64)"));
    }

    #[test]
    fn plural_reports_missing_message() {
        assert_eq!(
            expand_plural(quote! { "files", 2 }, &catalog(&[]))
                .err()
                .unwrap()
                .to_string(),
            "plural translation for \"files\" not found in locales \"fr\", \"en\" (locales)"
        );
    }

    #[test]
    fn plural_requires_count() {
        assert!(expand_plural(quote! { "files" }, &catalog(&[])).is_err());
        assert!(expand_plural(quote! { "files", 1, 2 }, &catalog(&[])).is_err());
    }

    #[test]
    fn plural_write_wraps_each_form() {
        let catalog = locale_catalog(
            "en",
            messages(&[
                ("files.one", "{count} file"),
                ("files.other", "{count} files"),
            ]),
        );
        let tokens = expand_plural_write(quote! { output, "files", total, extra = 1 }, &catalog)
            .unwrap()
            .to_string();
        assert!(
            tokens.contains(
                &quote! { ::core::write!(output, "{count} file", extra = 1) }.to_string()
            )
        );
        assert!(
            tokens.contains(
                &quote! { ::core::write!(output, "{count} files", extra = 1) }.to_string()
            )
        );
    }

    #[test]
    fn plural_write_rejects_c_string() {
        assert!(
            expand_plural_write(quote! { output, c"files", 1 }, &catalog(&[]))
                .err()
                .unwrap()
                .to_string()
                .contains("C strings cannot be used as format strings")
        );
    }

    #[test]
    fn plural_zero_form_overrides_when_language_has_none() {
        let catalog = locale_catalog(
            "en",
            messages(&[
                ("files.zero", "none"),
                ("files.one", "one"),
                ("files.other", "other"),
            ]),
        );
        let tokens = expand_plural(quote! { "files", 0 }, &catalog)
            .unwrap()
            .to_string();
        assert!(tokens.contains("== 0 { \"none\" }"));
    }

    #[test]
    fn select_matches_variants_and_defaults_to_other() {
        let catalog = locale_catalog(
            "en",
            messages(&[
                ("greeting.male", "He"),
                ("greeting.female", "She"),
                ("greeting.other", "They"),
                ("greeting.male.formal", "Sir"),
                ("greeting.empty", ""),
                ("greetings.other", "Unrelated"),
            ]),
        );
        let tokens = expand_select(quote! { "greeting", gender }, &catalog).unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! {
                match ::core::convert::AsRef::<str>::as_ref(&gender) {
                    "female" => "She",
                    "male" => "He",
                    _ => "They",
                }
            }
            .to_string()
        );
    }

    #[test]
    fn select_requires_other_variant() {
        let catalog = locale_catalog("en", messages(&[("greeting.male", "He")]));
        assert_eq!(
            expand_select(quote! { "greeting", gender }, &catalog)
                .err()
                .unwrap()
                .to_string(),
            "select translation for \"greeting\" with an `other` variant not found in locales \"en\" (locales)"
        );
    }

    #[test]
    fn select_variants_come_from_one_locale() {
        let catalog = Ok(Catalog {
            path: PathBuf::from("locales"),
            chain: vec!["fr".to_string(), "en".to_string()],
            primary: vec!["fr".to_string()],
            strict: false,
            files: Vec::new(),
            locales: vec![
                Locale {
                    name: "fr".to_string(),
                    translations: messages(&[("greeting.other", "Iel")]),
                },
                Locale {
                    name: "en".to_string(),
                    translations: messages(&[("greeting.male", "He"), ("greeting.other", "They")]),
                },
            ],
        });
        let tokens = expand_select(quote! { "greeting", gender }, &catalog).unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! {
                {
                    let _ = &gender;
                    "Iel"
                }
            }
            .to_string()
        );
    }

    #[test]
    fn select_with_context_and_write() {
        let catalog = locale_catalog(
            "en",
            messages(&[
                ("mail\u{4}greeting.male", "Dear {name}"),
                ("mail.greeting.other", "Hi {name}"),
            ]),
        );
        assert!(expand_select(quote! { context = "mail", "greeting", gender }, &catalog).is_ok());
        let tokens = expand_select_write(
            quote! { output, context = "mail", "greeting", gender, name = "Ada" },
            &catalog,
        )
        .unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! {
                {
                    let _ = &gender;
                    ::core::write!(output, "Hi {name}", name = "Ada")
                }
            }
            .to_string()
        );
    }

    #[test]
    fn select_write_rejects_c_string() {
        assert!(
            expand_select_write(quote! { output, c"greeting", gender }, &catalog(&[])).is_err()
        );
    }

    #[test]
    fn strict_flag_values() {
        assert!(!is_enabled(None));
        for value in ["", "0", "false", "No", " off "] {
            assert!(!is_enabled(Some(value)), "{:?}", value);
        }
        for value in ["1", "true", "yes", "ON"] {
            assert!(is_enabled(Some(value)), "{:?}", value);
        }
    }

    #[cfg(feature = "backend-json")]
    fn strict_catalog() -> Result<Catalog, String> {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join("fr.json"),
            r#"{"door": "Porte", "files": {"one": "fichier", "other": "fichiers"}, "replied": {"other": "Iel"}}"#,
        )
        .unwrap();
        std::fs::write(
            directory.path().join("en.json"),
            r#"{"door": "Door", "train": "Train", "boats": {"one": "boat", "other": "boats"}, "answered": {"other": "They"}}"#,
        )
        .unwrap();
        let catalog = load_catalog(
            directory.path(),
            "fr-CA",
            "en",
            Options {
                strict: true,
                ..Options::default()
            },
        )
        .unwrap();
        assert_eq!(catalog.primary, ["fr-CA", "fr"]);
        Ok(catalog)
    }

    #[cfg(feature = "backend-json")]
    #[test]
    fn strict_accepts_keys_of_the_primary_language() {
        let catalog = strict_catalog();
        assert!(expand(quote! { "door" }, &catalog).is_ok());
        assert!(expand_plural(quote! { "files", 2 }, &catalog).is_ok());
        assert!(expand_select(quote! { "replied", gender }, &catalog).is_ok());
    }

    #[cfg(feature = "backend-json")]
    #[test]
    fn strict_rejects_keys_only_in_fallbacks() {
        let catalog = strict_catalog();
        assert_eq!(
            error(quote! { "train" }, &catalog),
            "translation for \"train\" is missing from the primary locale \"fr-CA\", \"fr\" (EMBEDDED_I18N_STRICT is set)"
        );
        assert!(
            expand_plural(quote! { "boats", 2 }, &catalog)
                .err()
                .unwrap()
                .to_string()
                .starts_with("plural translation for \"boats\" is missing from the primary locale")
        );
        assert!(
            expand_select(quote! { "answered", gender }, &catalog)
                .err()
                .unwrap()
                .to_string()
                .contains("is missing from the primary locale")
        );
    }

    #[cfg(feature = "backend-json")]
    #[test]
    fn strict_reports_unknown_keys_as_not_found() {
        assert!(error(quote! { "unknown" }, &strict_catalog()).starts_with(
            "translation for \"unknown\" not found in locales \"fr-CA\", \"fr\", \"en\""
        ));
    }

    #[cfg(feature = "backend-json")]
    #[test]
    fn pseudo_locale_transforms_fallbacks() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join("en.json"),
            r#"{"open": "Open {name}", "files": {"one": "{count} file", "other": "{count} files"}}"#,
        )
        .unwrap();
        let catalog = load_catalog(
            directory.path(),
            "pseudo",
            "en",
            Options {
                strict: true,
                ..Options::default()
            },
        )
        .unwrap();
        assert_eq!(catalog.chain, ["pseudo", "en"]);
        assert_eq!(get(&catalog, "open"), "[Ópéñ {name} ~~]");
        assert_eq!(get(&catalog, "files.one"), "[{count} fílé ~~]");
        let catalog = Ok(catalog);
        assert!(expand(quote! { "open" }, &catalog).is_ok());
        assert!(expand_plural(quote! { "files", 2 }, &catalog).is_ok());
    }

    #[cfg(feature = "backend-po")]
    #[test]
    fn bidi_isolation_wraps_placeholders_of_right_to_left_locales() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join("ar.po"),
            "msgid \"hello\"\nmsgstr \"مرحبا {name}\"\n\nmsgid \"file\"\nmsgid_plural \"files\"\nmsgstr[0] \"{count} ملف\"\nmsgstr[1] \"{count} ملفات\"\n",
        )
        .unwrap();
        std::fs::write(
            directory.path().join("en.po"),
            "msgid \"bye\"\nmsgstr \"Bye {name}\"\n",
        )
        .unwrap();
        let options = Options {
            isolate: true,
            ..Options::default()
        };
        let catalog = load_catalog(directory.path(), "ar", "en", options).unwrap();
        assert_eq!(
            catalog.lookup(&["hello".to_string()], false).unwrap(),
            "مرحبا \u{2068}{name}\u{2069}"
        );
        assert_eq!(
            catalog.locales[0].translations.plurals["file"][1],
            "\u{2068}{count}\u{2069} ملفات"
        );
        assert_eq!(
            catalog.lookup(&["bye".to_string()], false).unwrap(),
            "Bye {name}"
        );
        let catalog = load_catalog(directory.path(), "ar", "en", Options::default()).unwrap();
        assert_eq!(
            catalog.lookup(&["hello".to_string()], false).unwrap(),
            "مرحبا {name}"
        );
    }

    fn multiple(locales: Vec<Result<Catalog, String>>) -> Catalogs {
        Catalogs {
            multiple: true,
            catalogs: locales,
        }
    }

    fn english_and_french() -> Catalogs {
        multiple(vec![
            locale_catalog(
                "en",
                messages(&[
                    ("greeting", "Hello {name}"),
                    ("files.one", "file"),
                    ("files.other", "files"),
                ]),
            ),
            locale_catalog(
                "fr",
                messages(&[
                    ("greeting", "Bonjour {name}"),
                    ("files.one", "fichier"),
                    ("files.other", "fichiers"),
                ]),
            ),
        ])
    }

    #[test]
    fn parses_embedded_locales() {
        assert_eq!(embedded_locales(" en, fr_CA ,,de "), ["en", "fr_CA", "de"]);
        assert!(embedded_locales("").is_empty());
        assert!(embedded_locales(" , ").is_empty());
    }

    #[test]
    fn single_mode_keeps_literals() {
        let tokens = localized(
            quote! { "greeting" },
            &single(catalog(&[("greeting", "Bonjour")])),
            expand,
            combine_array,
        )
        .unwrap();
        assert_eq!(tokens.to_string(), quote! { "Bonjour" }.to_string());
    }

    #[test]
    fn multiple_mode_indexes_an_array() {
        let tokens = localized(
            quote! { "greeting" },
            &english_and_french(),
            expand,
            combine_array,
        )
        .unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! {
                (&["Hello {name}", "Bonjour {name}"])[::embedded_i18n::locale_index()]
            }
            .to_string()
        );
    }

    #[test]
    fn multiple_mode_matches_on_the_locale() {
        let tokens = localized(
            quote! { output, "greeting", name = "Ada" },
            &english_and_french(),
            expand_write,
            combine_match,
        )
        .unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! {
                match ::embedded_i18n::locale_index() {
                    0usize => ::core::write!(output, "Hello {name}", name = "Ada"),
                    _ => ::core::write!(output, "Bonjour {name}", name = "Ada"),
                }
            }
            .to_string()
        );
        assert!(
            localized(
                quote! { "files", 2 },
                &english_and_french(),
                expand_plural,
                combine_match
            )
            .unwrap()
            .to_string()
            .contains("\"fichiers\"")
        );
    }

    #[test]
    fn multiple_mode_formats_through_a_closure() {
        let tokens =
            expand_args_localized(quote! { "greeting", name = "Ada" }, &english_and_french())
                .unwrap()
                .to_string();
        assert!(
            tokens
                .starts_with(":: core :: format_args ! (\"{}\" , :: embedded_i18n :: __FormatWith")
        );
        assert!(tokens.contains("\"Bonjour {name}\""));
    }

    #[test]
    fn multiple_mode_reports_every_missing_locale() {
        let catalogs = multiple(vec![
            locale_catalog("en", messages(&[])),
            locale_catalog("fr", messages(&[])),
        ]);
        let error = localized(quote! { "greeting" }, &catalogs, expand, combine_array)
            .err()
            .unwrap();
        let messages: Vec<String> = error.into_iter().map(|error| error.to_string()).collect();
        assert_eq!(
            messages,
            [
                "translation for \"greeting\" not found in locales \"en\" (locales)",
                "translation for \"greeting\" not found in locales \"fr\" (locales)",
            ]
        );
    }

    #[test]
    fn single_embedded_locale_needs_no_dispatch() {
        let catalogs = multiple(vec![locale_catalog(
            "fr",
            messages(&[("greeting", "Salut")]),
        )]);
        let tokens = localized(quote! { "greeting" }, &catalogs, expand, combine_array).unwrap();
        assert_eq!(tokens.to_string(), quote! { "Salut" }.to_string());
    }

    #[test]
    fn track_locales_lists_files_of_every_locale() {
        let with_files = |name: &str, files: &[&str]| {
            Ok(Catalog {
                path: PathBuf::from("locales"),
                chain: vec![name.to_string()],
                primary: vec![name.to_string()],
                strict: false,
                files: files.iter().map(PathBuf::from).collect(),
                locales: Vec::new(),
            })
        };
        let catalogs = multiple(vec![
            with_files("fr", &["/l/fr.json", "/l/en.json"]),
            with_files("de", &["/l/de.json", "/l/en.json"]),
        ]);
        let tokens = expand_track_locales(quote! {}, &catalogs)
            .unwrap()
            .to_string();
        assert_eq!(tokens.matches("include_bytes").count(), 3);
    }
}
