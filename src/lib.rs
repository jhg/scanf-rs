//! C-style scanf/sscanf procedural macros for text parsing.
//!
//! - `scanf!`: Read and parse from stdin
//! - `sscanf!`: Parse from string
//!
//! # Architecture
//!
//! Compile-time: `tokenization` → `codegen` → expansion
//! Runtime: Generated code uses `.find()` and `.parse()`
//!
//! Modules: `constants`, `types`, `validation`, `parsing`, `tokenization`, `codegen`
//!
//! # Hygiene
//!
//! Generated code uses isolated scopes - no variable pollution.
//!
//! # Limitations
//!
//! - Consecutive placeholders `{}{}` not allowed (ambiguous)
//! - `{}` arguments are all or nothing: one `&mut` per `{}`, or none and get the values back
//! - Greedy parsing (no backtracking)
//! - Types must implement `FromStr`
//! - `scanf!` trims trailing newlines
//! - Surrounding whitespace is kept unless the `trim-whitespace` feature is enabled (see README)
//!
//! # Security
//!
//! **`DoS` limits:** 10K bytes format, 256 tokens, 128 char identifiers
//! **Memory:** `#![forbid(unsafe_code)]`, `Box<str>`, bounds-checked
//! **Validation:** Rejects empty formats, keywords, invalid identifiers

#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod codegen;
mod constants;
mod parsing;
mod tokenization;
mod types;
mod validation;

use proc_macro::TokenStream;
use quote::quote_spanned;
use syn::parse_macro_input;

use codegen::{generate_scanf_implementation, internal_span};
use parsing::{ScanfArgs, SscanfArgs};

/// Parse a string with a format string, similar to C's `sscanf`.
///
/// Syntax: `sscanf!(input, "format", args...)`
///
/// Placeholders: `{name}` writes into the variable `name`, and `{}` either takes a
/// `&mut` argument or, with no arguments at all, is returned. Types must implement `FromStr`.
///
/// Returns `io::Result<()>`, or `io::Result<T>` / `io::Result<(T1, T2, ..)>` when the `{}`
/// placeholders have no arguments and the values come back instead.
///
/// # Examples
///
/// ```
/// use scanf::sscanf;
///
/// // Anonymous placeholders
/// let input = "42: hello";
/// let mut num: i32 = 0;
/// let mut text: String = String::new();
/// sscanf!(input, "{}: {}", &mut num, &mut text).unwrap();
/// assert_eq!(num, 42);
/// assert_eq!(text, "hello");
///
/// // Named placeholders
/// let input = "x=10, y=20";
/// let mut x: i32 = 0;
/// let mut y: i32 = 0;
/// sscanf!(input, "x={x}, y={y}").unwrap();
/// assert_eq!(x, 10);
/// assert_eq!(y, 20);
///
/// // No arguments: the values are returned, the types come from the annotation
/// let age: u32 = sscanf!("age=30", "age={}").unwrap();
/// let (name, score): (String, f64) = sscanf!("Ana: 9.5", "{}: {}").unwrap();
/// assert_eq!(age, 30);
/// assert_eq!(name, "Ana");
/// assert_eq!(score, 9.5);
/// ```
#[proc_macro]
pub fn sscanf(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as SscanfArgs);

    let input_expr = &args.input;
    let format_lit = &args.format;
    let explicit_args: Vec<_> = args.args.iter().collect();

    let generated = match generate_scanf_implementation(format_lit, &explicit_args) {
        Ok(code) => code,
        Err(err) => return err,
    };

    // Scope isolation ensures macro hygiene
    let span = internal_span();
    let declarations = generated.declarations();
    let statements = &generated.statements;
    let result = generated.result();
    let expanded = quote_spanned! {span=> {
        (|| -> std::io::Result<_> {
            #declarations
            let mut remaining: &str = #input_expr;
            #(#statements)*
            #result
        })()
    }};

    TokenStream::from(expanded)
}

/// Read from stdin and parse with a format string, similar to C's `scanf`.
///
/// Syntax: `scanf!("format", args...)`
///
/// Flushes stdout, reads a line and parses it (newline trimmed). Takes the same arguments as
/// `sscanf!`, and likewise returns `io::Result<()>` or, without arguments, the parsed values.
///
/// # Examples
///
/// ```no_run
/// use scanf::scanf;
///
/// // Read a number
/// let mut age: i32 = 0;
/// print!("Enter your age: ");
/// scanf!("{}", &mut age).unwrap();
///
/// // Named placeholders
/// let mut name: String = String::new();
/// let mut score: f64 = 0.0;
/// print!("Enter name and score: ");
/// scanf!("{name}: {score}").unwrap();
///
/// // Or let it return the value
/// let height: f64 = scanf!("{}").unwrap();
/// ```
#[proc_macro]
pub fn scanf(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as ScanfArgs);
    let format_lit = &args.format;
    let explicit_args: Vec<_> = args.args.iter().collect();

    let generated = match generate_scanf_implementation(format_lit, &explicit_args) {
        Ok(code) => code,
        Err(err) => return err,
    };

    // Scope isolation ensures macro hygiene
    let span = internal_span();
    let declarations = generated.declarations();
    let statements = &generated.statements;
    let result = generated.result();
    let expanded = quote_spanned! {span=> {
        (|| -> std::io::Result<_> {
            #declarations
            let mut buffer = String::new();
            let _ = std::io::Write::flush(&mut std::io::stdout());
            std::io::stdin().read_line(&mut buffer)?;
            let input = buffer.trim_end_matches('\n').trim_end_matches('\r');
            let mut remaining: &str = input;
            #(#statements)*
            #result
        })()
    }};
    TokenStream::from(expanded)
}
