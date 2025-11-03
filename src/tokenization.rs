//! Format string tokenization at compile-time.

use crate::constants::{
    IDENTIFIER_CAPACITY, MAX_FORMAT_STRING_LEN, MAX_IDENTIFIER_LEN, MAX_TOKENS,
    TEXT_SEGMENT_CAPACITY, TOKENS_INITIAL_CAPACITY,
};
use crate::types::{FormatToken, Placeholder};
use crate::validation::is_valid_identifier;
use proc_macro::TokenStream;
use syn::LitStr;

/// Tokenize format string into text/placeholders. Handles `{{`/`}}` escapes.
///
/// Security: enforces MAX_FORMAT_STRING_LEN, MAX_TOKENS, MAX_IDENTIFIER_LEN limits.
pub fn tokenize_format_string(
    format_str: &str,
    format_lit: &LitStr,
) -> Result<Vec<FormatToken>, TokenStream> {
    if format_str.len() > MAX_FORMAT_STRING_LEN {
        return Err(syn::Error::new(
            format_lit.span(),
            format!(
                "Format string too long ({} bytes). Maximum allowed: {} bytes. \
                 This limit prevents compile-time DoS attacks.",
                format_str.len(),
                MAX_FORMAT_STRING_LEN
            ),
        )
        .to_compile_error()
        .into());
    }

    let mut tokens: Vec<FormatToken> = Vec::with_capacity(TOKENS_INITIAL_CAPACITY);
    let mut chars = format_str.chars().peekable();
    let mut current_text = String::with_capacity(TEXT_SEGMENT_CAPACITY);

    let push_token =
        |tokens: &mut Vec<FormatToken>, token: FormatToken| -> Result<(), TokenStream> {
            if tokens.len() >= MAX_TOKENS {
                return Err(syn::Error::new(
                    format_lit.span(),
                    format!(
                        "Too many tokens in format string (would exceed {}). Maximum allowed: {}. \
                     This limit prevents compile-time resource exhaustion.",
                        tokens.len() + 1,
                        MAX_TOKENS
                    ),
                )
                .to_compile_error()
                .into());
            }
            tokens.push(token);
            Ok(())
        };

    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    current_text.push('{');
                    continue;
                }

                if !current_text.is_empty() {
                    push_token(
                        &mut tokens,
                        FormatToken::Text(std::mem::take(&mut current_text).into_boxed_str()),
                    )?;
                    current_text = String::with_capacity(TEXT_SEGMENT_CAPACITY);
                }

                let mut content = String::with_capacity(IDENTIFIER_CAPACITY);
                for c2 in chars.by_ref() {
                    if c2 == '}' {
                        break;
                    }

                    if content.len() >= MAX_IDENTIFIER_LEN {
                        return Err(syn::Error::new(
                            format_lit.span(),
                            format!(
                                "Identifier in placeholder too long (>{} characters). \
                                 This limit prevents compile-time DoS attacks.",
                                MAX_IDENTIFIER_LEN
                            ),
                        )
                        .to_compile_error()
                        .into());
                    }

                    content.push(c2);
                }

                if content.is_empty() {
                    push_token(
                        &mut tokens,
                        FormatToken::Placeholder(Placeholder::Anonymous),
                    )?;
                } else if is_valid_identifier(&content) {
                    push_token(
                        &mut tokens,
                        FormatToken::Placeholder(Placeholder::Named(content.into_boxed_str())),
                    )?;
                } else {
                    return Err(syn::Error::new(
                        format_lit.span(),
                        format!(
                            "Invalid identifier '{}' in placeholder. \
                             Identifiers must start with a letter or underscore, \
                             contain only alphanumeric characters or underscores, \
                             and not be Rust keywords. Use '{{}}' for anonymous placeholders.",
                            content
                        ),
                    )
                    .to_compile_error()
                    .into());
                }
            }
            '}' => {
                if chars.peek() == Some(&'}') {
                    chars.next();
                    current_text.push('}');
                } else {
                    return Err(syn::Error::new(
                        format_lit.span(),
                        "Unescaped '}' in format string. Use '}}' to escape it.",
                    )
                    .to_compile_error()
                    .into());
                }
            }
            other => current_text.push(other),
        }
    }

    if !current_text.is_empty() {
        push_token(
            &mut tokens,
            FormatToken::Text(current_text.into_boxed_str()),
        )?;
    }

    Ok(tokens)
}
