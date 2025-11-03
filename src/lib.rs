//! C-style scanf/sscanf procedural macros for text parsing.
//!
//! - `scanf!`: Read and parse from stdin
//! - `sscanf!`: Parse from string
//!
//! # Architecture
//!
//! Compile-time: `tokenization` → `codegen` → expansion
//! Runtime: Generated code parses input with `.find()` and `.parse()`
//!
//! Modules: `constants`, `types`, `validation`, `parsing`, `tokenization`, `codegen`
//!
//! # Hygiene
//!
//! Generated code uses isolated scopes `{{ }}` - no prefix pollution.
//!
//! # Limitations
//!
//! - Consecutive placeholders `{}{}` not allowed (ambiguous)
//! - Greedy parsing (no backtracking)
//! - Types must implement `FromStr`
//! - `scanf!` trims trailing newlines
//!
//! # Security
//!
//! **DoS limits:** 10K bytes format, 256 tokens, 128 char identifiers
//! **Memory:** `#![forbid(unsafe_code)]`, `Box<str>`, bounds-checked
//! **Validation:** Rejects empty formats, keywords, invalid identifiers

#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]
#![doc = include_str!("../README.md")]

mod codegen;
mod constants;
mod parsing;
mod tokenization;
mod types;
mod validation;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use codegen::generate_scanf_implementation;
use parsing::{ScanfArgs, SscanfArgs};

/// Parse a string with a format string, similar to C's `sscanf`.
///
/// Syntax: `sscanf!(input, "format", args...)`
///
/// Placeholders: `{name}` captures to variable, `{}` needs `&mut arg`
///
/// Returns `io::Result<()>`. Types must implement `FromStr`.
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
    let expanded = quote! {{
        let mut result: std::io::Result<()> = Ok(());
        let mut remaining = #input_expr;
        #(#generated)*
        result
    }};

    TokenStream::from(expanded)
}

/// Read from stdin and parse with a format string, similar to C's `scanf`.
///
/// Syntax: `scanf!("format", args...)`
///
/// Flushes stdout, reads line, parses (newline trimmed). Returns `io::Result<()>`.
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
    let expanded = quote! {{
        let mut result: std::io::Result<()> = Ok(());
        let mut buffer = String::new();
        let _ = std::io::Write::flush(&mut std::io::stdout());
        match std::io::stdin().read_line(&mut buffer) {
            Ok(_) => {
                let input = buffer.trim_end_matches('\n').trim_end_matches('\r');
                let mut remaining: &str = input;
                #(#generated)*
                result
            }
            Err(e) => Err(e)
        }
    }};
    TokenStream::from(expanded)
}
