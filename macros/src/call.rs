//! `t!` call parsing.
//!
//! A call is a comma separated list of items: one message key (a string or
//! C string literal), the reserved options `context`, `locale`, `count` and
//! `select` (before or after the key), and named arguments (after the key),
//! written `name = value` or `"name" => value`, each optionally followed by a
//! format specification `: { :08 }`.

use proc_macro2::{Span, TokenStream, TokenTree};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Expr, ExprLit, Lit, LitCStr, LitStr, Token};

use crate::CONTEXT_SEPARATOR;

const RESERVED: [&str; 4] = ["context", "locale", "count", "select"];

/// The message key of a call.
pub enum Key {
    /// A string literal.
    Str(LitStr),
    /// A C string literal.
    CStr(LitCStr),
}

impl Key {
    /// The span of the key literal.
    pub fn span(&self) -> Span {
        match self {
            Key::Str(literal) => literal.span(),
            Key::CStr(literal) => literal.span(),
        }
    }

    /// The key as a string.
    pub fn value(&self) -> syn::Result<String> {
        match self {
            Key::Str(literal) => Ok(literal.value()),
            Key::CStr(literal) => literal
                .value()
                .into_string()
                .map_err(|_| syn::Error::new(literal.span(), "C string key must be valid UTF-8")),
        }
    }
}

/// The value of the `locale` option.
pub enum LocaleArgument {
    /// A string literal, resolved at compile time.
    Literal(LitStr),
    /// Any other expression, resolved at run time.
    Expr(Expr),
}

/// A named argument of a call.
pub struct Argument {
    /// The name, matched against the placeholders of the message.
    pub name: String,
    /// The span of the name.
    pub span: Span,
    /// The value expression.
    pub value: Expr,
    /// The format specification without its leading `:` (`08`, `>10.2`, `?`).
    pub spec: Option<String>,
}

/// A parsed `t!` call.
pub struct Call {
    /// The message key.
    pub key: Key,
    /// The `context` option.
    pub context: Option<LitStr>,
    /// The `locale` option.
    pub locale: Option<LocaleArgument>,
    /// The `count` option.
    pub count: Option<Expr>,
    /// The `select` option.
    pub select: Option<Expr>,
    /// The named arguments, in call order.
    pub arguments: Vec<Argument>,
}

impl Call {
    /// The lookup keys of the message `name`, most specific first.
    pub fn keys(&self, name: &str) -> Vec<String> {
        match &self.context {
            Some(context) => vec![
                format!("{}{}{}", context.value(), CONTEXT_SEPARATOR, name),
                format!("{}.{}", context.value(), name),
            ],
            None => vec![name.to_string()],
        }
    }

    /// A human readable description of the message `name`, for diagnostics.
    pub fn describe(&self, name: &str) -> String {
        match &self.context {
            Some(context) => format!("{:?} in context {:?}", name, context.value()),
            None => format!("{:?}", name),
        }
    }

    /// Whether the call expands to a plain string (no argument, no count).
    pub fn returns_str(&self) -> bool {
        self.arguments.is_empty() && self.count.is_none()
    }
}

fn given_twice(name: &str, span: Span) -> syn::Error {
    syn::Error::new(span, format!("`{}` is given twice", name))
}

fn no_key(span: Span) -> syn::Error {
    syn::Error::new(span, "expected a message key (string or C string literal)")
}

fn positional(span: Span) -> syn::Error {
    syn::Error::new(
        span,
        "positional arguments are not supported, name the argument: `name = value`",
    )
}

fn set<T>(slot: &mut Option<T>, name: &str, span: Span, value: T) -> syn::Result<()> {
    if slot.is_some() {
        return Err(given_twice(name, span));
    }
    *slot = Some(value);
    Ok(())
}

/// Parses the optional `: { :spec }` after an argument value.
fn parse_spec(input: ParseStream) -> syn::Result<Option<String>> {
    if !input.peek(Token![:]) {
        return Ok(None);
    }
    input.parse::<Token![:]>()?;
    let content;
    let brace = syn::braced!(content in input);
    let tokens: TokenStream = content.parse()?;
    let mut tokens = tokens.into_iter();
    match tokens.next() {
        Some(TokenTree::Punct(colon)) if colon.as_char() == ':' => {}
        _ => {
            return Err(syn::Error::new(
                brace.span.join(),
                "a format specification starts with `:`, as in `{:08}`",
            ));
        }
    }
    let spec: String = tokens.map(|token| token.to_string()).collect();
    Ok(if spec.is_empty() { None } else { Some(spec) })
}

/// Applies a reserved option, parsing its value.
fn parse_reserved(
    call: &mut PartialCall,
    input: ParseStream,
    name: &str,
    span: Span,
) -> syn::Result<()> {
    let value: Expr = input.parse()?;
    match name {
        "context" => match value {
            Expr::Lit(ExprLit {
                lit: Lit::Str(literal),
                ..
            }) => set(&mut call.context, name, span, literal),
            other => Err(syn::Error::new(
                other.span(),
                "`context` expects a string literal",
            )),
        },
        "locale" => {
            let locale = match value {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(literal),
                    ..
                }) => LocaleArgument::Literal(literal),
                other => LocaleArgument::Expr(other),
            };
            set(&mut call.locale, name, span, locale)
        }
        "count" => set(&mut call.count, name, span, value),
        "select" => set(&mut call.select, name, span, value),
        _ => unreachable!("not a reserved option: {name}"),
    }
}

