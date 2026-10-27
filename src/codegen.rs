//! Code generation for scanf macros.

use crate::tokenization::tokenize_format_string;
use crate::types::{FormatToken, Placeholder};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::{Expr, Ident, LitStr, punctuated::Punctuated, spanned::Spanned, token::Comma};

/// Span for the macro's own locals (`remaining`, `parsed`, ...).
///
/// Mixed-site hygiene keeps them apart from user variables, so a `{remaining}` placeholder
/// assigns the user's `remaining` and never the internal one.
pub fn internal_span() -> Span {
    Span::mixed_site()
}

/// Output of code generation.
struct Generated {
    /// One statement per format token, in order.
    statements: Vec<TokenStream2>,
    /// Values returned by the macro, one per anonymous placeholder. Empty unless the format
    /// has anonymous placeholders and no explicit arguments (return mode).
    outputs: Vec<Ident>,
    /// Number of anonymous placeholders in the format.
    anonymous_count: usize,
}

impl Generated {
    /// `let out0; let out1; ...`: declarations the placeholders assign to later.
    fn declarations(&self) -> TokenStream2 {
        let outputs = &self.outputs;
        quote! { #(let #outputs;)* }
    }

    /// Final `Ok(..)`: `()` without outputs, the value for one, a tuple for several.
    fn result(&self) -> TokenStream2 {
        let outputs = &self.outputs;
        match outputs.len() {
            0 => quote! { Ok(()) },
            1 => quote! { Ok(#(#outputs)*) },
            _ => quote! { Ok((#(#outputs),*)) },
        }
    }
}

/// Place a placeholder assigns its parsed value to, and how error messages call it.
struct Target {
    tokens: TokenStream2,
    description: String,
}

impl Target {
    fn named(name: &str) -> Self {
        let ident = Ident::new(name, Span::call_site());
        Self {
            tokens: quote! { #ident },
            description: format!("variable '{name}'"),
        }
    }

    fn anonymous(tokens: TokenStream2, number: usize) -> Self {
        Self {
            tokens,
            description: format!("anonymous placeholder #{number}"),
        }
    }
}

/// Turns placeholders into parsing statements, resolving where each parsed value goes.
struct PlaceholderEmitter<'a> {
    explicit_args: &'a [&'a Expr],
    format_lit: &'a LitStr,
    /// Anonymous placeholders produce values instead of writing through explicit arguments.
    return_mode: bool,
    outputs: Vec<Ident>,
    anonymous_count: usize,
}

impl PlaceholderEmitter<'_> {
    /// Statement for `placeholder`: followed by `separator`, or, with `None`, the final
    /// placeholder that consumes the rest of the input.
    fn emit_placeholder(
        &mut self,
        placeholder: &Placeholder,
        separator: Option<&LitStr>,
    ) -> syn::Result<TokenStream2> {
        let target = match placeholder {
            Placeholder::Named(name) => Target::named(name),
            Placeholder::Anonymous => self.anonymous_target(separator.is_none())?,
        };
        Ok(match separator {
            Some(separator) => generate_placeholder_with_separator(&target, separator),
            None => generate_final_placeholder(&target),
        })
    }

    /// Next anonymous target: an output slot in return mode, otherwise the next explicit
    /// `&mut` argument.
    fn anonymous_target(&mut self, is_final: bool) -> syn::Result<Target> {
        let index = self.anonymous_count;
        let tokens = if self.return_mode {
            let out = Ident::new(&format!("out{index}"), internal_span());
            self.outputs.push(out.clone());
            quote! { #out }
        } else {
            let Some(arg_expr) = self.explicit_args.get(index) else {
                return Err(missing_argument_error(index + 1, is_final, self.format_lit));
            };
            quote! { *#arg_expr }
        };
        self.anonymous_count += 1;
        Ok(Target::anonymous(tokens, self.anonymous_count))
    }
}

/// Generate parsing code from tokens.
///
/// Errors on consecutive placeholders or missing args.
fn generate_parsing_code(
    tokens: &[FormatToken],
    explicit_args: &[&Expr],
    format_lit: &LitStr,
    return_mode: bool,
) -> syn::Result<Generated> {
    let mut resolver = PlaceholderEmitter {
        explicit_args,
        format_lit,
        return_mode,
        outputs: Vec::new(),
        anonymous_count: 0,
    };
    let mut statements = Vec::with_capacity(tokens.len());
    let mut pending_placeholder: Option<&Placeholder> = None;

    for token in tokens {
        match token {
            FormatToken::Placeholder(placeholder) => {
                if pending_placeholder.is_some() {
                    return Err(syn::Error::new(
                        format_lit.span(),
                        "Consecutive placeholders without separator are ambiguous and not supported. \
                         Add text between placeholders to separate them. Example: '{}:{}' instead of '{}{}'",
                    ));
                }
                pending_placeholder = Some(placeholder);
            }
            FormatToken::Text(text) => {
                let lit_text = LitStr::new(text, Span::call_site());
                statements.push(match pending_placeholder.take() {
                    Some(placeholder) => resolver.emit_placeholder(placeholder, Some(&lit_text))?,
                    None => generate_fixed_text_match(&lit_text),
                });
            }
        }
    }

    if let Some(placeholder) = pending_placeholder {
        statements.push(resolver.emit_placeholder(placeholder, None)?);
    }

    Ok(Generated {
        statements,
        outputs: resolver.outputs,
        anonymous_count: resolver.anonymous_count,
    })
}

/// Expression parsing `source`. With the `trim-whitespace` feature, a failed parse is retried
/// with surrounding whitespace trimmed; without it, input is parsed strictly as-is.
///
/// The macro cannot know the target type, so the retry is what keeps `String` (which never
/// fails to parse) untouched while numbers and other strict types tolerate stray whitespace.
/// On double failure the error of the first, untrimmed attempt is reported.
fn parse_with_trim_fallback(source: &TokenStream2) -> TokenStream2 {
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

/// Generate code for placeholder followed by a separator.
fn generate_placeholder_with_separator(target: &Target, separator: &LitStr) -> TokenStream2 {
    let Target {
        tokens,
        description,
    } = target;
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
                        #description,
                        remaining
                    )
                )
            })?;
            let slice = &remaining[..pos];
            let parsed = #parse_expr.map_err(|error| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Failed to parse {} from {:?}: {}", #description, slice, error)
                )
            })?;
            #tokens = parsed;
            remaining = &remaining[pos + #separator.len()..];
        }
    }
}

