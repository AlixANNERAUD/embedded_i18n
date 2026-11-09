//! Procedural macros of [embedded-i18n](https://docs.rs/embedded-i18n).
//!
//! This crate is an implementation detail and is not meant to be used
//! directly: depend on `embedded-i18n`, which re-exports and documents the
//! macros, and provides the runtime items their expansions refer to. Its API
//! and versioning follow `embedded-i18n`.
//!
//! `i18n!` resolves the configuration of the calling crate (its arguments,
//! `[package.metadata.i18n]` and the `EMBEDDED_I18N_LOCALE` /
//! `EMBEDDED_I18N_LOCALES` overrides) and hands it to a crate-local
//! `macro_rules!`, through which `t!`, `locale!` and `available_locales!`
//! reach `__expand!`. `__expand!` reads the locale files at compile time and
//! expands to string literals or to selections between them. The locale file
//! formats are enabled by the `backend-*` features, which `embedded-i18n`
//! forwards.
#![cfg_attr(doc, deny(missing_docs))]

use std::collections::HashMap;
use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex, PoisonError};

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Ident, Literal, Span, TokenStream as TokenStream2, TokenTree};
use quote::{ToTokens, quote};
use syn::{LitCStr, LitStr};

use call::{Call, Key, LocaleArgument};
use config::{Config, Environment};
use plural::{Category, Expression};

mod call;
mod config;
#[cfg(feature = "backend-fluent")]
mod fluent;
#[cfg(feature = "backend-json")]
mod json;
mod locale;
mod locale_id;
#[cfg(feature = "backend-mo")]
mod mo;
mod placeholder;
#[cfg_attr(
    not(any(feature = "backend-po", feature = "backend-mo")),
    allow(dead_code)
)]
mod plural;
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
                "{} is missing from the primary locale {} (strict mode)",
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

#[derive(Clone, Copy, Default)]
struct Options {
    strict: bool,
    isolate: bool,
}

const PSEUDO_LOCALE: &str = "pseudo";

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

/// The catalogs of a configuration, one per available locale, in configuration order.
type Catalogs = Arc<Vec<Result<Catalog, String>>>;

/// Catalogs already loaded by this proc-macro process, by configuration.
static CACHE: LazyLock<Mutex<HashMap<Config, Catalogs>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn load_catalogs(config: &Config) -> Vec<Result<Catalog, String>> {
    let fallback = config.fallback.join(",");
    let options = Options {
        strict: config.strict,
        isolate: config.bidi_isolation,
    };
    config
        .available_locales
        .iter()
        .map(|locale| load_catalog(&config.path, locale, &fallback, options))
        .collect()
}

/// The catalogs of `config`, loaded on first use.
fn catalogs(config: &Config) -> Catalogs {
    let mut cache = CACHE.lock().unwrap_or_else(PoisonError::into_inner);
    cache
        .entry(config.clone())
        .or_insert_with(|| Arc::new(load_catalogs(config)))
        .clone()
}

/// Index of the default locale among the available locales.
fn default_index(config: &Config) -> usize {
    config
        .available_locales
        .iter()
        .position(|locale| *locale == config.default_locale)
        .unwrap_or(0)
}

/// Adds `error` to the errors collected so far.
fn push_error(errors: &mut Option<syn::Error>, error: syn::Error) {
    match errors {
        Some(errors) => errors.combine(error),
        None => *errors = Some(error),
    }
}

/// Resolves and validates the configuration of an `i18n!` call.
fn i18n_config(
    args: TokenStream2,
    manifest_dir: &Path,
    environment: &Environment,
) -> syn::Result<Config> {
    let config = Config::from_i18n_args(args, manifest_dir, environment)?;
    let mut names = std::iter::once(&config.default_locale)
        .chain(&config.available_locales)
        .chain(&config.fallback);
    if names.any(|name| name.trim().is_empty()) {
        return Err(syn::Error::new(
            Span::call_site(),
            "locale names cannot be empty",
        ));
    }
    locale_id::check_distinct(&config.available_locales)
        .map_err(|error| syn::Error::new(Span::call_site(), error))?;
    Ok(config)
}

