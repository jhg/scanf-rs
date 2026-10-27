//! Code generation for scanf macros.

use crate::tokenization::{format_error, tokenize_format_string};
use crate::types::{FormatToken, Placeholder};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote, quote_spanned};
use syn::{Expr, Ident, LitChar, LitStr, punctuated::Punctuated, spanned::Spanned, token::Comma};

/// Span for the macro's own locals (`remaining`, `parsed`, ...).
///
/// Mixed-site hygiene keeps them apart from user variables, so a `{remaining}` placeholder
/// assigns the user's `remaining` and never the internal one.
fn internal_span() -> Span {
    Span::mixed_site()
}

/// Output of code generation.
struct Generated {
    /// One statement per format token, in order.
    statements: Vec<TokenStream2>,
    /// Values returned by the macro, one per anonymous placeholder. Empty unless the format
    /// has anonymous placeholders and no explicit arguments (return mode).
    outputs: Vec<Ident>,
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
            0 => quote! { ::core::result::Result::Ok(()) },
            1 => quote! { ::core::result::Result::Ok(#(#outputs)*) },
            _ => quote! { ::core::result::Result::Ok((#(#outputs),*)) },
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
        // The caller's own variable, so it keeps the call-site span instead of the hygienic one.
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

/// Decides where each placeholder's parsed value goes and builds its statement.
struct TargetResolver<'a> {
    explicit_args: &'a [&'a Expr],
    format_lit: &'a LitStr,
    /// Anonymous placeholders produce values instead of writing through explicit arguments.
    return_mode: bool,
    outputs: Vec<Ident>,
    anonymous_count: usize,
}

impl TargetResolver<'_> {
    /// Statement for `placeholder`: followed by `separator`, or, with `None`, the final
    /// placeholder that consumes the rest of the input.
    fn statement_for(
        &mut self,
        placeholder: &Placeholder,
        separator: Option<&LitStr>,
    ) -> syn::Result<TokenStream2> {
        let target = match placeholder {
            Placeholder::Named(name) => Target::named(name),
            Placeholder::Anonymous => self.anonymous_target(separator.is_none())?,
        };
        Ok(match separator {
            Some(separator) => separated_placeholder_statement(&target, separator),
            None => final_placeholder_statement(&target),
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
            quote! { *(#arg_expr) }
        };
        self.anonymous_count += 1;
        Ok(Target::anonymous(tokens, self.anonymous_count))
    }
}

/// Build one statement per token, pairing each placeholder with the text that follows it.
///
/// Also checks that every explicit argument was used by an anonymous placeholder.
fn statements_for_tokens(
    tokens: &[FormatToken],
    explicit_args: &[&Expr],
    format_lit: &LitStr,
    return_mode: bool,
) -> syn::Result<Generated> {
    let mut resolver = TargetResolver {
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
            FormatToken::Placeholder(placeholder) => pending_placeholder = Some(placeholder),
            FormatToken::Text(text) => {
                let text = LitStr::new(text, Span::call_site());
                statements.push(match pending_placeholder.take() {
                    Some(placeholder) => resolver.statement_for(placeholder, Some(&text))?,
                    None => fixed_text_statement(&text),
                });
            }
        }
    }

    if let Some(placeholder) = pending_placeholder {
        statements.push(resolver.statement_for(placeholder, None)?);
    }

    check_all_arguments_used(explicit_args, resolver.anonymous_count)?;
    Ok(Generated {
        statements,
        outputs: resolver.outputs,
    })
}

/// Expression parsing `source`. With the `trim-whitespace` feature, a failed parse is retried
/// with surrounding whitespace trimmed; without it, input is parsed strictly as-is.
///
/// The macro cannot know the target type, so the retry is what keeps `String` (which never
/// fails to parse) untouched while numbers and other strict types tolerate stray whitespace.
/// On double failure the error of the first, untrimmed attempt is reported.
fn parse_with_trim_fallback(source: &Ident) -> TokenStream2 {
    let span = internal_span();
    if !cfg!(feature = "trim-whitespace") {
        return quote_spanned! {span=> #source.parse() };
    }
    quote_spanned! {span=>
        #source.parse().or_else(|error| #source.trim().parse().map_err(|_| error))
    }
}

/// Runtime `io::Error` of kind `InvalidInput`, built from a message expression.
fn invalid_input(message: TokenStream2) -> TokenStream2 {
    let span = internal_span();
    quote_spanned! {span=>
        ::std::io::Error::new(::std::io::ErrorKind::InvalidInput, #message)
    }
}

/// Statements that parse the text in the variable `source` and assign it to `target`.
///
/// `source_wording` goes between "from" and the quoted text in the error message.
fn parse_and_assign(target: &Target, source: &Ident, source_wording: &str) -> TokenStream2 {
    let Target {
        tokens,
        description,
    } = target;
    let span = internal_span();
    let parse_expr = parse_with_trim_fallback(source);
    let message = format!("Failed to parse {{}} from {source_wording}{{:?}}: {{}}");
    let error = invalid_input(quote_spanned! {span=>
        ::std::format!(#message, #description, #source, error)
    });
    quote_spanned! {span=>
        let parsed = #parse_expr.map_err(|error| #error)?;
        #tokens = parsed;
    }
}

/// Placeholder followed by a separator: parses what comes before the first occurrence of it.
fn separated_placeholder_statement(target: &Target, separator: &LitStr) -> TokenStream2 {
    let span = internal_span();
    let slice = Ident::new("slice", span);
    let description = &target.description;
    let not_found = invalid_input(quote_spanned! {span=>
        ::std::format!(
            "Expected separator {:?} for {} not found in remaining input: {:?}",
            #separator,
            #description,
            remaining
        )
    });
    let parse_and_assign = parse_and_assign(target, &slice, "");
    // A char pattern lets `split_once` use the cheaper single-character search.
    let pattern = match single_char(separator) {
        Some(ch) => LitChar::new(ch, separator.span()).into_token_stream(),
        None => separator.into_token_stream(),
    };
    quote_spanned! {span=>
        {
            let (slice, rest) = remaining.split_once(#pattern).ok_or_else(|| #not_found)?;
            #parse_and_assign
            remaining = rest;
        }
    }
}

/// The only character of `literal`, if it has exactly one.
fn single_char(literal: &LitStr) -> Option<char> {
    let value = literal.value();
    let mut chars = value.chars();
    let first = chars.next()?;
    chars.next().is_none().then_some(first)
}

/// Literal text that must appear at the current position.
fn fixed_text_statement(text: &LitStr) -> TokenStream2 {
    let span = internal_span();
    let mismatch = invalid_input(quote_spanned! {span=>
        ::std::format!(
            "Expected text {:?} at current position. Remaining input: {:?}",
            #text,
            remaining
        )
    });
    quote_spanned! {span=>
        {
            let ::core::option::Option::Some(rest) = remaining.strip_prefix(#text) else {
                return ::core::result::Result::Err(#mismatch);
            };
            remaining = rest;
        }
    }
}

/// Last placeholder: parses everything that is left.
fn final_placeholder_statement(target: &Target) -> TokenStream2 {
    let span = internal_span();
    let remaining = Ident::new("remaining", span);
    let parse_and_assign = parse_and_assign(target, &remaining, "remaining input ");
    quote_spanned! {span=>
        {
            #parse_and_assign
            remaining = "";
        }
    }
}

fn missing_argument_error(position: usize, is_final: bool, format_lit: &LitStr) -> syn::Error {
    let prefix = if is_final { "Final " } else { "" };
    format_error(
        format_lit,
        format!(
            "{prefix}anonymous placeholder '{{}}' at position {position} has no corresponding argument. \
             Pass no arguments to get the values returned, use a named placeholder (e.g., '{{var}}'), or provide a mutable reference for every '{{}}' (e.g., &mut var)"
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

/// Tokenize the format string and generate the parsing statements.
///
/// Anonymous placeholders without any explicit argument select return mode.
fn generate(format_lit: &LitStr, explicit_args: &[&Expr]) -> syn::Result<Generated> {
    let tokens = tokenize_format_string(&format_lit.value(), format_lit)?;

    let has_anonymous = tokens
        .iter()
        .any(|token| matches!(token, FormatToken::Placeholder(Placeholder::Anonymous)));
    let return_mode = explicit_args.is_empty() && has_anonymous;

    statements_for_tokens(&tokens, explicit_args, format_lit, return_mode)
}

/// Where the text to parse comes from.
pub enum Source<'a> {
    /// `sscanf!`: the value of this expression, evaluated once.
    Expression(&'a Expr),
    /// `scanf!`: one line from stdin.
    Stdin,
}

impl Source<'_> {
    /// Code that defines `remaining: &str` and then runs `body` with it in scope.
    fn bind_remaining(&self, body: TokenStream2) -> TokenStream2 {
        let span = internal_span();
        match self {
            // The match keeps temporaries in the input expression alive for the whole body,
            // so `sscanf!(line().trim(), ..)` borrows from a value that is still around.
            Source::Expression(expr) => quote_spanned! {span=>
                match &(#expr) {
                    input => {
                        let mut remaining: &str = input;
                        #body
                    }
                }
            },
            Source::Stdin => quote_spanned! {span=>
                let mut buffer = ::std::string::String::new();
                let _ = ::std::io::Write::flush(&mut ::std::io::stdout());
                let bytes_read = ::std::io::stdin().read_line(&mut buffer)?;
                if bytes_read == 0 {
                    return ::core::result::Result::Err(::std::io::Error::new(
                        ::std::io::ErrorKind::UnexpectedEof,
                        "no input available on stdin",
                    ));
                }
                // One line ending is dropped: `\n`, `\r\n` or a lone `\r`. Any other `\r` stays.
                let line = buffer.strip_suffix('\n').unwrap_or(&buffer);
                let mut remaining: &str = line.strip_suffix('\r').unwrap_or(line);
                #body
            },
        }
    }
}

/// Expand a scanf-style macro: tokenize the format, pick return or write mode, generate the
/// statements and check the arguments, then wrap it all in a closure returning `io::Result`.
/// This is the single point where generation errors become compile errors.
pub fn expand(
    format_lit: &LitStr,
    args: &Punctuated<Expr, Comma>,
    source: &Source,
) -> proc_macro::TokenStream {
    let explicit_args: Vec<_> = args.iter().collect();
    let generated = match generate(format_lit, &explicit_args) {
        Ok(generated) => generated,
        Err(error) => return error.to_compile_error().into(),
    };

    let span = internal_span();
    let declarations = generated.declarations();
    let statements = &generated.statements;
    let result = generated.result();
    let body = source.bind_remaining(quote_spanned! {span=>
        #(#statements)*
        #result
    });
    quote_spanned! {span=> {
        (|| -> ::std::io::Result<_> {
            #declarations
            #body
        })()
    }}
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn statements(format: &str, args: &[Expr]) -> syn::Result<Generated> {
        let format_lit = LitStr::new(format, Span::call_site());
        let args: Vec<&Expr> = args.iter().collect();
        generate(&format_lit, &args)
    }

    #[test]
    fn only_one_character_separators_use_a_char_pattern() {
        let lit = |text: &str| LitStr::new(text, Span::call_site());
        assert_eq!(single_char(&lit(",")), Some(','));
        assert_eq!(single_char(&lit("→")), Some('→'));
        assert_eq!(single_char(&lit("")), None);
        assert_eq!(single_char(&lit("::")), None);
    }

    fn error_of(format: &str, args: &[Expr]) -> String {
        match statements(format, args) {
            Ok(_) => panic!("format {format:?} should be rejected"),
            Err(error) => error.to_string(),
        }
    }

    fn arg(source: &str) -> Expr {
        syn::parse_str(source).unwrap()
    }

    #[test]
    fn anonymous_placeholders_without_arguments_return_their_values() {
        let generated = statements("{}:{}", &[]).unwrap();
        assert_eq!(generated.outputs.len(), 2);
        assert_eq!(generated.statements.len(), 2);
    }

    #[test]
    fn explicit_arguments_produce_no_outputs() {
        let generated = statements("{}:{}", &[arg("&mut a"), arg("&mut b")]).unwrap();
        assert!(generated.outputs.is_empty());
        assert_eq!(generated.statements.len(), 2);
    }

    #[test]
    fn named_placeholders_produce_no_outputs() {
        let generated = statements("{x}", &[]).unwrap();
        assert!(generated.outputs.is_empty());
        assert_eq!(generated.statements.len(), 1);
    }

    #[test]
    fn named_placeholders_do_not_enable_return_mode() {
        let generated = statements("{a}:{b}", &[]).unwrap();
        assert!(generated.outputs.is_empty());
    }

    #[test]
    fn fixed_text_alone_is_one_statement() {
        let generated = statements("exact", &[]).unwrap();
        assert_eq!(generated.statements.len(), 1);
        assert!(generated.outputs.is_empty());
    }

    #[test]
    fn a_missing_argument_names_its_position() {
        let message = error_of("{}:{}", &[arg("&mut a")]);
        assert!(message.contains("position 2"), "{message}");
        assert!(message.contains("Final"), "{message}");
    }

    #[test]
    fn an_argument_missing_in_the_middle_is_reported_without_final() {
        let message = error_of("{}:{}:{}", &[arg("&mut a")]);
        assert!(message.starts_with("anonymous placeholder"), "{message}");
        assert!(message.contains("position 2"), "{message}");
    }

    #[test]
    fn surplus_arguments_are_counted() {
        let message = error_of("{}:{}", &[arg("&mut a"), arg("&mut b"), arg("&mut c")]);
        assert!(message.contains("Too many arguments: 1"), "{message}");
    }

    #[test]
    fn arguments_without_anonymous_placeholders_are_surplus() {
        let message = error_of("{x}", &[arg("&mut a")]);
        assert!(message.contains("Too many arguments: 1"), "{message}");
    }

    #[test]
    fn empty_format_is_rejected() {
        assert!(error_of("", &[]).contains("cannot be empty"));
    }

    #[test]
    fn consecutive_placeholders_are_rejected() {
        for format in ["{}{}", "{a}{b}", "{}{a}", "{a}{}"] {
            assert!(
                error_of(format, &[]).contains("Consecutive placeholders"),
                "format: {format:?}"
            );
        }
    }

    #[test]
    fn escaped_braces_between_placeholders_separate_them() {
        assert!(statements("{}{{}}{}", &[]).is_ok());
    }
}
