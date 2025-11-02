#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]
#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    Expr, Ident, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
};

/// Arguments for the `sscanf!` macro.
/// 
/// Consists of:
/// - `input`: The string expression to parse
/// - `format`: The format string literal containing placeholders
/// - `args`: Optional explicit arguments for anonymous placeholders
struct SscanfArgs {
    input: Expr,
    format: LitStr,
    args: Punctuated<Expr, Comma>,
}

impl Parse for SscanfArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let input_expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let format = input.parse()?;

        let mut args = Punctuated::new();
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            args.push(input.parse()?);
        }

        Ok(SscanfArgs {
            input: input_expr,
            format,
            args,
        })
    }
}

/// Represents a placeholder in a format string.
/// 
/// Placeholders can be either named (e.g., `{variable}`) or anonymous (e.g., `{}`).
#[derive(Debug, PartialEq, Clone)]
enum Placeholder {
    /// A named placeholder that captures to a specific variable
    Named(String),
    /// An anonymous placeholder that requires an explicit argument
    Anonymous,
}



/// Checks if a string is a valid Rust identifier.
/// 
/// A valid identifier must:
/// - Start with an alphabetic character or underscore
/// - Contain only alphanumeric characters or underscores
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    chars.all(|c| c.is_alphanumeric() || c == '_')
}

#[proc_macro]
pub fn sscanf(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as SscanfArgs);

    let input_expr = &args.input;
    let format_lit = &args.format;
    let format_str = format_lit.value();
    let explicit_args: Vec<_> = args.args.iter().collect();

    // Compile-time tokenization (text segments + placeholder sequence) to eliminate runtime cost
    #[derive(Debug, Clone)]
    enum CTToken {
        Text(String),
        Placeholder(Placeholder),
    }

    let mut ct_tokens: Vec<CTToken> = Vec::new();
    let mut chars = format_str.chars().peekable();
    let mut current_text = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                if chars.peek() == Some(&'{') {
                    // Escaped open brace
                    chars.next();
                    current_text.push('{');
                    continue;
                }
                // Flush accumulated text
                if !current_text.is_empty() {
                    ct_tokens.push(CTToken::Text(std::mem::take(&mut current_text)));
                }
                // Capture placeholder content
                let mut content = String::new();
                for c2 in chars.by_ref() {
                    if c2 == '}' {
                        break;
                    }
                    content.push(c2);
                }
                if content.is_empty() {
                    ct_tokens.push(CTToken::Placeholder(Placeholder::Anonymous));
                } else if is_valid_identifier(&content) {
                    ct_tokens.push(CTToken::Placeholder(Placeholder::Named(content)));
                } else {
                    // Invalid identifier => treat as anonymous to mirror runtime behavior
                    ct_tokens.push(CTToken::Placeholder(Placeholder::Anonymous));
                }
            }
            '}' => {
                if chars.peek() == Some(&'}') {
                    // Escaped close brace
                    chars.next();
                    current_text.push('}');
                } else {
                    // Unescaped single '}' is invalid in runtime parser; we signal compile-time error
                    return syn::Error::new(format_lit.span(), "Unescaped '}' in format string")
                        .to_compile_error()
                        .into();
                }
            }
            other => current_text.push(other),
        }
    }
    if !current_text.is_empty() {
        ct_tokens.push(CTToken::Text(current_text));
    }

    // Generate code implementing mismo algoritmo que runtime InputFormatParser::inputs
    let mut generated = Vec::new();
    let mut pending_placeholder: Option<Placeholder> = None;
    let mut anon_index: usize = 0; // index into explicit_args

    for token in &ct_tokens {
        match token {
            CTToken::Placeholder(ph) => {
                if pending_placeholder.is_some() {
                    return syn::Error::new(
                        format_lit.span(),
                        "Consecutive placeholders without separator are not supported",
                    )
                    .to_compile_error()
                    .into();
                }
                pending_placeholder = Some((*ph).clone());
            }
            CTToken::Text(text) => {
                let lit_text = LitStr::new(text, Span::call_site());
                // When we reach text we must resolve pending placeholder (if any)
                if let Some(ph) = pending_placeholder.take() {
                    match ph {
                        Placeholder::Named(name) => {
                            let ident = Ident::new(&name, Span::call_site());
                            generated.push(quote! {
                                if let Some(pos) = __rest.find(#lit_text) {
                                    let slice = &__rest[..pos];
                                    match slice.parse() {
                                        Ok(parsed) => {
                                            #ident = parsed;
                                        }
                                        Err(error) => {
                                            __result = __result.and(Err(std::io::Error::new(
                                                std::io::ErrorKind::InvalidInput,
                                                error
                                            )));
                                        }
                                    }
                                    __rest = &__rest[pos + #lit_text.len()..];
                                } else {
                                    __result = __result.and(Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidInput,
                                        format!("Can not find text separator {:?}", #lit_text)
                                    )));
                                }
                            });
                        }
                        Placeholder::Anonymous => {
                            if anon_index >= explicit_args.len() {
                                return syn::Error::new(
                                    format_lit.span(),
                                    "Anonymous placeholder '{}' found but no corresponding argument provided"
                                )
                                .to_compile_error()
                                .into();
                            }
                            let arg_expr = explicit_args[anon_index];
                            anon_index += 1;
                            generated.push(quote! {
                                if let Some(pos) = __rest.find(#lit_text) {
                                    let slice = &__rest[..pos];
                                    match slice.parse() {
                                        Ok(parsed) => {
                                            *#arg_expr = parsed;
                                        }
                                        Err(error) => {
                                            __result = __result.and(Err(std::io::Error::new(
                                                std::io::ErrorKind::InvalidInput,
                                                error
                                            )));
                                        }
                                    }
                                    __rest = &__rest[pos + #lit_text.len()..];
                                } else {
                                    __result = __result.and(Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidInput,
                                        format!("Can not find text separator {:?}", #lit_text)
                                    )));
                                }
                            });
                        }
                    }
                } else {
                    // Just advance over required fixed text at start or between plain texts (rare)
                    generated.push(quote! {
                        if let Some(pos) = __rest.find(#lit_text) {
                            // Ensure we match immediately at position 0
                            if pos == 0 {
                                __rest = &__rest[#lit_text.len()..];
                            } else {
                                __result = __result.and(Err(std::io::Error::new(
                                    std::io::ErrorKind::InvalidInput,
                                    format!("Expected text {:?} at current position", #lit_text)
                                )));
                            }
                        } else {
                            __result = __result.and(Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                format!("Can not find text separator {:?}", #lit_text)
                            )));
                        }
                    });
                }
            }
        }
    }

    // Final pending placeholder consumes rest
    if let Some(ph) = pending_placeholder.take() {
        match ph {
            Placeholder::Named(name) => {
                let ident = Ident::new(&name, Span::call_site());
                generated.push(quote! {
                    match __rest.parse() {
                        Ok(parsed) => {
                            #ident = parsed;
                        }
                        Err(error) => {
                            __result = __result.and(Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                error
                            )));
                        }
                    }
                    __rest = ""; // consumed
                });
            }
            Placeholder::Anonymous => {
                if anon_index >= explicit_args.len() {
                    return syn::Error::new(
                        format_lit.span(),
                        "Anonymous placeholder '{}' found but no corresponding argument provided",
                    )
                    .to_compile_error()
                    .into();
                }
                let arg_expr = explicit_args[anon_index];
                anon_index += 1;
                generated.push(quote! {
                    match __rest.parse() {
                        Ok(parsed) => {
                            *#arg_expr = parsed;
                        }
                        Err(error) => {
                            __result = __result.and(Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                error
                            )));
                        }
                    }
                    __rest = ""; // consumed
                });
            }
        }
    }

    if anon_index < explicit_args.len() {
        return syn::Error::new(
            explicit_args[anon_index].span(),
            "Too many arguments provided for format string",
        )
        .to_compile_error()
        .into();
    }

    let expanded = quote! {{
        let mut __result: std::io::Result<()> = Ok(());
        let mut __rest = #input_expr;
        #(#generated)*
        __result
    }};

    TokenStream::from(expanded)
}

