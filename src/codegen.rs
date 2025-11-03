//! Code generation for scanf macros.
//!
//! This module generates the Rust code that performs runtime parsing
//! based on the tokenized format string.

use crate::tokenization::tokenize_format_string;
use crate::types::{FormatToken, Placeholder};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{spanned::Spanned, Expr, Ident, LitStr};

/// Generates parsing code from the tokenized format string.
///
/// This function takes the tokenized format string and generates the corresponding
/// Rust code that will perform parsing of the input according to the specification.
///
/// # Algorithm
///
/// For each token:
/// - **Literal text**: Searches for and consumes that exact text from input
/// - **Placeholder + Text**: Searches for the text and parses everything before it
/// - **Final placeholder**: Parses all remaining input
///
/// # Error Handling
///
/// Errors accumulate in a `result` variable that combines multiple errors
/// using the `.and(Err(...))` pattern. This allows parsing to continue
/// to provide better error feedback.
///
/// # Design Note
///
/// Code for Named and Anonymous placeholders is similar but NOT extracted
/// into a helper function because:
/// - Error messages are different and specific to each case
/// - Clarity is more important than extreme DRY
/// - Inline code is easier to understand and maintain (human-first)
///
/// # Arguments
///
/// - `tokens`: The tokenized format string
/// - `explicit_args`: Arguments provided for anonymous placeholders
/// - `format_lit`: The original format string literal for error reporting
///
/// # Returns
///
/// Returns `Ok((generated_code, anonymous_count))` on success:
/// - `generated_code`: Vec of token streams for generated parsing code
/// - `anonymous_count`: Number of anonymous placeholders processed
///
/// # Errors
///
/// Returns a compile error if:
/// - Consecutive placeholders without separator are found (ambiguous parsing)
/// - Anonymous placeholders don't have corresponding arguments
pub fn generate_parsing_code(
    tokens: &[FormatToken],
    explicit_args: &[&Expr],
    format_lit: &LitStr,
) -> Result<(Vec<proc_macro2::TokenStream>, usize), TokenStream> {
    // Pre-allocate: typically one code block per token
    let mut generated = Vec::with_capacity(tokens.len());
    let mut pending_placeholder: Option<Placeholder> = None;
    let mut anon_index: usize = 0;

    for token in tokens {
        match token {
            FormatToken::Placeholder(ph) => {
                // Check for consecutive placeholders (ambiguous)
                if pending_placeholder.is_some() {
                    return Err(syn::Error::new(
                        format_lit.span(),
                        "Consecutive placeholders without separator are ambiguous and not supported. \
                         Add text between placeholders to separate them. Example: '{}:{}' instead of '{}{}'",
                    )
                    .to_compile_error()
                    .into());
                }
                pending_placeholder = Some(ph.clone());
            }
            FormatToken::Text(text) => {
                let lit_text = LitStr::new(text, Span::call_site());

                if let Some(ph) = pending_placeholder.take() {
                    // Generate code for placeholder followed by text
                    match ph {
                        Placeholder::Named(name) => {
                            generated.push(generate_named_placeholder_with_separator(
                                &name, &lit_text,
                            ));
                        }
                        Placeholder::Anonymous => {
                            if anon_index >= explicit_args.len() {
                                return Err(make_missing_argument_error(
                                    anon_index + 1,
                                    false,
                                    format_lit,
                                ));
                            }
                            let arg_expr = explicit_args[anon_index];
                            anon_index += 1;
                            generated.push(generate_anonymous_placeholder_with_separator(
                                arg_expr,
                                anon_index,
                                &lit_text,
                            ));
                        }
                    }
                } else {
                    // No placeholder - just fixed text that must match
                    generated.push(generate_fixed_text_match(&lit_text));
                }
            }
        }
    }

    // Handle final pending placeholder (consumes rest of input)
    if let Some(ph) = pending_placeholder {
        match ph {
            Placeholder::Named(name) => {
                generated.push(generate_final_named_placeholder(&name));
            }
            Placeholder::Anonymous => {
                if anon_index >= explicit_args.len() {
                    return Err(make_missing_argument_error(anon_index + 1, true, format_lit));
                }
                let arg_expr = explicit_args[anon_index];
                anon_index += 1;
                generated.push(generate_final_anonymous_placeholder(arg_expr, anon_index));
            }
        }
    }

    Ok((generated, anon_index))
}

// ============================================================================
// Code Generation Helpers
// ============================================================================

/// Generates code for a named placeholder followed by a separator.
fn generate_named_placeholder_with_separator(
    name: &str,
    separator: &LitStr,
) -> proc_macro2::TokenStream {
    let ident = Ident::new(name, Span::call_site());
    let var_name = format!("variable '{}'", name);

    quote! {
        // Parse named placeholder into variable
        if let Some(pos) = remaining.find(#separator) {
            let slice = &remaining[..pos];
            match slice.parse() {
                Ok(parsed) => {
                    #ident = parsed;
                }
                Err(error) => {
                    result = result.and(Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Failed to parse {} from {:?}: {}", #var_name, slice, error)
                    )));
                }
            }
            remaining = &remaining[pos + #separator.len()..];
        } else {
            result = result.and(Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Expected separator {:?} for {} not found in remaining input: {:?}",
                    #separator,
                    #var_name,
                    remaining
                )
            )));
        }
    }
}

