//! Macro argument parsing structures.

use syn::{
    Expr, LitStr, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
};

/// Arguments after the format string: nothing, or `, a, b, ...` with an optional trailing comma.
fn parse_optional_args(input: ParseStream) -> syn::Result<Punctuated<Expr, Comma>> {
    if input.is_empty() {
        return Ok(Punctuated::new());
    }
    input.parse::<Token![,]>()?;
    Punctuated::parse_terminated(input)
}

/// Arguments of the macros that take a source first: `sscanf!` (a string) and `fscanf!`
/// (a reader).
pub struct SourceArgs {
    pub source: Expr,
    pub format: LitStr,
    pub args: Punctuated<Expr, Comma>,
}

impl Parse for SourceArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let source = input.parse()?;
        input.parse::<Token![,]>()?;
        let format = input.parse()?;

        let args = parse_optional_args(input)?;

        Ok(Self {
            source,
            format,
            args,
        })
    }
}

pub struct ScanfArgs {
    pub format: LitStr,
    pub args: Punctuated<Expr, Comma>,
}

impl Parse for ScanfArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let format: LitStr = input.parse()?;

        let args = parse_optional_args(input)?;

        Ok(Self { format, args })
    }
}
