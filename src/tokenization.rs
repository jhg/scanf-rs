//! Format string tokenization at compile-time.

use crate::constants::{
    IDENTIFIER_CAPACITY, MAX_FORMAT_STRING_LEN, MAX_IDENTIFIER_LEN, MAX_TOKENS,
    TEXT_SEGMENT_CAPACITY, TOKENS_INITIAL_CAPACITY,
};
use crate::types::{FormatToken, Placeholder};
use crate::validation::is_valid_identifier;
use std::iter::Peekable;
use std::str::Chars;
use syn::LitStr;

/// Push token with `MAX_TOKENS` validation.
#[inline]
fn push_token(
    tokens: &mut Vec<FormatToken>,
    token: FormatToken,
    format_lit: &LitStr,
) -> syn::Result<()> {
    if tokens.len() >= MAX_TOKENS {
        return Err(syn::Error::new(
            format_lit.span(),
            format!(
                "Too many tokens (would exceed {}). Maximum: {}. Prevents compile-time DoS.",
                tokens.len() + 1,
                MAX_TOKENS
            ),
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
    let text = std::mem::replace(text, String::with_capacity(TEXT_SEGMENT_CAPACITY));
    push_token(tokens, FormatToken::Text(text.into_boxed_str()), format_lit)
}

/// Read the placeholder body after `{` up to (and consuming) the closing `}`.
fn read_placeholder_body(chars: &mut Peekable<Chars>, format_lit: &LitStr) -> syn::Result<String> {
    let mut content = String::with_capacity(IDENTIFIER_CAPACITY);
    for ch in chars.by_ref() {
        if ch == '}' {
            break;
        }
        if content.len() >= MAX_IDENTIFIER_LEN {
            return Err(syn::Error::new(
                format_lit.span(),
                format!(
                    "Identifier in placeholder too long (>{MAX_IDENTIFIER_LEN} characters). \
                     This limit prevents compile-time DoS attacks."
                ),
            ));
        }
        content.push(ch);
    }
    Ok(content)
}

/// `{}` is anonymous, `{name}` is named; anything else is an error.
fn classify_placeholder(content: String, format_lit: &LitStr) -> syn::Result<Placeholder> {
    if content.is_empty() {
        Ok(Placeholder::Anonymous)
    } else if is_valid_identifier(&content) {
        Ok(Placeholder::Named(content.into_boxed_str()))
    } else {
        Err(syn::Error::new(
            format_lit.span(),
            format!(
                "Invalid identifier '{content}' in placeholder. \
                 Identifiers must start with a letter or underscore, \
                 contain only alphanumeric characters or underscores, \
                 and not be Rust keywords. Use '{{}}' for anonymous placeholders."
            ),
        ))
    }
}

/// Tokenize format string into text/placeholders. Handles `{{`/`}}` escapes.
///
/// Enforces `MAX_FORMAT_STRING_LEN`, `MAX_TOKENS`, `MAX_IDENTIFIER_LEN` limits.
pub fn tokenize_format_string(
    format_str: &str,
    format_lit: &LitStr,
) -> syn::Result<Vec<FormatToken>> {
    if format_str.len() > MAX_FORMAT_STRING_LEN {
        return Err(syn::Error::new(
            format_lit.span(),
            format!(
                "Format string too long ({} bytes). Maximum allowed: {} bytes. \
                 This limit prevents compile-time DoS attacks.",
                format_str.len(),
                MAX_FORMAT_STRING_LEN
            ),
        ));
    }

    let mut tokens: Vec<FormatToken> = Vec::with_capacity(TOKENS_INITIAL_CAPACITY);
    let mut chars = format_str.chars().peekable();
    let mut current_text = String::with_capacity(TEXT_SEGMENT_CAPACITY);

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
                return Err(syn::Error::new(
                    format_lit.span(),
                    "Unescaped '}' in format string. Use '}}' to escape it.",
                ));
            }
            other => current_text.push(other),
        }
    }

    flush_text(&mut tokens, &mut current_text, format_lit)?;
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::LitStr;

    #[test]
    fn test_max_tokens_at_boundary() {
        // Exactly MAX_TOKENS (256) works correctly
        // This test ensures that BOTH text and placeholder tokens count toward the limit.
        // If placeholders don't count (the vulnerability), this would fail.
        let format_lit: LitStr = syn::parse_quote!("{}");

        // 128 placeholders + 128 text separators = 256 tokens total
        let mut format = String::new();
        for _ in 0..128 {
            format.push_str("{} ");
        }

        let result = tokenize_format_string(&format, &format_lit);
        assert!(result.is_ok(), "Should accept exactly 256 tokens");
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 256, "Should have exactly 256 tokens");

        // Verify both placeholders and text are counted
        let placeholder_count = tokens
            .iter()
            .filter(|t| matches!(t, FormatToken::Placeholder(_)))
            .count();
        let text_count = tokens
            .iter()
            .filter(|t| matches!(t, FormatToken::Text(_)))
            .count();
        assert_eq!(placeholder_count, 128, "Should have 128 placeholders");
        assert_eq!(text_count, 128, "Should have 128 text tokens");
    }

    #[test]
    fn test_tokenization_basic() {
        let format_lit: LitStr = syn::parse_quote!("{x}");
        let result = tokenize_format_string("{x} text {y}", &format_lit);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3); // placeholder, text, placeholder
    }

    #[test]
    fn test_escaped_braces() {
        let format_lit: LitStr = syn::parse_quote!("{{}}");
        let result = tokenize_format_string("{{text}}", &format_lit);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1); // Single text token with literal braces
    }
}
