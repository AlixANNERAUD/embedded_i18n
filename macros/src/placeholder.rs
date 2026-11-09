//! Placeholder templates of translations.

/// A translation converted to a Rust format string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Template {
    /// Rust format string with named placeholders only.
    pub format: String,
    /// Distinct placeholder names, in order of first occurrence.
    pub placeholders: Vec<String>,
}

/// Converts the text of a translation to a [`Template`].
///
/// Two syntaxes are accepted. Text containing `%{` uses the rust-i18n syntax:
/// `%{name}` is a placeholder and every other brace is literal. Any other text
/// uses the Rust format syntax: `{name}` or `{name:spec}` is a placeholder and
/// `{{` / `}}` are escapes.
pub fn template(text: &str) -> Result<Template, String> {
    let percent = text.contains("%{");
    let mut format = String::with_capacity(text.len());
    let mut placeholders: Vec<String> = Vec::new();
    let mut chars = text.char_indices().peekable();

    while let Some((index, c)) = chars.next() {
        match c {
            '%' if percent && matches!(chars.peek(), Some((_, '{'))) => {
                let start = index + 2;
                let length = text[start..]
                    .find('}')
                    .ok_or_else(|| format!("unbalanced brace in {text:?}"))?;
                let name = &text[start..start + length];
                if !is_identifier(name) {
                    return Err(format!("invalid placeholder %{{{name}}} in {text:?}"));
                }
                record(&mut placeholders, name);
                format.push('{');
                format.push_str(name);
                format.push('}');
                // Skip the `{`, the name and the closing `}`.
                for _ in 0..name.chars().count() + 2 {
                    chars.next();
                }
            }
            '{' if percent => format.push_str("{{"),
            '}' if percent => format.push_str("}}"),
            '{' if matches!(chars.peek(), Some((_, '{'))) => {
                chars.next();
                format.push_str("{{");
            }
            '}' if matches!(chars.peek(), Some((_, '}'))) => {
                chars.next();
                format.push_str("}}");
            }
            '{' => {
                let start = index + 1;
                let length = text[start..]
                    .find('}')
                    .ok_or_else(|| format!("unbalanced brace in {text:?}"))?;
                let inner = &text[start..start + length];
                let name = inner.split_once(':').map_or(inner, |(name, _)| name);
                if name.is_empty() || name.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(format!(
                        "positional placeholder {{{inner}}} in {text:?}, use a named placeholder"
                    ));
                }
                if !is_identifier(name) || inner.contains('{') {
                    return Err(format!("invalid placeholder {{{inner}}} in {text:?}"));
                }
                record(&mut placeholders, name);
                format.push('{');
                format.push_str(inner);
                format.push('}');
                // Skip the inner text and the closing `}`.
                for _ in 0..inner.chars().count() + 1 {
                    chars.next();
                }
            }
            '}' => return Err(format!("unbalanced brace in {text:?}")),
            other => format.push(other),
        }
    }

    Ok(Template {
        format,
        placeholders,
    })
}

/// Adds `:spec` to the placeholders of that name that have none.
pub fn apply_specs(template: &Template, specs: &[(&str, &str)]) -> String {
    let format = template.format.as_str();
    let mut out = String::with_capacity(format.len());
    let mut chars = format.char_indices().peekable();

    while let Some((index, c)) = chars.next() {
        match c {
            '{' | '}' if matches!(chars.peek(), Some((_, next)) if *next == c) => {
                chars.next();
                out.push(c);
                out.push(c);
            }
            '{' => {
                // `template` guarantees a closing brace.
                let start = index + 1;
                let length = format[start..].find('}').unwrap_or(format.len() - start);
                let inner = &format[start..start + length];
                out.push('{');
                out.push_str(inner);
                if !inner.contains(':') {
                    if let Some((_, spec)) = specs.iter().find(|(name, _)| *name == inner) {
                        out.push(':');
                        out.push_str(spec);
                    }
                }
                // Keep the closing `}` (if any) for the next iteration of the loop.
                for _ in 0..inner.chars().count() {
                    chars.next();
                }
            }
            other => out.push(other),
        }
    }
    out
}

