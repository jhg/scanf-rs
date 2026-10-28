//! Format string tokenization at compile time.

use crate::types::{FormatToken, Placeholder};
use crate::validation::is_valid_identifier;
use std::fmt::Display;
use std::iter::Peekable;
use std::str::Chars;
use syn::LitStr;

/// Longest format string accepted, in bytes.
const MAX_FORMAT_STRING_LEN: usize = 10_000;
/// Most tokens (text segments plus placeholders) a format string may have.
const MAX_TOKENS: usize = 256;
/// Longest placeholder name accepted, in characters.
const MAX_IDENTIFIER_LEN: usize = 128;

/// Compile error reported on the format string literal.
pub fn format_error(format_lit: &LitStr, message: impl Display) -> syn::Error {
    syn::Error::new(format_lit.span(), message)
}

fn push_token(
    tokens: &mut Vec<FormatToken>,
    token: FormatToken,
    format_lit: &LitStr,
) -> syn::Result<()> {
    if tokens.len() >= MAX_TOKENS {
        return Err(format_error(
            format_lit,
            format!("Too many tokens (more than the maximum of {MAX_TOKENS})."),
        ));
    }
    tokens.push(token);
    Ok(())
}

/// Push the pending literal text as a token, if there is any.
fn flush_text(
    tokens: &mut Vec<FormatToken>,
    text: &mut String,
    format_lit: &LitStr,
) -> syn::Result<()> {
    if text.is_empty() {
        return Ok(());
    }
    let text = std::mem::take(text);
    push_token(tokens, FormatToken::Text(text.into_boxed_str()), format_lit)
}

/// Read the placeholder body after `{` up to (and consuming) the closing `}`.
fn read_placeholder_body(chars: &mut Peekable<Chars>, format_lit: &LitStr) -> syn::Result<String> {
    let mut content = String::new();
    for (length, ch) in chars.by_ref().enumerate() {
        if ch == '}' {
            return Ok(content);
        }
        if length >= MAX_IDENTIFIER_LEN {
            return Err(format_error(
                format_lit,
                format!(
                    "Identifier in placeholder too long (over {MAX_IDENTIFIER_LEN} characters)."
                ),
            ));
        }
        content.push(ch);
    }
    Err(format_error(
        format_lit,
        "Unclosed placeholder, missing closing '}'. Use '{{' for a literal brace.",
    ))
}

/// `{}` is anonymous, `{name}` is named; anything else is an error.
fn classify_placeholder(content: String, format_lit: &LitStr) -> syn::Result<Placeholder> {
    if content.is_empty() {
        Ok(Placeholder::Anonymous)
    } else if is_valid_identifier(&content) {
        Ok(Placeholder::Named(content.into_boxed_str()))
    } else {
        Err(format_error(
            format_lit,
            format!(
                "Invalid identifier '{content}' in placeholder. \
                 Identifiers must start with a letter or underscore, \
                 contain only alphanumeric characters or underscores, \
                 and not be Rust keywords. Use '{{}}' for anonymous placeholders."
            ),
        ))
    }
}

/// Two placeholders with nothing between them cannot be split: where would the first end?
fn reject_consecutive_placeholders(tokens: &[FormatToken], format_lit: &LitStr) -> syn::Result<()> {
    let consecutive = tokens.windows(2).any(|pair| {
        matches!(
            pair,
            [FormatToken::Placeholder(_), FormatToken::Placeholder(_)]
        )
    });
    if consecutive {
        return Err(consecutive_placeholders_error(format_lit));
    }
    Ok(())
}

fn consecutive_placeholders_error(format_lit: &LitStr) -> syn::Error {
    format_error(
        format_lit,
        "Consecutive placeholders without separator are ambiguous and not supported. \
         Add text between placeholders to separate them. Example: '{}:{}' instead of '{}{}'",
    )
}