/// Generates code for an anonymous placeholder followed by a separator.
fn generate_anonymous_placeholder_with_separator(
    arg_expr: &Expr,
    placeholder_num: usize,
    separator: &LitStr,
) -> proc_macro2::TokenStream {
    quote! {
        // Parse anonymous placeholder (argument position)
        if let Some(pos) = remaining.find(#separator) {
            let slice = &remaining[..pos];
            match slice.parse() {
                Ok(parsed) => {
                    *#arg_expr = parsed;
                }
                Err(error) => {
                    result = result.and(Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!(
                            "Failed to parse anonymous placeholder #{} from {:?}: {}",
                            #placeholder_num,
                            slice,
                            error
                        )
                    )));
                }
            }
            remaining = &remaining[pos + #separator.len()..];
        } else {
            result = result.and(Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Expected separator {:?} for anonymous placeholder #{} not found in remaining input: {:?}",
                    #separator,
                    #placeholder_num,
                    remaining
                )
            )));
        }
    }
}

/// Generates code for matching fixed text at current position.
fn generate_fixed_text_match(text: &LitStr) -> proc_macro2::TokenStream {
    quote! {
        // Match required fixed text
        if let Some(pos) = remaining.find(#text) {
            // Ensure we match immediately at position 0 (no skipping)
            if pos == 0 {
                remaining = &remaining[#text.len()..];
            } else {
                result = result.and(Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "Expected text {:?} at current position, but found it at offset {}. \
                         Remaining input: {:?}",
                        #text,
                        pos,
                        remaining
                    )
                )));
            }
        } else {
            result = result.and(Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Required text separator {:?} not found. Remaining input: {:?}",
                    #text,
                    remaining
                )
            )));
        }
    }
}

/// Generates code for a final named placeholder (consumes rest of input).
fn generate_final_named_placeholder(name: &str) -> proc_macro2::TokenStream {
    let ident = Ident::new(name, Span::call_site());
    let var_name = format!("variable '{}'", name);

    quote! {
        // Parse final named placeholder (consumes all remaining input)
        match remaining.parse() {
            Ok(parsed) => {
                #ident = parsed;
            }
            Err(error) => {
                result = result.and(Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Failed to parse {} from remaining input {:?}: {}", #var_name, remaining, error)
                )));
            }
        }
        remaining = ""; // consumed
    }
}

/// Generates code for a final anonymous placeholder (consumes rest of input).
fn generate_final_anonymous_placeholder(
    arg_expr: &Expr,
    placeholder_num: usize,
) -> proc_macro2::TokenStream {
    quote! {
        // Parse final anonymous placeholder (consumes all remaining input)
        match remaining.parse() {
            Ok(parsed) => {
                *#arg_expr = parsed;
            }
            Err(error) => {
                result = result.and(Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "Failed to parse anonymous placeholder #{} from remaining input {:?}: {}",
                        #placeholder_num,
                        remaining,
                        error
                    )
                )));
            }
        }
        remaining = ""; // consumed
    }
}

// ============================================================================
// Error Generation Helpers
// ============================================================================

/// Creates an error for missing anonymous placeholder argument.
///
/// # Arguments
///
/// - `position`: The position of the placeholder (1-indexed)
/// - `is_final`: Whether this is the final placeholder in the format string
/// - `format_lit`: The format string literal for error span
fn make_missing_argument_error(
    position: usize,
    is_final: bool,
    format_lit: &LitStr,
) -> TokenStream {
    let prefix = if is_final { "Final " } else { "" };
    syn::Error::new(
        format_lit.span(),
        format!(
            "{}anonymous placeholder '{{}}' at position {} has no corresponding argument. \
             Provide a mutable reference argument (e.g., &mut var) or use a named placeholder (e.g., '{{var}}')",
            prefix, position
        ),
    )
    .to_compile_error()
    .into()
}

/// Generates complete scanf implementation from format string and arguments.
///
/// This is the top-level code generation function that orchestrates tokenization,
/// validation, and code generation.
///
/// # Arguments
///
/// - `format_lit`: The format string literal
/// - `explicit_args`: Arguments provided for anonymous placeholders
///
/// # Returns
///
/// Returns the generated parsing code on success, or a compile error on failure.
///
/// # Errors
///
/// Returns a compile error if:
/// - Format string is empty
/// - Format string contains no parseable content
/// - There are unused arguments
/// - Any validation or tokenization errors occur
pub fn generate_scanf_implementation(
    format_lit: &LitStr,
    explicit_args: &[&Expr],
) -> Result<Vec<proc_macro2::TokenStream>, TokenStream> {
    let format_str = format_lit.value();

    // Validate format string is not empty
    if format_str.is_empty() {
        return Err(syn::Error::new(
            format_lit.span(),
            "Format string cannot be empty. Provide at least one placeholder or literal text.",
        )
        .to_compile_error()
        .into());
    }

    // Tokenize the format string at compile-time
    let tokens = tokenize_format_string(&format_str, format_lit)?;

    // Validate there's at least something to parse
    if tokens.is_empty() {
        return Err(syn::Error::new(
            format_lit.span(),
            "Format string contains no parseable content",
        )
        .to_compile_error()
        .into());
    }

    // Generate the parsing code
    let (generated, anon_index) = generate_parsing_code(&tokens, explicit_args, format_lit)?;

    // Check if there are unused arguments
    if anon_index < explicit_args.len() {
        let unused_count = explicit_args.len() - anon_index;
        return Err(syn::Error::new(
            explicit_args[anon_index].span(),
            format!(
                "Too many arguments: {} unused argument(s) provided. \
                 The format string only has {} anonymous placeholder(s)",
                unused_count, anon_index
            ),
        )
        .to_compile_error()
        .into());
    }

    Ok(generated)
}