/// Checks that every placeholder of the template is an argument of `t!`.
pub fn check(
    template: &Template,
    arguments: &[&str],
    key: &str,
    locale: &str,
) -> Result<(), String> {
    match template
        .placeholders
        .iter()
        .find(|name| !arguments.contains(&name.as_str()))
    {
        Some(name) => Err(format!(
            "translation of {key:?} in locale {locale:?} uses the placeholder {{{name}}}, which is not an argument of t!"
        )),
        None => Ok(()),
    }
}

fn record(placeholders: &mut Vec<String>, name: &str) {
    if !placeholders.iter().any(|known| known == name) {
        placeholders.push(name.to_owned());
    }
}

fn is_identifier(name: &str) -> bool {
    if name == "_" {
        return false;
    }
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(b) if b.is_ascii_alphabetic() || b == b'_')
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(template: &Template) -> Vec<&str> {
        template.placeholders.iter().map(String::as_str).collect()
    }

    #[test]
    fn rust_syntax_placeholders() {
        let t = template("Hello {name}, {count} {{x}}").unwrap();
        assert_eq!(t.format, "Hello {name}, {count} {{x}}");
        assert_eq!(names(&t), ["name", "count"]);
    }

    #[test]
    fn keeps_inline_specs() {
        let t = template("{n:08}").unwrap();
        assert_eq!(t.format, "{n:08}");
        assert_eq!(names(&t), ["n"]);
    }

    #[test]
    fn percent_syntax_converts() {
        let t = template("Hello, %{name}").unwrap();
        assert_eq!(t.format, "Hello, {name}");
        assert_eq!(names(&t), ["name"]);
    }

    #[test]
    fn percent_syntax_escapes_plain_braces() {
        let t = template("%{name} {raw}").unwrap();
        assert_eq!(t.format, "{name} {{raw}}");
        assert_eq!(names(&t), ["name"]);
    }

    #[test]
    fn rejects_positional_and_unbalanced() {
        assert_eq!(
            template("{}").err().unwrap(),
            "positional placeholder {} in \"{}\", use a named placeholder"
        );
        assert_eq!(
            template("{0}").err().unwrap(),
            "positional placeholder {0} in \"{0}\", use a named placeholder"
        );
        assert_eq!(
            template("a {b").err().unwrap(),
            "unbalanced brace in \"a {b\""
        );
        assert_eq!(
            template("a b}").err().unwrap(),
            "unbalanced brace in \"a b}\""
        );
        assert_eq!(
            template("%{b").err().unwrap(),
            "unbalanced brace in \"%{b\""
        );
        assert_eq!(
            template("{a-b}").err().unwrap(),
            "invalid placeholder {a-b} in \"{a-b}\""
        );
        assert_eq!(
            template("{_}").err().unwrap(),
            "invalid placeholder {_} in \"{_}\""
        );
    }

    #[test]
    fn distinct_names_in_first_occurrence_order() {
        let t = template("{b} {a} {b:x}").unwrap();
        assert_eq!(names(&t), ["b", "a"]);
    }

    #[test]
    fn applies_argument_specs() {
        let t = template("{sn} {sn:x}").unwrap();
        assert_eq!(apply_specs(&t, &[("sn", "08")]), "{sn:08} {sn:x}");
    }

    #[test]
    fn apply_specs_ignores_escapes_and_other_names() {
        let t = template("{{sn}} {other} {sn}").unwrap();
        assert_eq!(apply_specs(&t, &[("sn", "08")]), "{{sn}} {other} {sn:08}");
    }

    #[test]
    fn checks_missing_arguments() {
        let t = template("{name}").unwrap();
        assert_eq!(
            check(&t, &["count"], "k", "fr").unwrap_err(),
            "translation of \"k\" in locale \"fr\" uses the placeholder {name}, which is not an argument of t!"
        );
        assert!(check(&t, &["name", "extra"], "k", "fr").is_ok());
    }
}