/// Split a format string into literal text and placeholders, resolving `{{` and `}}` escapes.
///
/// Rejects empty formats, strings over the size limits and consecutive placeholders.
pub fn tokenize_format_string(
    format_str: &str,
    format_lit: &LitStr,
) -> syn::Result<Vec<FormatToken>> {
    if format_str.is_empty() {
        return Err(format_error(
            format_lit,
            "Format string cannot be empty. Provide at least one placeholder or literal text.",
        ));
    }
    if format_str.len() > MAX_FORMAT_STRING_LEN {
        return Err(format_error(
            format_lit,
            format!(
                "Format string too long ({} bytes). Maximum allowed: {MAX_FORMAT_STRING_LEN} bytes.",
                format_str.len()
            ),
        ));
    }

    let mut tokens = Vec::new();
    let mut chars = format_str.chars().peekable();
    let mut current_text = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '{' if chars.next_if_eq(&'{').is_some() => current_text.push('{'),
            '{' => {
                flush_text(&mut tokens, &mut current_text, format_lit)?;
                let content = read_placeholder_body(&mut chars, format_lit)?;
                let placeholder = classify_placeholder(content, format_lit)?;
                push_token(
                    &mut tokens,
                    FormatToken::Placeholder(placeholder),
                    format_lit,
                )?;
            }
            '}' if chars.next_if_eq(&'}').is_some() => current_text.push('}'),
            '}' => {
                return Err(format_error(
                    format_lit,
                    "Unescaped '}' in format string. Use '}}' to escape it.",
                ));
            }
            other => current_text.push(other),
        }
    }

    flush_text(&mut tokens, &mut current_text, format_lit)?;
    reject_consecutive_placeholders(&tokens, format_lit)?;
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(format: &str) -> syn::Result<Vec<FormatToken>> {
        let format_lit: LitStr = syn::parse_quote!("unused");
        tokenize_format_string(format, &format_lit)
    }

    fn error_of(format: &str) -> String {
        tokenize(format)
            .expect_err("format should be rejected")
            .to_string()
    }

    fn text(content: &str) -> FormatToken {
        FormatToken::Text(content.into())
    }

    fn named(name: &str) -> FormatToken {
        FormatToken::Placeholder(Placeholder::Named(name.into()))
    }

    fn anonymous() -> FormatToken {
        FormatToken::Placeholder(Placeholder::Anonymous)
    }

    /// Compare ignoring `Debug` noise: `FormatToken` does not implement `PartialEq`.
    fn assert_tokens(format: &str, expected: &[FormatToken]) {
        let tokens = tokenize(format).unwrap();
        assert_eq!(
            format!("{tokens:?}"),
            format!("{expected:?}"),
            "format: {format:?}"
        );
    }

    #[test]
    fn splits_text_and_placeholders() {
        assert_tokens("{x} text {y}", &[named("x"), text(" text "), named("y")]);
    }

    #[test]
    fn distinguishes_anonymous_from_named() {
        assert_tokens("a{}b{c}", &[text("a"), anonymous(), text("b"), named("c")]);
    }

    #[test]
    fn escaped_braces_become_literal_text() {
        assert_tokens("{{text}}", &[text("{text}")]);
    }

    #[test]
    fn escapes_merge_with_adjacent_text() {
        assert_tokens("a{{b}}c", &[text("a{b}c")]);
    }

    #[test]
    fn escaped_brace_followed_by_placeholder() {
        assert_tokens("{{{}", &[text("{"), anonymous()]);
    }

    #[test]
    fn placeholder_followed_by_escaped_brace() {
        assert_tokens("{}}}", &[anonymous(), text("}")]);
    }

    #[test]
    fn escaped_open_brace_leaves_a_stray_closing_brace() {
        assert!(error_of("{{}").contains("Unescaped '}'"));
    }

    #[test]
    fn three_closing_braces_are_an_escape_plus_an_error() {
        assert!(error_of("}}}").contains("Unescaped '}'"));
    }

    #[test]
    fn unescaped_closing_brace_is_rejected() {
        assert!(error_of("a}b").contains("Unescaped '}'"));
        assert!(error_of("}").contains("Unescaped '}'"));
    }

    #[test]
    fn rejects_invalid_identifiers() {
        for format in ["{1a}", "{a-b}", "{let}", "{self}", "{ }", "{ x }"] {
            assert!(
                error_of(format).contains("Invalid identifier"),
                "format: {format:?}"
            );
        }
    }

    #[test]
    fn empty_format_is_rejected() {
        assert!(error_of("").contains("cannot be empty"));
    }

    #[test]
    fn accepts_a_format_of_exactly_the_maximum_length() {
        let format = "a".repeat(MAX_FORMAT_STRING_LEN);
        assert_tokens(&format, &[text(&format)]);
    }

    #[test]
    fn rejects_a_format_one_byte_over_the_maximum_length() {
        let format = "a".repeat(MAX_FORMAT_STRING_LEN + 1);
        assert!(error_of(&format).contains("Format string too long"));
    }

    #[test]
    fn format_length_limit_counts_bytes() {
        // 5_000 two-byte characters are exactly 10_000 bytes, one more goes over.
        let at_limit = "é".repeat(MAX_FORMAT_STRING_LEN / 2);
        assert!(tokenize(&at_limit).is_ok());
        let over_limit = format!("{at_limit}a");
        assert!(error_of(&over_limit).contains("Format string too long"));
    }

    #[test]
    fn accepts_an_identifier_of_exactly_the_maximum_length() {
        let name = "a".repeat(MAX_IDENTIFIER_LEN);
        assert_tokens(&format!("{{{name}}}"), &[named(&name)]);
    }

    #[test]
    fn rejects_an_identifier_one_character_over_the_maximum_length() {
        let name = "a".repeat(MAX_IDENTIFIER_LEN + 1);
        assert!(error_of(&format!("{{{name}}}")).contains("too long"));
    }

    #[test]
    fn identifier_length_limit_counts_characters_not_bytes() {
        let at_limit = "é".repeat(MAX_IDENTIFIER_LEN);
        assert_tokens(&format!("{{{at_limit}}}"), &[named(&at_limit)]);
        let over_limit = "é".repeat(MAX_IDENTIFIER_LEN + 1);
        assert!(error_of(&format!("{{{over_limit}}}")).contains("too long"));
    }

    #[test]
    fn a_hundred_multibyte_characters_are_a_valid_identifier() {
        let name = "é".repeat(100);
        assert_tokens(&format!("{{{name}}}"), &[named(&name)]);
    }

    #[test]
    fn accepts_exactly_the_maximum_number_of_tokens() {
        // Placeholders and text segments both count toward the limit.
        let format = "{} ".repeat(MAX_TOKENS / 2);
        let tokens = tokenize(&format).unwrap();
        assert_eq!(tokens.len(), MAX_TOKENS);
        let placeholders = tokens
            .iter()
            .filter(|token| matches!(token, FormatToken::Placeholder(_)))
            .count();
        assert_eq!(placeholders, MAX_TOKENS / 2);
    }

    #[test]
    fn rejects_one_token_over_the_limit_ending_in_a_placeholder() {
        let format = format!("{}{{}}", "{} ".repeat(MAX_TOKENS / 2));
        assert!(error_of(&format).contains("Too many tokens"));
    }

    #[test]
    fn rejects_one_token_over_the_limit_ending_in_a_named_placeholder() {
        let format = format!("{}{{name}}", "{} ".repeat(MAX_TOKENS / 2));
        assert!(error_of(&format).contains("Too many tokens"));
    }

    #[test]
    fn consecutive_placeholders_cannot_bypass_the_token_limit() {
        for placeholder in ["{}", "{name}"] {
            let format = placeholder.repeat(MAX_TOKENS + 1);
            // Enforce the limit while tokenizing, before checking for adjacent placeholders.
            assert!(error_of(&format).contains("Too many tokens"));
        }
    }

    #[test]
    fn rejects_one_token_over_the_limit_in_intermediate_text() {
        let format = format!("{}b{{}}", "a{}".repeat(MAX_TOKENS / 2));
        assert!(error_of(&format).contains("Too many tokens"));
    }

    #[test]
    fn rejects_one_token_over_the_limit_ending_in_text() {
        let format = format!("{}b", "a{}".repeat(MAX_TOKENS / 2));
        assert!(error_of(&format).contains("Too many tokens"));
    }

    #[test]
    fn unterminated_placeholders_are_rejected() {
        for format in ["{", "{x", "a {name", "x={x", "{abc"] {
            assert!(error_of(format).contains("Unclosed"), "format: {format:?}");
        }
    }

    #[test]
    fn overlong_unterminated_placeholder_is_rejected() {
        let format = format!("{{{}", "a".repeat(MAX_IDENTIFIER_LEN + 1));
        assert!(tokenize(&format).is_err());
    }
}
