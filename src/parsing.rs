//! Argument parsing for scanf macros.
//!
//! This module defines the structures used to parse macro arguments at compile-time.

use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    Expr, LitStr, Token,
};

// ============================================================================
// sscanf! Macro Arguments
// ============================================================================

/// Arguments for the `sscanf!` macro.
///
/// # Fields
///
/// - `input`: The string expression to parse
/// - `format`: The format string literal containing placeholders
/// - `args`: Optional explicit arguments for anonymous placeholders
pub struct SscanfArgs {
    pub input: Expr,
    pub format: LitStr,
    pub args: Punctuated<Expr, Comma>,
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

// ============================================================================
// scanf! Macro Arguments
// ============================================================================

/// Arguments for the `scanf!` macro.
///
/// # Fields
///
/// - `format`: The format string literal containing placeholders
/// - `args`: Optional explicit arguments for anonymous placeholders
pub struct ScanfArgs {
    pub format: LitStr,
    pub args: Punctuated<Expr, Comma>,
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
