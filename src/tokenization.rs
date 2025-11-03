//! Format string tokenization at compile-time.

use crate::constants::*;
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
    // Security: Protect against DoS via extremely long format strings
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

    // Pre-allocate for typical case (2-4 tokens)
    let mut tokens: Vec<FormatToken> = Vec::with_capacity(TOKENS_INITIAL_CAPACITY);
    let mut chars = format_str.chars().peekable();
    // Pre-allocate for typical separator length
    let mut current_text = String::with_capacity(TEXT_SEGMENT_CAPACITY);

    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                if chars.peek() == Some(&'{') {
                    // Escaped open brace: {{ becomes {
                    chars.next();
                    current_text.push('{');
                    continue;
                }

                // Flush accumulated text before processing placeholder
                if !current_text.is_empty() {
                    tokens.push(FormatToken::Text(std::mem::take(&mut current_text)));

                    // Security check: prevent excessive token creation
                    if tokens.len() > MAX_TOKENS {
                        return Err(syn::Error::new(
                            format_lit.span(),
                            format!(
                                "Too many tokens in format string ({}). Maximum allowed: {}. \
                                 This limit prevents compile-time resource exhaustion.",
                                tokens.len(),
                                MAX_TOKENS
                            ),
                        )
                        .to_compile_error()
                        .into());
                    }

                    // Reset capacity hint for next text segment
                    current_text = String::with_capacity(TEXT_SEGMENT_CAPACITY);
                }

                // Capture placeholder content
                let mut content = String::with_capacity(IDENTIFIER_CAPACITY);
                for c2 in chars.by_ref() {
                    if c2 == '}' {
                        break;
                    }

                    // Security check: prevent extremely long identifiers
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

                // Process the captured placeholder
                if content.is_empty() {
                    // Anonymous placeholder: {}
                    tokens.push(FormatToken::Placeholder(Placeholder::Anonymous));
                } else if is_valid_identifier(&content) {
                    // Named placeholder: {identifier}
                    // Convert String to Box<str> for memory efficiency (33% savings)
                    tokens.push(FormatToken::Placeholder(Placeholder::Named(
                        content.into_boxed_str(),
                    )));
                } else {
                    // Invalid identifier - return helpful error
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
                    // Escaped close brace: }} becomes }
                    chars.next();
                    current_text.push('}');
                } else {
                    // Unescaped single '}' is invalid
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

    // Flush any remaining text
    if !current_text.is_empty() {
        tokens.push(FormatToken::Text(current_text));
    }

    Ok(tokens)
}
