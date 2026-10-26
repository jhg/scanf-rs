//! Code generation for scanf macros.

use crate::tokenization::tokenize_format_string;
use crate::types::{FormatToken, Placeholder};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{Expr, Ident, LitStr, spanned::Spanned};

/// Span for the macro's own locals (`remaining`, `parsed`, ...).
///
/// Mixed-site hygiene keeps them apart from user variables, so a `{remaining}` placeholder
/// assigns the user's `remaining` and never the internal one.
pub fn internal_span() -> Span {
    Span::mixed_site()
}

/// Output of code generation.
pub struct Generated {
    /// One statement per format token, in order.
    pub statements: Vec<proc_macro2::TokenStream>,
    /// Values returned by the macro, one per anonymous placeholder. Empty unless the format
    /// has anonymous placeholders and no explicit arguments (return mode).
    outputs: Vec<Ident>,
}

impl Generated {
    /// `let out0; let out1; ...`: declarations the placeholders assign to later.
    pub fn declarations(&self) -> proc_macro2::TokenStream {
        let outputs = &self.outputs;
        quote! { #(let #outputs;)* }
    }

    /// Final `Ok(..)`: `()` without outputs, the value for one, a tuple for several.
    pub fn result(&self) -> proc_macro2::TokenStream {
        let outputs = &self.outputs;
        match outputs.len() {
            0 => quote! { Ok(()) },
            1 => quote! { Ok(#(#outputs)*) },
            _ => quote! { Ok((#(#outputs),*)) },
        }
    }
}

/// Target an anonymous placeholder assigns to: an output slot in return mode, otherwise
/// the next explicit `&mut` argument.
fn anonymous_target(
    index: usize,
    is_final: bool,
    explicit_args: &[&Expr],
    outputs: Option<&mut Vec<Ident>>,
    format_lit: &LitStr,
) -> Result<proc_macro2::TokenStream, TokenStream> {
    if let Some(outputs) = outputs {
        let out = Ident::new(&format!("out{index}"), internal_span());
        outputs.push(out.clone());
        return Ok(quote! { #out });
    }
    let Some(arg_expr) = explicit_args.get(index) else {
        return Err(make_missing_argument_error(index + 1, is_final, format_lit));
    };
    Ok(quote! { *#arg_expr })
}

/// Generate parsing code from tokens.
///
/// Returns the generated code and the anonymous placeholder count, or an error for
/// consecutive placeholders / missing args. `return_mode` makes anonymous placeholders
/// produce values instead of writing through explicit arguments.
fn generate_parsing_code(
    tokens: &[FormatToken],
    explicit_args: &[&Expr],
    format_lit: &LitStr,
    return_mode: bool,
) -> Result<(Generated, usize), TokenStream> {
    let mut generated = Vec::with_capacity(tokens.len());
    let mut outputs = Vec::new();
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
                            let target = anonymous_target(
                                anon_index,
                                false,
                                explicit_args,
                                return_mode.then_some(&mut outputs),
                                format_lit,
                            )?;
                            anon_index += 1;
                            generated.push(generate_anonymous_placeholder_with_separator(
                                &target, anon_index, &lit_text,
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
                let target = anonymous_target(
                    anon_index,
                    true,
                    explicit_args,
                    return_mode.then_some(&mut outputs),
                    format_lit,
                )?;
                anon_index += 1;
                generated.push(generate_final_anonymous_placeholder(&target, anon_index));
            }
        }
    }

    Ok((
        Generated {
            statements: generated,
            outputs,
        },
        anon_index,
    ))
}

/// Expression parsing `source`. With the `trim-whitespace` feature, a failed parse is retried
/// with surrounding whitespace trimmed; without it, input is parsed strictly as-is.
///
/// The macro cannot know the target type, so the retry is what keeps `String` (which never
/// fails to parse) untouched while numbers and other strict types tolerate stray whitespace.
/// On double failure the error of the first, untrimmed attempt is reported.
fn parse_with_trim_fallback(source: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let span = internal_span();
    if !cfg!(feature = "trim-whitespace") {
        return quote_spanned! {span=> #source.parse() };
    }
    quote_spanned! {span=>
        match #source.parse() {
            Ok(value) => Ok(value),
            Err(error) => match #source.trim().parse() {
                Ok(value) => Ok(value),
                Err(_) => Err(error),
            },
        }
    }
}

/// Generate code for placeholder with separator (named or anonymous).
///
/// `target` is the place the parsed value is assigned to (`name`, `*arg` or an output slot).
fn generate_placeholder_with_separator(
    target: &proc_macro2::TokenStream,
    var_desc: &str,
    separator: &LitStr,
) -> proc_macro2::TokenStream {
    let span = internal_span();
    let slice = quote_spanned! {span=> slice };
    let parse_expr = parse_with_trim_fallback(&slice);
    quote_spanned! {span=>
        {
            let pos = remaining.find(#separator).ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "Expected separator {:?} for {} not found in remaining input: {:?}",
                        #separator,
                        #var_desc,
                        remaining
                    )
                )
            })?;
            let slice = &remaining[..pos];
            let parsed = #parse_expr.map_err(|error| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Failed to parse {} from {:?}: {}", #var_desc, slice, error)
                )
            })?;
            #target = parsed;
            remaining = &remaining[pos + #separator.len()..];
        }
    }
}

/// Generate code for named placeholder with separator.
fn generate_named_placeholder_with_separator(
    name: &str,
    separator: &LitStr,
) -> proc_macro2::TokenStream {
    let ident = Ident::new(name, Span::call_site());
    let var_desc = format!("variable '{name}'");
    generate_placeholder_with_separator(&quote! { #ident }, &var_desc, separator)
}

/// Generate code for anonymous placeholder with separator.
fn generate_anonymous_placeholder_with_separator(
    target: &proc_macro2::TokenStream,
    placeholder_num: usize,
    separator: &LitStr,
) -> proc_macro2::TokenStream {
    let var_desc = format!("anonymous placeholder #{placeholder_num}");
    generate_placeholder_with_separator(target, &var_desc, separator)
}

/// Generate code for fixed text matching at current position.
fn generate_fixed_text_match(text: &LitStr) -> proc_macro2::TokenStream {
    let span = internal_span();
    quote_spanned! {span=>
        {
            if !remaining.starts_with(#text) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "Expected text {:?} at current position. Remaining input: {:?}",
                        #text,
                        remaining
                    )
                ));
            }
            remaining = &remaining[#text.len()..];
        }
    }
}

/// Generate code for final placeholder (consumes rest of input).
fn generate_final_placeholder(
    target: &proc_macro2::TokenStream,
    var_desc: &str,
) -> proc_macro2::TokenStream {
    let span = internal_span();
    let remaining = quote_spanned! {span=> remaining };
    let parse_expr = parse_with_trim_fallback(&remaining);
    quote_spanned! {span=>
        {
            let parsed = #parse_expr.map_err(|error| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Failed to parse {} from remaining input {:?}: {}", #var_desc, remaining, error)
                )
            })?;
            #target = parsed;
            remaining = "";
        }
    }
}