/// Applies a named argument, or the reserved option of the same name.
fn parse_named(
    call: &mut PartialCall,
    input: ParseStream,
    name: String,
    span: Span,
) -> syn::Result<()> {
    if RESERVED.contains(&name.as_str()) {
        return parse_reserved(call, input, &name, span);
    }
    if call.key.is_none() {
        return Err(syn::Error::new(
            span,
            "arguments must follow the message key",
        ));
    }
    if call.arguments.iter().any(|argument| argument.name == name) {
        return Err(given_twice(&name, span));
    }
    let value: Expr = input.parse()?;
    let spec = parse_spec(input)?;
    call.arguments.push(Argument {
        name,
        span,
        value,
        spec,
    });
    Ok(())
}

#[derive(Default)]
struct PartialCall {
    key: Option<Key>,
    context: Option<LitStr>,
    locale: Option<LocaleArgument>,
    count: Option<Expr>,
    select: Option<Expr>,
    arguments: Vec<Argument>,
}

fn parse_item(call: &mut PartialCall, input: ParseStream) -> syn::Result<()> {
    if input.peek(Lit) {
        let literal: Lit = input.parse()?;
        return match literal {
            Lit::Str(literal) if input.peek(Token![=>]) => {
                input.parse::<Token![=>]>()?;
                parse_named(call, input, literal.value(), literal.span())
            }
            Lit::Str(literal) if call.key.is_none() => {
                call.key = Some(Key::Str(literal));
                Ok(())
            }
            Lit::CStr(literal) if call.key.is_none() => {
                call.key = Some(Key::CStr(literal));
                Ok(())
            }
            other if call.key.is_none() && !matches!(other, Lit::Str(_) | Lit::CStr(_)) => {
                Err(no_key(other.span()))
            }
            other => Err(positional(other.span())),
        };
    }

    if input.peek(syn::Ident::peek_any) && input.peek2(Token![=]) && !input.peek2(Token![==]) {
        let ident = syn::Ident::parse_any(input)?;
        input.parse::<Token![=]>()?;
        return parse_named(call, input, ident.unraw().to_string(), ident.span());
    }

    if call.key.is_none() {
        Err(no_key(input.span()))
    } else {
        Err(positional(input.span()))
    }
}

