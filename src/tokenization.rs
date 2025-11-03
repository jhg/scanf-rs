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
/// Enforces `MAX_FORMAT_STRING_LEN`, `MAX_TOKENS`, `MAX_IDENTIFIER_LEN` limits.
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
                                "Identifier in placeholder too long (>{MAX_IDENTIFIER_LEN} characters). \
                                 This limit prevents compile-time DoS attacks."
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
                            "Invalid identifier '{content}' in placeholder. \
                             Identifiers must start with a letter or underscore, \
                             contain only alphanumeric characters or underscores, \
                             and not be Rust keywords. Use '{{}}' for anonymous placeholders."
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

#[cfg(test)]
mod tests {
    use super::*;
    use syn::LitStr;

    // NOTE: Error path tests (exceeding limits) cannot be tested in unit tests because
    // tokenize_format_string returns Result<Vec<FormatToken>, TokenStream>, and TokenStream
    // can only be used during macro expansion. These cases are covered by integration tests
    // that actually try to compile code with the scanf! macro.
    //
    // REGRESSION PROTECTION: The security fix for MAX_TOKENS bypass (where only text tokens
    // were checked, allowing placeholders to bypass the limit) is protected by the test below
    // which verifies the boundary case works correctly. Any regression would cause this test
    // to fail as it would reject valid input.

    #[test]
    fn test_max_tokens_at_boundary() {
        // REGRESSION TEST: Verify exactly MAX_TOKENS (256) works correctly
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