/// Generate code for final named placeholder (consumes rest of input).
fn generate_final_named_placeholder(name: &str) -> proc_macro2::TokenStream {
    let ident = Ident::new(name, Span::call_site());
    let var_desc = format!("variable '{name}'");
    generate_final_placeholder(&quote! { #ident }, &var_desc)
}

/// Generate code for final anonymous placeholder (consumes rest of input).
fn generate_final_anonymous_placeholder(
    target: &proc_macro2::TokenStream,
    placeholder_num: usize,
) -> proc_macro2::TokenStream {
    let var_desc = format!("anonymous placeholder #{placeholder_num}");
    generate_final_placeholder(target, &var_desc)
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
            "{prefix}anonymous placeholder '{{}}' at position {position} has no corresponding argument. \
             Provide a mutable reference argument for every '{{}}' (e.g., &mut var), none at all to get the values returned, or use a named placeholder (e.g., '{{var}}')"
        ),
    )
    .to_compile_error()
    .into()
}

/// Generate complete scanf implementation: tokenize, validate, codegen.
///
/// Errors on empty format, no content, unused args, or validation failures.
/// Anonymous placeholders without any explicit argument select return mode.
pub fn generate_scanf_implementation(
    format_lit: &LitStr,
    explicit_args: &[&Expr],
) -> Result<Generated, TokenStream> {
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

    // No explicit arguments plus anonymous placeholders: the macro returns the parsed values.
    let has_anonymous = tokens
        .iter()
        .any(|token| matches!(token, FormatToken::Placeholder(Placeholder::Anonymous)));
    let return_mode = explicit_args.is_empty() && has_anonymous;

    let (generated, anon_index) =
        generate_parsing_code(&tokens, explicit_args, format_lit, return_mode)?;

    if anon_index < explicit_args.len() {
        let unused_count = explicit_args.len() - anon_index;
        return Err(syn::Error::new(
            explicit_args[anon_index].span(),
            format!(
                "Too many arguments: {unused_count} unused argument(s) provided. \
                 The format string only has {anon_index} anonymous placeholder(s)"
            ),
        )
        .to_compile_error()
        .into());
    }

    Ok(generated)
}
