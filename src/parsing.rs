//! Macro argument parsing structures.

use syn::{
    Expr, LitStr, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
};

/// sscanf! arguments: input, format, args
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

        let args = if input.is_empty() {
            Punctuated::new()
        } else {
            input.parse::<Token![,]>()?;
            Punctuated::parse_terminated(input)?
        };

        Ok(Self {
            input: input_expr,
            format,
            args,
        })
    }
}

/// scanf! arguments: format, args
pub struct ScanfArgs {
    pub format: LitStr,
    pub args: Punctuated<Expr, Comma>,
}

impl Parse for ScanfArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let format: LitStr = input.parse()?;

        let args = if input.is_empty() {
            Punctuated::new()
        } else {
            input.parse::<Token![,]>()?;
            Punctuated::parse_terminated(input)?
        };

        Ok(Self { format, args })
    }
}
