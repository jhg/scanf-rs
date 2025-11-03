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

/// scanf! arguments: format, args
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