/// Generate code for fixed text matching at current position.
fn generate_fixed_text_match(text: &LitStr) -> TokenStream2 {
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
fn generate_final_placeholder(target: &Target) -> TokenStream2 {
    let Target {
        tokens,
        description,
    } = target;
    let span = internal_span();
    let remaining = quote_spanned! {span=> remaining };
    let parse_expr = parse_with_trim_fallback(&remaining);
    quote_spanned! {span=>
        {
            let parsed = #parse_expr.map_err(|error| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Failed to parse {} from remaining input {:?}: {}", #description, remaining, error)
                )
            })?;
            #tokens = parsed;
            remaining = "";
        }
    }
}

/// Create error for missing anonymous placeholder argument.
fn missing_argument_error(position: usize, is_final: bool, format_lit: &LitStr) -> syn::Error {
    let prefix = if is_final { "Final " } else { "" };
    syn::Error::new(
        format_lit.span(),
        format!(
            "{prefix}anonymous placeholder '{{}}' at position {position} has no corresponding argument. \
             Provide a mutable reference argument for every '{{}}' (e.g., &mut var), none at all to get the values returned, or use a named placeholder (e.g., '{{var}}')"
        ),
    )
}

/// Error when explicit arguments outnumber the anonymous placeholders.
fn check_all_arguments_used(explicit_args: &[&Expr], anonymous_count: usize) -> syn::Result<()> {
    if anonymous_count >= explicit_args.len() {
        return Ok(());
    }
    let unused_count = explicit_args.len() - anonymous_count;
    Err(syn::Error::new(
        explicit_args[anonymous_count].span(),
        format!(
            "Too many arguments: {unused_count} unused argument(s) provided. \
             The format string only has {anonymous_count} anonymous placeholder(s)"
        ),
    ))
}

/// Tokenize the format string, validate it and generate the parsing statements.
///
/// Errors on empty format, no content, unused args, or validation failures.
/// Anonymous placeholders without any explicit argument select return mode.
fn generate_statements(format_lit: &LitStr, explicit_args: &[&Expr]) -> syn::Result<Generated> {
    let format_str = format_lit.value();

    if format_str.is_empty() {
        return Err(syn::Error::new(
            format_lit.span(),
            "Format string cannot be empty. Provide at least one placeholder or literal text.",
        ));
    }

    let tokens = tokenize_format_string(&format_str, format_lit)?;

    if tokens.is_empty() {
        return Err(syn::Error::new(
            format_lit.span(),
            "Format string contains no parsable content",
        ));
    }

    // No explicit arguments plus anonymous placeholders: the macro returns the parsed values.
    let has_anonymous = tokens
        .iter()
        .any(|token| matches!(token, FormatToken::Placeholder(Placeholder::Anonymous)));
    let return_mode = explicit_args.is_empty() && has_anonymous;

    let generated = generate_parsing_code(&tokens, explicit_args, format_lit, return_mode)?;
    check_all_arguments_used(explicit_args, generated.anonymous_count)?;
    Ok(generated)
}

/// Expand a scanf-style macro: generate the parsing statements and wrap them in a closure
/// returning `io::Result`. `prelude` runs first and must define `remaining: &str`.
///
/// This is the single point where generation errors become compile errors.
pub fn expand(
    format_lit: &LitStr,
    args: &Punctuated<Expr, Comma>,
    prelude: TokenStream2,
) -> proc_macro::TokenStream {
    let explicit_args: Vec<_> = args.iter().collect();
    let generated = match generate_statements(format_lit, &explicit_args) {
        Ok(generated) => generated,
        Err(error) => return error.to_compile_error().into(),
    };

    let span = internal_span();
    let declarations = generated.declarations();
    let statements = &generated.statements;
    let result = generated.result();
    quote_spanned! {span=> {
        (|| -> std::io::Result<_> {
            #declarations
            #prelude
            #(#statements)*
            #result
        })()
    }}
    .into()
}