impl Parse for Call {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut call = PartialCall::default();
        while !input.is_empty() {
            parse_item(&mut call, input)?;
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        let key = call.key.ok_or_else(|| no_key(input.span()))?;
        if matches!(key, Key::CStr(_)) && (!call.arguments.is_empty() || call.count.is_some()) {
            return Err(syn::Error::new(
                key.span(),
                "C strings cannot take arguments",
            ));
        }
        Ok(Call {
            key,
            context: call.context,
            locale: call.locale,
            count: call.count,
            select: call.select,
            arguments: call.arguments,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn parse(tokens: TokenStream) -> Call {
        syn::parse2(tokens).unwrap_or_else(|error| panic!("unexpected error: {error}"))
    }

    fn error(tokens: TokenStream) -> String {
        match syn::parse2::<Call>(tokens) {
            Ok(_) => panic!("expected an error"),
            Err(error) => error.to_string(),
        }
    }

    fn text(expr: &Expr) -> String {
        quote!(#expr).to_string()
    }

    #[test]
    fn parses_a_bare_key() {
        let call = parse(quote!("hello"));
        assert!(matches!(&call.key, Key::Str(key) if key.value() == "hello"));
        assert_eq!(call.key.value().unwrap(), "hello");
        assert!(call.context.is_none());
        assert!(call.locale.is_none());
        assert!(call.count.is_none());
        assert!(call.select.is_none());
        assert!(call.arguments.is_empty());
        assert!(call.returns_str());

        let call = parse(quote!(c"hello"));
        assert!(matches!(call.key, Key::CStr(_)));
        assert_eq!(call.key.value().unwrap(), "hello");
        assert!(call.returns_str());
    }

    #[test]
    fn parses_named_and_arrow_arguments() {
        let call = parse(quote!("m", name = user.name, "count" => 3 + 2, "other key" => x,));
        assert_eq!(call.arguments.len(), 2);
        assert_eq!(call.arguments[0].name, "name");
        assert_eq!(text(&call.arguments[0].value), "user . name");
        assert!(call.arguments[0].spec.is_none());
        assert_eq!(call.arguments[1].name, "other key");
        assert_eq!(text(&call.arguments[1].value), "x");
        // The arrow "count" is the reserved option, not an argument.
        assert_eq!(text(call.count.as_ref().unwrap()), "3 + 2");
        assert!(!call.returns_str());

        let call = parse(quote!("m", "select" => g, "locale" => "fr", "context" => "menu"));
        assert!(call.arguments.is_empty());
        assert_eq!(text(call.select.as_ref().unwrap()), "g");
        assert!(matches!(call.locale, Some(LocaleArgument::Literal(_))));
        assert_eq!(call.context.unwrap().value(), "menu");
    }

    #[test]
    fn parses_format_specifications() {
        let call = parse(quote!(
            "s",
            sn = 123 : {:08},
            x = y : {:>10.2},
            d = v : {:?},
            "w" => z : {:#010x},
            plain = 1
        ));
        let specs: Vec<_> = call
            .arguments
            .iter()
            .map(|argument| argument.spec.as_deref())
            .collect();
        assert_eq!(
            specs,
            [Some("08"), Some(">10.2"), Some("?"), Some("#010x"), None]
        );
        assert_eq!(text(&call.arguments[0].value), "123");
        assert_eq!(text(&call.arguments[1].value), "y");
    }

    #[test]
    fn options_before_or_after_the_key() {
        let before = parse(quote!(context = "menu", "Open"));
        let after = parse(quote!("Open", context = "menu"));
        for call in [before, after] {
            assert_eq!(call.key.value().unwrap(), "Open");
            assert_eq!(call.context.unwrap().value(), "menu");
        }

        let mixed = parse(quote!(
            count = n,
            "Open",
            locale = "fr",
            name = 1,
            select = g
        ));
        assert_eq!(mixed.key.value().unwrap(), "Open");
        assert_eq!(mixed.arguments.len(), 1);
        assert!(mixed.count.is_some());
        assert!(mixed.select.is_some());
        assert!(mixed.locale.is_some());
    }

    #[test]
    fn locale_literal_or_expression() {
        let call = parse(quote!("k", locale = "fr"));
        assert!(matches!(call.locale, Some(LocaleArgument::Literal(l)) if l.value() == "fr"));
        let call = parse(quote!("k", locale = user.locale));
        match call.locale {
            Some(LocaleArgument::Expr(expr)) => assert_eq!(text(&expr), "user . locale"),
            _ => panic!("expected an expression"),
        }
    }

    #[test]
    fn count_and_select_are_reserved() {
        let call = parse(quote!("k", count = n, select = g));
        assert!(call.arguments.is_empty());
        assert_eq!(text(call.count.as_ref().unwrap()), "n");
        assert_eq!(text(call.select.as_ref().unwrap()), "g");
        assert!(!call.returns_str());

        let call = parse(quote!("k", select = g));
        assert!(call.arguments.is_empty());
        assert!(call.returns_str());
    }

    #[test]
    fn keys_and_description_follow_the_context() {
        let call = parse(quote!("Open"));
        assert_eq!(call.keys("Open"), ["Open"]);
        assert_eq!(call.describe("Open"), "\"Open\"");

        let call = parse(quote!(context = "menu", "Open"));
        assert_eq!(call.keys("Open"), ["menu\u{4}Open", "menu.Open"]);
        assert_eq!(call.describe("Open"), "\"Open\" in context \"menu\"");
    }

    #[test]
    fn rejects_invalid_calls() {
        let no_key = "expected a message key (string or C string literal)";
        assert_eq!(error(quote!()), no_key);
        assert_eq!(error(quote!(context = "menu")), no_key);
        assert_eq!(error(quote!(5)), no_key);

        assert_eq!(
            error(quote!("k", count = 1, count = 2)),
            "`count` is given twice"
        );
        assert_eq!(
            error(quote!("k", "count" => 1, count = 2)),
            "`count` is given twice"
        );
        assert_eq!(
            error(quote!(context = "a", "k", context = "b")),
            "`context` is given twice"
        );
        assert_eq!(error(quote!("k", a = 1, a = 2)), "`a` is given twice");

        let before = "arguments must follow the message key";
        assert_eq!(error(quote!(name = 1, "k")), before);
        assert_eq!(error(quote!("name" => 1, "k")), before);

        let positional =
            "positional arguments are not supported, name the argument: `name = value`";
        assert_eq!(error(quote!("k", user.name)), positional);
        assert_eq!(error(quote!("k", "other")), positional);
        assert_eq!(error(quote!("k", 5)), positional);

        let context = "`context` expects a string literal";
        assert_eq!(error(quote!("k", context = menu)), context);
        assert_eq!(error(quote!("k", context = c"menu")), context);

        let cstr = "C strings cannot take arguments";
        assert_eq!(error(quote!(c"k", a = 1)), cstr);
        assert_eq!(error(quote!(c"k", count = 1)), cstr);
    }

    #[test]
    fn rejects_malformed_specs_and_separators() {
        assert!(syn::parse2::<Call>(quote!("k", a = 1 : {08})).is_err());
        assert!(syn::parse2::<Call>(quote!("k" a = 1)).is_err());
        assert!(syn::parse2::<Call>(quote!("k",, a = 1)).is_err());
    }
}
