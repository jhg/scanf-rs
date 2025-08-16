use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Expr, Ident, LitStr, Token,
};

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

#[derive(Debug, PartialEq)]
enum Placeholder {
    Named(String),
    Anonymous,
}

fn parse_format_string(format_str: &str) -> Vec<Placeholder> {
    let mut placeholders = Vec::new();
    let mut chars = format_str.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                chars.next(); // skip escaped brace
                continue;
            }
            
            let mut content = String::new();
            while let Some(ch) = chars.next() {
                if ch == '}' {
                    break;
                }
                content.push(ch);
            }
            
            if content.is_empty() {
                placeholders.push(Placeholder::Anonymous);
            } else if is_valid_identifier(&content) {
                placeholders.push(Placeholder::Named(content));
            } else {
                // Invalid identifier, treat as anonymous
                placeholders.push(Placeholder::Anonymous);
            }
        } else if ch == '}' && chars.peek() == Some(&'}') {
            chars.next(); // skip escaped brace
        }
    }
    
    placeholders
}

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
    
    let placeholders = parse_format_string(&format_str);
    let explicit_args: Vec<_> = args.args.iter().collect();
    
    // Generate the parsing and assignment code
    let mut assignments = Vec::new();
    let mut arg_index = 0;
    
    for placeholder in placeholders {
        match placeholder {
            Placeholder::Named(var_name) => {
                // Generate code to assign to the named variable
                let var_ident = Ident::new(&var_name, Span::call_site());
                assignments.push(quote! {
                    if let Some(input_element) = inputs_iter.next() {
                        match input_element.as_str().parse() {
                            Ok(parsed) => #var_ident = parsed,
                            Err(error) => {
                                result = result.and(Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, error)));
                            }
                        }
                    } else {
                        result = result.and(Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Not enough input values for placeholders"
                        )));
                    }
                });
            }
            Placeholder::Anonymous => {
                // Use the provided argument
                if arg_index < explicit_args.len() {
                    let arg_expr = explicit_args[arg_index];
                    assignments.push(quote! {
                        if let Some(input_element) = inputs_iter.next() {
                            match input_element.as_str().parse() {
                                Ok(parsed) => *#arg_expr = parsed,
                                Err(error) => {
                                    result = result.and(Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, error)));
                                }
                            }
                        } else {
                            result = result.and(Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "Not enough input values for placeholders"
                            )));
                        }
                    });
                    arg_index += 1;
                } else {
                    return syn::Error::new(
                        format_lit.span(),
                        "Anonymous placeholder '{}' found but no corresponding argument provided"
                    ).to_compile_error().into();
                }
            }
        }
    }
    
    // Check if there are unused arguments
    if arg_index < explicit_args.len() {
        return syn::Error::new(
            explicit_args[arg_index].span(),
            "Too many arguments provided for format string"
        ).to_compile_error().into();
    }
    
    let expanded = quote! {
        {
            match scanf::format::InputFormatParser::new(#format_lit) {
                Ok(input_format_parser) => {
                    match input_format_parser.inputs(#input_expr) {
                        Ok(inputs) => {
                            let mut inputs_iter = inputs.iter();
                            let mut result = Ok(());
                            
                            #(#assignments)*
                            
                            result
                        }
                        Err(error) => Err(error),
                    }
                }
                Err(error) => Err(error),
            }
        }
    };
    
    TokenStream::from(expanded)
}