// ===== scanf! procedural macro (reads from stdin) =====

/// Arguments for the `scanf!` macro.
/// 
/// Consists of:
/// - `format`: The format string literal containing placeholders
/// - `args`: Optional explicit arguments for anonymous placeholders
struct ScanfArgs {
    format: LitStr,
    args: Punctuated<Expr, Comma>,
}

impl Parse for ScanfArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let format: LitStr = input.parse()?;
        let mut args = Punctuated::new();
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            args.push(input.parse()?);
        }
        Ok(Self { format, args })
    }
}

#[proc_macro]
pub fn scanf(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as ScanfArgs);
    let format_lit = &args.format;
    let format_str = format_lit.value();
    let explicit_args: Vec<_> = args.args.iter().collect();

    #[derive(Debug, Clone)]
    enum CTToken {
        Text(String),
        Placeholder(Placeholder),
    }
    let mut ct_tokens: Vec<CTToken> = Vec::new();
    let mut chars = format_str.chars().peekable();
    let mut current_text = String::new();
    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    current_text.push('{');
                    continue;
                }
                if !current_text.is_empty() {
                    ct_tokens.push(CTToken::Text(std::mem::take(&mut current_text)));
                }
                let mut content = String::new();
                for c2 in chars.by_ref() {
                    if c2 == '}' {
                        break;
                    }
                    content.push(c2);
                }
                if content.is_empty() {
                    ct_tokens.push(CTToken::Placeholder(Placeholder::Anonymous));
                } else if is_valid_identifier(&content) {
                    ct_tokens.push(CTToken::Placeholder(Placeholder::Named(content)));
                } else {
                    ct_tokens.push(CTToken::Placeholder(Placeholder::Anonymous));
                }
            }
            '}' => {
                if chars.peek() == Some(&'}') {
                    chars.next();
                    current_text.push('}');
                } else {
                    return syn::Error::new(format_lit.span(), "Unescaped '}' in format string")
                        .to_compile_error()
                        .into();
                }
            }
            other => current_text.push(other),
        }
    }
    if !current_text.is_empty() {
        ct_tokens.push(CTToken::Text(current_text));
    }

    let mut generated = Vec::new();
    let mut pending_placeholder: Option<Placeholder> = None;
    let mut anon_index: usize = 0;
    for token in &ct_tokens {
        match token {
            CTToken::Placeholder(ph) => {
                if pending_placeholder.is_some() {
                    return syn::Error::new(
                        format_lit.span(),
                        "Consecutive placeholders without separator are not supported",
                    )
                    .to_compile_error()
                    .into();
                }
                pending_placeholder = Some((*ph).clone());
            }
            CTToken::Text(text) => {
                let lit_text = LitStr::new(text, Span::call_site());
                if let Some(ph) = pending_placeholder.take() {
                    match ph {
                        Placeholder::Named(name) => {
                            let ident = Ident::new(&name, Span::call_site());
                            generated.push(quote! {
                                if let Some(pos) = __rest.find(#lit_text) {
                                    let slice = &__rest[..pos];
                                    match slice.parse() {
                                        Ok(parsed) => {
                                            #ident = parsed;
                                        }
                                        Err(error) => {
                                            __result = __result.and(Err(std::io::Error::new(
                                                std::io::ErrorKind::InvalidInput,
                                                error
                                            )));
                                        }
                                    }
                                    __rest = &__rest[pos + #lit_text.len()..];
                                } else {
                                    __result = __result.and(Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidInput,
                                        format!("Can not find text separator {:?}", #lit_text)
                                    )));
                                }
                            });
                        }
                        Placeholder::Anonymous => {
                            if anon_index >= explicit_args.len() {
                                return syn::Error::new(
                                    format_lit.span(),
                                    "Anonymous placeholder '{}' found but no corresponding argument provided"
                                )
                                .to_compile_error()
                                .into();
                            }
                            let arg_expr = explicit_args[anon_index];
                            anon_index += 1;
                            generated.push(quote! {
                                if let Some(pos) = __rest.find(#lit_text) {
                                    let slice = &__rest[..pos];
                                    match slice.parse() {
                                        Ok(parsed) => {
                                            *#arg_expr = parsed;
                                        }
                                        Err(error) => {
                                            __result = __result.and(Err(std::io::Error::new(
                                                std::io::ErrorKind::InvalidInput,
                                                error
                                            )));
                                        }
                                    }
                                    __rest = &__rest[pos + #lit_text.len()..];
                                } else {
                                    __result = __result.and(Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidInput,
                                        format!("Can not find text separator {:?}", #lit_text)
                                    )));
                                }
                            });
                        }
                    }
                } else {
                    generated.push(quote! {
                        if let Some(pos) = __rest.find(#lit_text) {
                            if pos == 0 {
                                __rest = &__rest[#lit_text.len()..];
                            } else {
                                __result = __result.and(Err(std::io::Error::new(
                                    std::io::ErrorKind::InvalidInput,
                                    format!("Expected text {:?} at current position", #lit_text)
                                )));
                            }
                        } else {
                            __result = __result.and(Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                format!("Can not find text separator {:?}", #lit_text)
                            )));
                        }
                    });
                }
            }
        }
    }
    if let Some(ph) = pending_placeholder.take() {
        match ph {
            Placeholder::Named(name) => {
                let ident = Ident::new(&name, Span::call_site());
                generated.push(quote! {
                    match __rest.parse() {
                        Ok(parsed) => {
                            #ident = parsed;
                        }
                        Err(error) => {
                            __result = __result.and(Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                error
                            )));
                        }
                    }
                    __rest = "";
                });
            }
            Placeholder::Anonymous => {
                if anon_index >= explicit_args.len() {
                    return syn::Error::new(
                        format_lit.span(),
                        "Anonymous placeholder '{}' found but no corresponding argument provided",
                    )
                    .to_compile_error()
                    .into();
                }
                let arg_expr = explicit_args[anon_index];
                anon_index += 1;
                generated.push(quote! {
                    match __rest.parse() {
                        Ok(parsed) => {
                            *#arg_expr = parsed;
                        }
                        Err(error) => {
                            __result = __result.and(Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                error
                            )));
                        }
                    }
                    __rest = "";
                });
            }
        }
    }
    if anon_index < explicit_args.len() {
        return syn::Error::new(
            explicit_args[anon_index].span(),
            "Too many arguments provided for format string",
        )
        .to_compile_error()
        .into();
    }

    let expanded = quote! {{
        let mut __result: std::io::Result<()> = Ok(());
        let mut __buffer = String::new();
        let _ = std::io::Write::flush(&mut std::io::stdout());
        match std::io::stdin().read_line(&mut __buffer) {
            Ok(_) => {
                let mut __rest: &str = __buffer.as_str();
                #(#generated)*
                __result
            }
            Err(e) => Err(e)
        }
    }};
    TokenStream::from(expanded)
}