/// Expands `i18n!`: the crate-local configuration macro, rebuild tracking and,
/// with several locales, the crate's resolution function.
fn expand_i18n(
    args: TokenStream2,
    manifest_dir: &Path,
    environment: &Environment,
) -> syn::Result<TokenStream2> {
    let config = i18n_config(args, manifest_dir, environment)?;

    let mut files: Vec<PathBuf> = Vec::new();
    let mut messages: Vec<&str> = Vec::new();
    let loaded = catalogs(&config);
    for catalog in loaded.iter() {
        match catalog {
            Ok(catalog) => {
                for file in &catalog.files {
                    if !files.contains(file) {
                        files.push(file.clone());
                    }
                }
            }
            Err(message) => {
                if !messages.contains(&message.as_str()) {
                    messages.push(message);
                }
            }
        }
    }
    let mut errors = None;
    for message in messages {
        push_error(&mut errors, syn::Error::new(Span::call_site(), message));
    }
    if let Some(errors) = errors {
        return Err(errors);
    }

    let manifest = Config::manifest_file(manifest_dir);
    if manifest.is_file() {
        files.push(manifest);
    }
    let files = files.iter().map(|file| file.to_string_lossy().into_owned());
    let config_tokens = config.to_tokens();
    let resolver = config
        .is_multiple()
        .then(|| locale_id::resolver(&config.available_locales, default_index(&config)));

    Ok(quote! {
        #[doc(hidden)]
        #[allow(unused_macros)]
        macro_rules! embedded_i18n_requires_i18n_at_the_crate_root {
            ($($tokens:tt)*) => {
                ::embedded_i18n::__expand!(#config_tokens $($tokens)*)
            };
        }
        #(const _: &[u8] = ::core::include_bytes!(#files);)*
        const _: ::core::option::Option<&str> = ::core::option_env!("EMBEDDED_I18N_LOCALE");
        const _: ::core::option::Option<&str> = ::core::option_env!("EMBEDDED_I18N_LOCALES");
        #resolver
        #[doc(hidden)]
        #[allow(dead_code)]
        const EMBEDDED_I18N_I18N_IS_CALLED_ONCE_PER_CRATE: () = ();
    })
}

/// Expands `__expand!({ <configuration> } @t ...)`, `(.. @locale)` or `(.. @available)`.
fn expand_request(input: TokenStream2) -> syn::Result<TokenStream2> {
    let malformed = || {
        syn::Error::new(
            Span::call_site(),
            "expected the configuration generated by `i18n!`, then `@t`, `@locale` or `@available`",
        )
    };
    let mut tokens = input.into_iter();
    let config = match tokens.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
            Config::from_resolved(TokenTree::Group(group).into())?
        }
        _ => return Err(malformed()),
    };
    let marker = match (tokens.next(), tokens.next()) {
        (Some(TokenTree::Punct(at)), Some(TokenTree::Ident(marker))) if at.as_char() == '@' => {
            marker
        }
        _ => return Err(malformed()),
    };
    let rest: TokenStream2 = tokens.collect();
    let no_argument = |rest: TokenStream2| match rest.into_iter().next() {
        Some(token) => Err(syn::Error::new(token.span(), "unexpected argument")),
        None => Ok(()),
    };
    match marker.to_string().as_str() {
        "t" => {
            let call: Call = syn::parse2(rest)?;
            expand_t(&config, &catalogs(&config), &call)
        }
        "locale" => no_argument(rest).map(|()| expand_locale(&config)),
        "available" => no_argument(rest).map(|()| expand_available(&config)),
        _ => Err(malformed()),
    }
}

/// The index of the current locale, resolved by the crate's resolution function.
fn current_index() -> TokenStream2 {
    quote! { crate::__embedded_i18n_resolve(::embedded_i18n::__current_locale()) }
}

