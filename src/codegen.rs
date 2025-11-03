//! Code generation for scanf macros.

use crate::tokenization::tokenize_format_string;
use crate::types::{FormatToken, Placeholder};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Expr, Ident, LitStr, spanned::Spanned};

/// Generate parsing code from tokens.
///
/// Returns `(code, anon_count)` or error for consecutive placeholders / missing args.
pub fn generate_parsing_code(
    tokens: &[FormatToken],
    explicit_args: &[&Expr],
    format_lit: &LitStr,
) -> Result<(Vec<proc_macro2::TokenStream>, usize), TokenStream> {
    let mut generated = Vec::with_capacity(tokens.len());
    let mut pending_placeholder: Option<Placeholder> = None;
    let mut anon_index: usize = 0;

    for token in tokens {
        match token {
            FormatToken::Placeholder(ph) => {
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
                    match ph {
                        Placeholder::Named(name) => {
                            generated
                                .push(generate_named_placeholder_with_separator(&name, &lit_text));
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
                                arg_expr, anon_index, &lit_text,
                            ));
                        }
                    }
                } else {
                    generated.push(generate_fixed_text_match(&lit_text));
                }
            }
        }
    }

    if let Some(ph) = pending_placeholder {
        match ph {
            Placeholder::Named(name) => {
                generated.push(generate_final_named_placeholder(&name));
            }
            Placeholder::Anonymous => {
                if anon_index >= explicit_args.len() {
                    return Err(make_missing_argument_error(
                        anon_index + 1,
                        true,
                        format_lit,
                    ));
                }
                let arg_expr = explicit_args[anon_index];
                anon_index += 1;
                generated.push(generate_final_anonymous_placeholder(arg_expr, anon_index));
            }
        }
    }

    Ok((generated, anon_index))
}

/// Generates code for a named placeholder followed by a separator.
fn generate_named_placeholder_with_separator(
    name: &str,
    separator: &LitStr,
) -> proc_macro2::TokenStream {
    let ident = Ident::new(name, Span::call_site());
    let var_name = format!("variable '{}'", name);

    quote! {
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
        if let Some(pos) = remaining.find(#text) {
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
        remaining = "";
    }
}

/// Generates code for a final anonymous placeholder (consumes rest of input).
fn generate_final_anonymous_placeholder(
    arg_expr: &Expr,
    placeholder_num: usize,
) -> proc_macro2::TokenStream {
    quote! {
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
        remaining = "";
    }
}

/// Create error for missing anonymous placeholder argument.
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

/// Generate complete scanf implementation: tokenize, validate, codegen.
///
/// Errors on empty format, no content, unused args, or validation failures.
pub fn generate_scanf_implementation(
    format_lit: &LitStr,
    explicit_args: &[&Expr],
) -> Result<Vec<proc_macro2::TokenStream>, TokenStream> {
    let format_str = format_lit.value();

    if format_str.is_empty() {
        return Err(syn::Error::new(
            format_lit.span(),
            "Format string cannot be empty. Provide at least one placeholder or literal text.",
        )
        .to_compile_error()
        .into());
    }

    let tokens = tokenize_format_string(&format_str, format_lit)?;

    if tokens.is_empty() {
        return Err(syn::Error::new(
            format_lit.span(),
            "Format string contains no parsable content",
        )
        .to_compile_error()
        .into());
    }

    let (generated, anon_index) = generate_parsing_code(&tokens, explicit_args, format_lit)?;

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