/// Expands `locale!()`.
fn expand_locale(config: &Config) -> TokenStream2 {
    if !config.is_multiple() {
        let locale = &config.default_locale;
        return quote! { #locale };
    }
    let locales = &config.available_locales;
    let index = current_index();
    quote! { (&[#(#locales),*])[#index] }
}

/// Expands `available_locales!()`.
fn expand_available(config: &Config) -> TokenStream2 {
    let locales = &config.available_locales;
    let binding = Ident::new("locales", Span::mixed_site());
    quote! {
        {
            let #binding: &'static [&'static str] = &[#(#locales),*];
            #binding
        }
    }
}

/// Wraps an expression in an invisible group, so that it keeps its precedence.
fn grouped(expr: &syn::Expr) -> TokenStream2 {
    Group::new(Delimiter::None, expr.to_token_stream()).into_token_stream()
}

/// Which locale a `t!` call translates into.
enum Target {
    /// One locale known at compile time; `discard` is a locale expression
    /// evaluated for its side effects only (single-locale mode).
    Fixed {
        index: usize,
        discard: Option<TokenStream2>,
    },
    /// The locale index computed at runtime by this expression.
    Runtime(TokenStream2),
}

fn target(config: &Config, call: &Call) -> Target {
    let default = default_index(config);
    if !config.is_multiple() {
        let discard = match &call.locale {
            Some(LocaleArgument::Expr(expr)) => {
                let expr = grouped(expr);
                Some(quote! {
                    let _: ::embedded_i18n::Locale = ::core::convert::Into::into(#expr);
                })
            }
            _ => None,
        };
        return Target::Fixed {
            index: default,
            discard,
        };
    }
    match &call.locale {
        Some(LocaleArgument::Literal(tag)) => Target::Fixed {
            index: locale_id::resolve(&config.available_locales, &tag.value()).unwrap_or(default),
            discard: None,
        },
        Some(LocaleArgument::Expr(expr)) => {
            let expr = grouped(expr);
            Target::Runtime(quote! {
                crate::__embedded_i18n_resolve(::embedded_i18n::Locale::from(#expr).id())
            })
        }
        None => Target::Runtime(current_index()),
    }
}

/// Runs `expand` for each index, collecting every error.
fn each<T>(
    indexes: &[usize],
    mut expand: impl FnMut(usize) -> syn::Result<T>,
) -> syn::Result<Vec<T>> {
    let mut values = Vec::new();
    let mut errors = None;
    for &index in indexes {
        match expand(index) {
            Ok(value) => values.push(value),
            Err(error) => push_error(&mut errors, error),
        }
    }
    match errors {
        Some(errors) => Err(errors),
        None => Ok(values),
    }
}

/// `match index { 0 => v0, .., _ => vN }`.
fn match_index(index: &TokenStream2, mut values: Vec<TokenStream2>) -> TokenStream2 {
    let last = values.pop();
    let indexes = (0..values.len()).map(Literal::usize_unsuffixed);
    quote! {
        match #index {
            #(#indexes => #values,)*
            _ => #last,
        }
    }
}

/// Expands `t!`.
fn expand_t(
    config: &Config,
    catalogs: &[Result<Catalog, String>],
    call: &Call,
) -> syn::Result<TokenStream2> {
    let name = call.key.value()?;
    let target = target(config, call);
    let indexes: Vec<usize> = match &target {
        Target::Fixed { index, .. } => vec![*index],
        Target::Runtime(_) => (0..catalogs.len()).collect(),
    };
    if call.returns_str() {
        expand_str(catalogs, call, &name, target, &indexes)
    } else {
        expand_display(config, catalogs, call, &name, target, &indexes)
    }
}

/// Expands a `t!` call without argument nor `count` to a `&'static str` (or `CStr`).
fn expand_str(
    catalogs: &[Result<Catalog, String>],
    call: &Call,
    name: &str,
    target: Target,
    indexes: &[usize],
) -> syn::Result<TokenStream2> {
    let select = call.select.as_ref().map(grouped);
    let arm = |text: &str| literal(&call.key, text, name);
    let mut values = each(indexes, |index| {
        selection(call, name, &catalogs[index], select.as_ref(), None, &arm)
    })?;
    Ok(match target {
        Target::Fixed { discard: None, .. } => values.remove(0),
        Target::Fixed {
            discard: Some(discard),
            ..
        } => {
            let value = values.remove(0);
            quote! { { #discard #value } }
        }
        Target::Runtime(index) if select.is_none() => quote! { (&[#(#values),*])[#index] },
        Target::Runtime(index) => match_index(&index, values),
    })
}

/// Expands a `t!` call with arguments or `count` to a value implementing `Display`.
fn expand_display(
    config: &Config,
    catalogs: &[Result<Catalog, String>],
    call: &Call,
    name: &str,
    target: Target,
    indexes: &[usize],
) -> syn::Result<TokenStream2> {
    let mixed = Span::mixed_site();
    let formatter = Ident::new("f", mixed);
    let count = Ident::new("__count", mixed);
    let select = Ident::new("__select", mixed);
    let index = Ident::new("__index", mixed);

    let mut values = Vec::new();
    let mut patterns = Vec::new();
    // Placeholder name and binding of each value usable by the translations.
    let mut bindings: Vec<(&str, Span, Ident)> = Vec::new();
    for (position, argument) in call.arguments.iter().enumerate() {
        let binding = Ident::new(&format!("__a{position}"), mixed);
        let value = grouped(&argument.value);
        values.push(quote! { &#value });
        patterns.push(binding.to_token_stream());
        bindings.push((&argument.name, argument.span, binding));
    }
    if let Some(value) = &call.count {
        let value = grouped(value);
        values.push(quote! { &#value });
        patterns.push(count.to_token_stream());
        bindings.push(("count", call.key.span(), count.clone()));
    }
    if let Some(value) = &call.select {
        let value = grouped(value);
        values.push(quote! { &#value });
        patterns.push(select.to_token_stream());
    }
    match &target {
        Target::Fixed {
            discard: Some(discard),
            ..
        } => {
            values.push(quote! { { #discard } });
            patterns.push(quote! { _ });
        }
        Target::Fixed { discard: None, .. } => {}
        Target::Runtime(expression) => {
            values.push(expression.clone());
            patterns.push(index.to_token_stream());
        }
    }

    let names: Vec<&str> = bindings.iter().map(|(name, _, _)| *name).collect();
    let specs: Vec<(&str, &str)> = call
        .arguments
        .iter()
        .filter_map(|argument| Some((argument.name.as_str(), argument.spec.as_deref()?)))
        .collect();
    let span = call.key.span();
    let select = call.select.as_ref().map(|_| select.to_token_stream());

    let mut bodies = each(indexes, |position| {
        let locale = config.available_locales[position].as_str();
        let arm = |text: &str| -> syn::Result<TokenStream2> {
            let template = placeholder::template(text).map_err(|error| {
                syn::Error::new(
                    span,
                    format!("translation of {name:?} in locale {locale:?}: {error}"),
                )
            })?;
            placeholder::check(&template, &names, name, locale)
                .map_err(|error| syn::Error::new(span, error))?;
            let format = LitStr::new(&placeholder::apply_specs(&template, &specs), span);
            let arguments = template.placeholders.iter().map(|placeholder| {
                let (_, span, binding) = bindings
                    .iter()
                    .find(|(name, _, _)| name == placeholder)
                    .expect("checked against the argument names");
                let placeholder = Ident::new(placeholder, *span);
                quote! { #placeholder = #binding }
            });
            Ok(quote! {
                #formatter.write_fmt(::core::format_args!(#format #(, #arguments)*))
            })
        };
        let count = call.count.as_ref().map(|_| &count);
        selection(
            call,
            name,
            &catalogs[position],
            select.as_ref(),
            count,
            &arm,
        )
    })?;

    let body = match &target {
        Target::Fixed { .. } => bodies.remove(0),
        Target::Runtime(_) => match_index(&index.to_token_stream(), bodies),
    };
    Ok(quote! {
        match (#(#values,)*) {
            (#(#patterns,)*) => ::embedded_i18n::__FormatWith(
                move |#formatter: &mut ::core::fmt::Formatter<'_>| -> ::core::fmt::Result {
                    #body
                }
            ),
        }
    })
}

/// The translation of a call in one catalog: a select variant, a plural form
/// (when `count` is given and the message has plural forms) or the message,
/// each turned into an expression by `arm`.
fn selection(
    call: &Call,
    name: &str,
    catalog: &Result<Catalog, String>,
    select: Option<&TokenStream2>,
    count: Option<&Ident>,
    arm: &dyn Fn(&str) -> syn::Result<TokenStream2>,
) -> syn::Result<TokenStream2> {
    let span = call.key.span();
    let catalog = catalog
        .as_ref()
        .map_err(|error| syn::Error::new(span, error))?;
    let keys = call.keys(name);

    if let Some(value) = select {
        return select_selection(call, name, catalog, value, arm);
    }
    if let Some(count) = count {
        if catalog.plural(&keys, false).is_some() {
            let n = Ident::new("n", Span::mixed_site());
            let selection = plural_selection(call, name, catalog, arm)?;
            return Ok(quote! {
                {
                    let #n: u64 = (*#count as i64).unsigned_abs();
                    #selection
                }
            });
        }
    }

    let value = catalog.lookup(&keys, catalog.strict).ok_or_else(|| {
        catalog.not_found(
            span,
            format!("translation for {}", call.describe(name)),
            catalog.lookup(&keys, false).is_some(),
        )
    })?;
    arm(value)
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
    call: &Call,
    name: &str,
    catalog: &Catalog,
    arm: &dyn Fn(&str) -> syn::Result<TokenStream2>,
) -> syn::Result<TokenStream2> {
    let span = call.key.span();
    let n = Ident::new("n", Span::mixed_site());

    let keys = call.keys(name);
    let plural = catalog.plural(&keys, catalog.strict).ok_or_else(|| {
        catalog.not_found(
            span,
            format!("plural translation for {}", call.describe(name)),
            catalog.plural(&keys, false).is_some(),
        )
    })?;

    match plural {
        Plural::Indexed { rule, forms } => {
            let germanic = Expression::germanic();
            let index = rule.unwrap_or(&germanic).tokens(&n);
            let arms = forms
                .iter()
                .map(|form| arm(form))
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
                        call.describe(name),
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
            let other = arm(form(Category::Other).unwrap_or_default())?;
            let conditions = branches.iter().map(|(condition, _)| condition);
            let values = branches
                .iter()
                .map(|(_, value)| arm(value))
                .collect::<syn::Result<Vec<_>>>()?;
            Ok(quote! {
                #(if #conditions { #values } else)* { #other }
            })
        }
    }
}

fn select_selection(
    call: &Call,
    name: &str,
    catalog: &Catalog,
    value: &TokenStream2,
    arm: &dyn Fn(&str) -> syn::Result<TokenStream2>,
) -> syn::Result<TokenStream2> {
    let span = call.key.span();
    let keys = call.keys(name);
    let variants = catalog.select(&keys, catalog.strict).ok_or_else(|| {
        catalog.not_found(
            span,
            format!(
                "select translation for {} with an `other` variant",
                call.describe(name)
            ),
            catalog.select(&keys, false).is_some(),
        )
    })?;

    let mut other = None;
    let mut patterns = Vec::new();
    let mut values = Vec::new();
    for (variant, text) in variants {
        let value = arm(text)?;
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

#[cfg_attr(
    doc,
    doc = "Configures the translations of the calling crate.\n\nSee [`embedded_i18n::i18n!`](https://docs.rs/embedded-i18n/latest/embedded_i18n/macro.i18n.html)."
)]
#[proc_macro]
pub fn i18n(input: TokenStream) -> TokenStream {
    let result = match std::env::var_os("CARGO_MANIFEST_DIR") {
        Some(manifest_dir) => {
            let environment = Environment {
                locale: std::env::var("EMBEDDED_I18N_LOCALE").ok(),
                locales: std::env::var("EMBEDDED_I18N_LOCALES").ok(),
            };
            expand_i18n(input.into(), Path::new(&manifest_dir), &environment)
        }
        None => Err(syn::Error::new(
            Span::call_site(),
            "CARGO_MANIFEST_DIR is not set",
        )),
    };
    result.unwrap_or_else(syn::Error::into_compile_error).into()
}

#[cfg_attr(
    doc,
    doc = "Expands `t!`, `locale!` and `available_locales!` with the configuration forwarded by `i18n!`."
)]
#[doc(hidden)]
#[proc_macro]
pub fn __expand(input: TokenStream) -> TokenStream {
    expand_request(input.into())
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

    /// A resolved configuration embedding `locales`, the first one being the default.
    fn config(locales: &[&str]) -> Config {
        Config {
            path: PathBuf::from("locales"),
            fallback: vec!["en".to_string()],
            default_locale: locales[0].to_string(),
            available_locales: locales.iter().map(|locale| locale.to_string()).collect(),
            strict: false,
            bidi_isolation: false,
        }
    }

    /// A configuration and its catalogs, one per available locale.
    struct Fixture {
        config: Config,
        catalogs: Vec<Result<Catalog, String>>,
    }

    impl Fixture {
        fn expand(&self, input: TokenStream2) -> syn::Result<TokenStream2> {
            let call: Call = syn::parse2(input)?;
            expand_t(&self.config, &self.catalogs, &call)
        }
    }

    /// Single-locale mode: the catalog is the only available locale, `fr`.
    fn single(catalog: Result<Catalog, String>) -> Fixture {
        Fixture {
            config: config(&["fr"]),
            catalogs: vec![catalog],
        }
    }

    /// Expands a `t!` call in single-locale mode against `catalog`.
    fn expand(input: TokenStream2, catalog: &Result<Catalog, String>) -> syn::Result<TokenStream2> {
        let call: Call = syn::parse2(input)?;
        expand_t(&config(&["fr"]), std::slice::from_ref(catalog), &call)
    }

    fn error(input: TokenStream2, catalog: &Result<Catalog, String>) -> String {
        match expand(input, catalog) {
            Ok(tokens) => panic!("expected an error, got {}", tokens),
            Err(error) => error.to_string(),
        }
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

    fn english_and_french() -> Fixture {
        Fixture {
            config: config(&["en", "fr"]),
            catalogs: vec![
                locale_catalog(
                    "en",
                    messages(&[
                        ("greeting", "Hello {name}"),
                        ("files.one", "{count} file"),
                        ("files.other", "{count} files"),
                    ]),
                ),
                locale_catalog(
                    "fr",
                    messages(&[
                        ("greeting", "Bonjour {name}"),
                        ("files.one", "{count} fichier"),
                        ("files.other", "{count} fichiers"),
                    ]),
                ),
            ],
        }
    }

    const CURRENT_INDEX: &str =
        "crate :: __embedded_i18n_resolve (:: embedded_i18n :: __current_locale ())";

    #[test]
    fn expands_a_literal_in_single_mode() {
        let tokens = single(catalog(&[("greeting", "Bonjour")]))
            .expand(quote! { "greeting" })
            .unwrap();
        assert_eq!(tokens.to_string(), quote! { "Bonjour" }.to_string());
    }

    #[test]
    fn expands_an_array_in_multi_mode() {
        let tokens = english_and_french().expand(quote! { "greeting" }).unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! {
                (&["Hello {name}", "Bonjour {name}"])[crate::__embedded_i18n_resolve(::embedded_i18n::__current_locale())]
            }
            .to_string()
        );
    }

    #[test]
    fn resolves_literal_locales_at_compile_time() {
        let fixture = english_and_french();
        let tokens = fixture
            .expand(quote! { "greeting", locale = "fr-CA" })
            .unwrap();
        assert_eq!(tokens.to_string(), quote! { "Bonjour {name}" }.to_string());
        let tokens = fixture
            .expand(quote! { "greeting", locale = "de" })
            .unwrap();
        assert_eq!(tokens.to_string(), quote! { "Hello {name}" }.to_string());
        let tokens = fixture
            .expand(quote! { "greeting", name = user, locale = "fr" })
            .unwrap()
            .to_string();
        assert!(!tokens.contains("__embedded_i18n_resolve"), "{}", tokens);
        assert!(tokens.contains("\"Bonjour {name}\""), "{}", tokens);
        assert!(!tokens.contains("Hello"), "{}", tokens);
    }

    #[test]
    fn formats_only_used_arguments() {
        let fixture = Fixture {
            config: config(&["en"]),
            catalogs: vec![locale_catalog(
                "en",
                messages(&[("files.one", "One file"), ("files.other", "{count} files")]),
            )],
        };
        let tokens = fixture
            .expand(quote! { "files", count = total })
            .unwrap()
            .to_string();
        assert!(
            tokens.contains(&quote! { ::core::format_args!("One file") }.to_string()),
            "{}",
            tokens
        );
        assert!(
            tokens.contains(
                &quote! { ::core::format_args!("{count} files", count = __count) }.to_string()
            ),
            "{}",
            tokens
        );
        assert!(
            tokens.contains(&quote! { let n: u64 = (*__count as i64).unsigned_abs(); }.to_string()),
            "{}",
            tokens
        );
    }

    #[test]
    fn reports_missing_placeholder_arguments() {
        assert_eq!(
            error(
                quote! { "with_named_args", name = "Ada" },
                &catalog(&[(
                    "with_named_args",
                    "Bonjour {name}, vous avez {count} messages"
                )])
            ),
            "translation of \"with_named_args\" in locale \"fr\" uses the placeholder {count}, which is not an argument of t!"
        );
    }

    #[cfg(feature = "backend-json")]
    fn crate_with_locales(manifest: &str) -> tempfile::TempDir {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("Cargo.toml"), manifest).unwrap();
        std::fs::create_dir(directory.path().join("locales")).unwrap();
        std::fs::write(
            directory.path().join("locales/en.json"),
            r#"{"greeting": "Hello"}"#,
        )
        .unwrap();
        std::fs::write(
            directory.path().join("locales/fr.json"),
            r#"{"greeting": "Bonjour"}"#,
        )
        .unwrap();
        directory
    }

    #[cfg(feature = "backend-json")]
    #[test]
    fn i18n_emits_the_config_macro_and_tracking() {
        let directory = crate_with_locales("[package]\nname = \"demo\"\n");
        let root = directory.path();
        let tokens = expand_i18n(
            quote! { available_locales = ["en", "fr"] },
            root,
            &Environment::default(),
        )
        .unwrap();

        let config = Config::from_i18n_args(
            quote! { available_locales = ["en", "fr"] },
            root,
            &Environment::default(),
        )
        .unwrap();
        let config_tokens = config.to_tokens();
        let en = root.join("locales/en.json").to_string_lossy().into_owned();
        let fr = root.join("locales/fr.json").to_string_lossy().into_owned();
        let manifest = root.join("Cargo.toml").to_string_lossy().into_owned();
        let resolver = locale_id::resolver(&config.available_locales, 0);
        let expected = quote! {
            #[doc(hidden)]
            #[allow(unused_macros)]
            macro_rules! embedded_i18n_requires_i18n_at_the_crate_root {
                ($($tokens:tt)*) => {
                    ::embedded_i18n::__expand!(#config_tokens $($tokens)*)
                };
            }
            const _: &[u8] = ::core::include_bytes!(#en);
            const _: &[u8] = ::core::include_bytes!(#fr);
            const _: &[u8] = ::core::include_bytes!(#manifest);
            const _: ::core::option::Option<&str> = ::core::option_env!("EMBEDDED_I18N_LOCALE");
            const _: ::core::option::Option<&str> = ::core::option_env!("EMBEDDED_I18N_LOCALES");
            #resolver
            #[doc(hidden)]
            #[allow(dead_code)]
            const EMBEDDED_I18N_I18N_IS_CALLED_ONCE_PER_CRATE: () = ();
        };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[cfg(feature = "backend-json")]
    #[test]
    fn i18n_single_locale_has_no_resolver_and_tracks_without_manifest() {
        let directory = crate_with_locales("");
        std::fs::remove_file(directory.path().join("Cargo.toml")).unwrap();
        let tokens = expand_i18n(quote! {}, directory.path(), &Environment::default())
            .unwrap()
            .to_string();
        assert!(!tokens.contains("__embedded_i18n_resolve"), "{}", tokens);
        assert!(!tokens.contains("Cargo.toml"), "{}", tokens);
        assert_eq!(tokens.matches("include_bytes").count(), 1, "{}", tokens);
    }

    #[cfg(feature = "backend-json")]
    #[test]
    fn i18n_follows_the_environment() {
        let directory = crate_with_locales("");
        let environment = Environment {
            locale: Some("fr".to_string()),
            locales: Some("en,fr".to_string()),
        };
        let tokens = expand_i18n(quote! {}, directory.path(), &environment)
            .unwrap()
            .to_string();
        assert!(tokens.contains("default_locale = \"fr\""), "{}", tokens);
        assert!(tokens.contains("__embedded_i18n_resolve"), "{}", tokens);
    }

    #[test]
    fn i18n_rejects_empty_locale_names() {
        let directory = tempfile::tempdir().unwrap();
        let error = |args: TokenStream2| {
            expand_i18n(args, directory.path(), &Environment::default())
                .unwrap_err()
                .to_string()
        };
        assert_eq!(
            error(quote! { default_locale = "", available_locales = [""] }),
            "locale names cannot be empty"
        );
        assert_eq!(
            error(quote! { available_locales = ["en", " "] }),
            "locale names cannot be empty"
        );
        assert_eq!(
            error(quote! { fallback = ["en", ""] }),
            "locale names cannot be empty"
        );
    }

    #[test]
    fn i18n_rejects_identical_locale_identifiers() {
        let directory = tempfile::tempdir().unwrap();
        let error = expand_i18n(
            quote! { default_locale = "fr-CA", available_locales = ["fr-CA", "fr_ca"] },
            directory.path(),
            &Environment::default(),
        )
        .unwrap_err()
        .to_string();
        assert_eq!(
            error,
            "available locales \"fr-CA\" and \"fr_ca\" have the same runtime identifier"
        );
    }

    #[test]
    fn i18n_reports_a_missing_locales_directory() {
        let directory = tempfile::tempdir().unwrap();
        let error = expand_i18n(quote! {}, directory.path(), &Environment::default())
            .unwrap_err()
            .to_string();
        assert!(
            error.starts_with("locales directory not found"),
            "{}",
            error
        );
    }

    #[cfg(feature = "backend-json")]
    #[test]
    fn expand_reads_the_forwarded_configuration() {
        let directory = crate_with_locales("");
        let config = Config::from_i18n_args(
            quote! { available_locales = ["en", "fr"] },
            directory.path(),
            &Environment::default(),
        )
        .unwrap();
        let config_tokens = config.to_tokens();
        let tokens =
            expand_request(quote! { #config_tokens @t "greeting", locale = "fr" }).unwrap();
        assert_eq!(tokens.to_string(), quote! { "Bonjour" }.to_string());
        let tokens = expand_request(quote! { #config_tokens @locale }).unwrap();
        assert_eq!(
            tokens.to_string(),
            format!("(& [\"en\" , \"fr\"]) [{}]", CURRENT_INDEX)
        );
        let tokens = expand_request(quote! { #config_tokens @available }).unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! { { let locales: &'static [&'static str] = &["en", "fr"]; locales } }
                .to_string()
        );
        assert!(expand_request(quote! { #config_tokens @other }).is_err());
        assert!(expand_request(quote! { @t "greeting" }).is_err());
    }

    #[test]
    fn locale_and_available_in_single_mode() {
        let config = config(&["fr"]);
        assert_eq!(expand_locale(&config).to_string(), "\"fr\"");
        assert_eq!(
            expand_available(&config).to_string(),
            quote! { { let locales: &'static [&'static str] = &["fr"]; locales } }.to_string()
        );
    }

    #[test]
    fn single_mode_evaluates_and_ignores_a_locale_expression() {
        let fixture = single(catalog(&[("greeting", "Bonjour {name}")]));
        let tokens = fixture
            .expand(quote! { "greeting", locale = user.locale })
            .unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! {
                {
                    let _: ::embedded_i18n::Locale = ::core::convert::Into::into(user.locale);
                    "Bonjour {name}"
                }
            }
            .to_string()
        );
        let tokens = fixture
            .expand(quote! { "greeting", name = user.name, locale = user.locale })
            .unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! {
                match (
                    &user.name,
                    { let _: ::embedded_i18n::Locale = ::core::convert::Into::into(user.locale); },
                ) {
                    (__a0, _,) => ::embedded_i18n::__FormatWith(
                        move |f: &mut ::core::fmt::Formatter<'_>| -> ::core::fmt::Result {
                            f.write_fmt(::core::format_args!("Bonjour {name}", name = __a0))
                        }
                    ),
                }
            }
            .to_string()
        );
    }

    #[test]
    fn multi_mode_resolves_a_locale_expression_at_runtime() {
        let tokens = english_and_french()
            .expand(quote! { "greeting", locale = user.locale })
            .unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! {
                (&["Hello {name}", "Bonjour {name}"])[crate::__embedded_i18n_resolve(
                    ::embedded_i18n::Locale::from(user.locale).id()
                )]
            }
            .to_string()
        );
    }

    #[test]
    fn multi_mode_matches_on_the_index_inside_the_closure() {
        let tokens = english_and_french()
            .expand(quote! { "greeting", name = "Ada" })
            .unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! {
                match (&"Ada", crate::__embedded_i18n_resolve(::embedded_i18n::__current_locale()),) {
                    (__a0, __index,) => ::embedded_i18n::__FormatWith(
                        move |f: &mut ::core::fmt::Formatter<'_>| -> ::core::fmt::Result {
                            match __index {
                                0 => f.write_fmt(::core::format_args!("Hello {name}", name = __a0)),
                                _ => f.write_fmt(::core::format_args!("Bonjour {name}", name = __a0)),
                            }
                        }
                    ),
                }
            }
            .to_string()
        );
    }

    #[test]
    fn multi_mode_reports_every_locale() {
        let fixture = Fixture {
            config: config(&["en", "fr"]),
            catalogs: vec![
                locale_catalog("en", messages(&[])),
                locale_catalog("fr", messages(&[("greeting", "Salut {nom}")])),
            ],
        };
        let error = fixture
            .expand(quote! { "greeting", name = 1 })
            .err()
            .unwrap();
        let messages: Vec<String> = error.into_iter().map(|error| error.to_string()).collect();
        assert_eq!(
            messages,
            [
                "translation for \"greeting\" not found in locales \"en\" (locales)",
                "translation of \"greeting\" in locale \"fr\" uses the placeholder {nom}, which is not an argument of t!",
            ]
        );
    }

    #[test]
    fn applies_argument_specs_and_percent_placeholders() {
        let fixture = single(catalog(&[
            ("serial", "SN {sn} {sn:x}"),
            ("hello", "Hello, %{name} {raw}"),
        ]));
        let tokens = fixture
            .expand(quote! { "serial", sn = 123 : {:08} })
            .unwrap()
            .to_string();
        assert!(
            tokens.contains(
                &quote! { ::core::format_args!("SN {sn:08} {sn:x}", sn = __a0) }.to_string()
            ),
            "{}",
            tokens
        );
        let tokens = fixture
            .expand(quote! { "hello", "name" => user })
            .unwrap()
            .to_string();
        assert!(
            tokens.contains(
                &quote! { ::core::format_args!("Hello, {name} {{raw}}", name = __a0) }.to_string()
            ),
            "{}",
            tokens
        );
    }

    #[test]
    fn without_arguments_placeholders_stay_as_written() {
        let tokens = single(catalog(&[("hello", "Hello, %{name} {{x}}")]))
            .expand(quote! { "hello" })
            .unwrap();
        let value: LitStr = syn::parse2(tokens).unwrap();
        assert_eq!(value.value(), "Hello, %{name} {{x}}");
    }

    #[test]
    fn reports_invalid_templates() {
        assert_eq!(
            error(
                quote! { "with_arg", value = 1 },
                &catalog(&[("with_arg", "Valeur : {}")])
            ),
            "translation of \"with_arg\" in locale \"fr\": positional placeholder {} in \"Valeur : {}\", use a named placeholder"
        );
    }

    #[test]
    fn count_without_plural_forms_is_an_argument() {
        let tokens = expand(
            quote! { "unread", count = n },
            &catalog(&[("unread", "{count} non lus")]),
        )
        .unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! {
                match (&n,) {
                    (__count,) => ::embedded_i18n::__FormatWith(
                        move |f: &mut ::core::fmt::Formatter<'_>| -> ::core::fmt::Result {
                            f.write_fmt(::core::format_args!("{count} non lus", count = __count))
                        }
                    ),
                }
            }
            .to_string()
        );
    }

    #[test]
    fn select_with_arguments_binds_the_value() {
        let catalog = locale_catalog(
            "en",
            messages(&[
                ("mail\u{4}greeting.male", "Dear {name}"),
                ("mail.greeting.other", "Hi {name}"),
            ]),
        );
        let tokens = Fixture {
            config: config(&["en"]),
            catalogs: vec![catalog],
        }
        .expand(quote! { context = "mail", "greeting", select = gender, name = "Ada" })
        .unwrap();
        assert_eq!(
            tokens.to_string(),
            quote! {
                match (&"Ada", &gender,) {
                    (__a0, __select,) => ::embedded_i18n::__FormatWith(
                        move |f: &mut ::core::fmt::Formatter<'_>| -> ::core::fmt::Result {
                            {
                                let _ = &__select;
                                f.write_fmt(::core::format_args!("Hi {name}", name = __a0))
                            }
                        }
                    ),
                }
            }
            .to_string()
        );
    }

    #[test]
    fn select_takes_precedence_over_plural_forms() {
        let catalog = locale_catalog(
            "en",
            messages(&[
                ("replied.male", "He replied {count}"),
                ("replied.other", "They replied {count}"),
            ]),
        );
        let tokens = Fixture {
            config: config(&["en"]),
            catalogs: vec![catalog],
        }
        .expand(quote! { "replied", select = gender, count = 2 })
        .unwrap()
        .to_string();
        assert!(!tokens.contains("unsigned_abs"), "{}", tokens);
        assert!(tokens.contains("\"male\""), "{}", tokens);
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

    #[test]
    fn plural_uses_cldr_categories() {
        let catalog = locale_catalog(
            "fr",
            messages(&[("files.one", "fichier"), ("files.other", "fichiers")]),
        );
        let tokens = expand(quote! { "files", count = total }, &catalog)
            .unwrap()
            .to_string();
        assert!(tokens.contains("match (& total ,)"), "{}", tokens);
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
            error(quote! { "files", count = 2 }, &catalog),
            "plural translation for \"files\" in locale \"ru\" is missing the few, many forms"
        );
    }

    #[test]
    fn plural_requires_known_language() {
        let catalog = locale_catalog("tlh", messages(&[("files.other", "files")]));
        assert!(
            error(quote! { "files", count = 2 }, &catalog)
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
        let tokens = expand(
            quote! { "files", count = 3 },
            &locale_catalog("xx", translations),
        )
        .unwrap()
        .to_string();
        let arm = |text: &str| quote! { f.write_fmt(::core::format_args!(#text)) }.to_string();
        assert!(
            tokens.contains(&format!("0u64 => {}", arm("a"))),
            "{}",
            tokens
        );
        assert!(tokens.contains(&format!("2u64 => {}", arm("c"))));
        assert!(tokens.contains(&format!("_ => {}", arm("a"))));
    }

    #[test]
    fn plural_without_rule_uses_germanic_default() {
        let mut translations = messages(&[]);
        translations
            .plurals
            .insert("files".to_string(), vec!["a".to_string(), "b".to_string()]);
        let tokens = expand(
            quote! { "files", count = 3 },
            &locale_catalog("xx", translations),
        )
        .unwrap()
        .to_string();
        assert!(tokens.contains("!= (1u64)"));
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
        let tokens = expand(quote! { "files", count = 0 }, &catalog)
            .unwrap()
            .to_string();
        assert!(
            tokens.contains(&format!(
                "== 0 {{ {} }}",
                quote! { f.write_fmt(::core::format_args!("none")) }
            )),
            "{}",
            tokens
        );
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
        let tokens = expand(quote! { "greeting", select = gender }, &catalog).unwrap();
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
            error(quote! { "greeting", select = gender }, &catalog),
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
        let tokens = expand(quote! { "greeting", select = gender }, &catalog).unwrap();
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
    fn multi_mode_select_matches_on_the_index() {
        let fixture = Fixture {
            config: config(&["en", "fr"]),
            catalogs: vec![
                locale_catalog("en", messages(&[("replied.other", "They replied")])),
                locale_catalog("fr", messages(&[("replied.other", "Iel a répondu")])),
            ],
        };
        let tokens = fixture
            .expand(quote! { "replied", select = gender })
            .unwrap()
            .to_string();
        assert!(
            tokens.starts_with(&format!("match {} {{ 0 =>", CURRENT_INDEX)),
            "{}",
            tokens
        );
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
        assert!(expand(quote! { "files", count = 2 }, &catalog).is_ok());
        assert!(expand(quote! { "replied", select = gender }, &catalog).is_ok());
    }

    #[cfg(feature = "backend-json")]
    #[test]
    fn strict_rejects_keys_only_in_fallbacks() {
        let catalog = strict_catalog();
        assert_eq!(
            error(quote! { "train" }, &catalog),
            "translation for \"train\" is missing from the primary locale \"fr-CA\", \"fr\" (strict mode)"
        );
        assert!(
            error(quote! { "boats", count = 2 }, &catalog)
                .starts_with("plural translation for \"boats\" is missing from the primary locale")
        );
        assert!(
            error(quote! { "answered", select = gender }, &catalog)
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
        assert!(expand(quote! { "open", name = 1 }, &catalog).is_ok());
        assert!(expand(quote! { "files", count = 2 }, &catalog).is_ok());
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

    #[cfg(feature = "backend-json")]
    #[test]
    fn catalogs_are_cached_per_configuration() {
        let directory = crate_with_locales("");
        let config = Config::from_i18n_args(
            quote! { available_locales = ["en", "fr"] },
            directory.path(),
            &Environment::default(),
        )
        .unwrap();
        let first = catalogs(&config);
        assert_eq!(first.len(), 2);
        assert!(Arc::ptr_eq(&first, &catalogs(&config)));
    }
}
