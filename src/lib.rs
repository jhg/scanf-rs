//! Procedural macros for C-style scanf/sscanf text parsing.
//!
//! This crate provides two main macros:
//! - `scanf!`: Reads and parses from stdin
//! - `sscanf!`: Parses from a string
//!
//! # Architecture
//!
//! The parsing process is divided into three phases:
//! 1. **Tokenization**: The format string is analyzed at compile-time to identify
//!    literal text and placeholders
//! 2. **Code Generation**: Rust code is generated that performs parsing at runtime
//! 3. **Expansion**: The macro expands to the generated code
//!
//! The implementation is organized into focused modules:
//! - `constants`: Security limits and memory allocation hints
//! - `types`: Core type definitions
//! - `validation`: Identifier validation
//! - `parsing`: Macro argument parsing
//! - `tokenization`: Format string tokenization
//! - `codegen`: Code generation
//!
//! # Name Hygiene
//!
//! The macros generate code within isolated scopes `{{ ... }}` to avoid
//! name collisions. Internal variables use descriptive names without
//! special prefixes, relying on scope isolation.
//!
//! # Known Limitations
//!
//! - **Consecutive placeholders**: Placeholders without separators (e.g., `{}{}`) are not allowed,
//!   as they would result in ambiguous parsing.
//! - **Greedy parsing**: Placeholders consume text until finding the next
//!   separator. Backtracking is not supported.
//! - **Required trait**: All types must implement `FromStr`.
//! - **Newlines in scanf!**: Line breaks at the end of input are automatically removed
//!   to facilitate parsing.
//!
//! # Performance
//!
//! The generated code is efficient:
//! - Zero-cost abstractions: no overhead vs manual parsing
//! - No additional allocations in generated code
//! - Errors detected at compile-time when possible
//! - Smart memory pre-allocation where appropriate
//!
//! # Security
//!
//! This crate implements multiple layers of protection:
//!
//! ## Compile-Time DoS Protection
//!
//! - **Format strings**: Maximum 10,000 bytes
//! - **Tokens**: Maximum 256 tokens per format string
//! - **Identifiers**: Maximum 128 characters
//!
//! These limits prevent denial-of-service attacks during compilation
//! while allowing all legitimate use cases.
//!
//! ## Memory Safety
//!
//! - `#![forbid(unsafe_code)]`: No unsafe code
//! - Use of `Box<str>` instead of `String` where appropriate
//! - No integer overflow: all indices are bounds-checked
//! - No hidden panics in generated code
//!
//! ## Generated Code
//!
//! The generated code uses only safe operations:
//! - `.find()` for text search (cannot panic)
//! - Slicing only after validating indices
//! - `.parse()` with explicit error handling
//! - Result types for error propagation
//!
//! ## Input Validation
//!
//! - Empty format strings rejected
//! - Invalid Rust identifiers rejected
//! - Rust keywords rejected in placeholders
//! - Unescaped braces detected at compile-time

#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]
#![doc = include_str!("../README.md")]

// ============================================================================
// Module Organization
// ============================================================================

mod codegen;
mod constants;
mod parsing;
mod tokenization;
mod types;
mod validation;

// ============================================================================
// Re-exports for Public API
// ============================================================================

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use codegen::generate_scanf_implementation;
use parsing::{ScanfArgs, SscanfArgs};

// ============================================================================
// Public Macros
// ============================================================================

/// Parses a string according to a format string, similar to C's `sscanf`.
///
/// # Syntax
///
/// ```ignore
/// sscanf!(input_expr, "format string", args...)
/// ```
///
/// - `input_expr`: Expression that evaluates to a `&str`
/// - `format string`: String literal with `{}` or `{name}` placeholders
/// - `args...`: Mutable references for anonymous `{}` placeholders
///
/// # Placeholders
///
/// - **Named**: `{variable}` - captures to a variable with that name in scope
/// - **Anonymous**: `{}` - requires an explicit `&mut var` argument
///
/// # Returns
///
/// Returns `std::io::Result<()>`:
/// - `Ok(())` if parsing was successful
/// - `Err(...)` if there was a parsing or format error
///
/// # Limitations
///
/// - Cannot have consecutive placeholders without separator (ambiguous)
/// - Types must implement `FromStr`
/// - Parsing is greedy: consumes until finding the next separator
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

    // Generate the parsing implementation
    let generated = match generate_scanf_implementation(format_lit, &explicit_args) {
        Ok(code) => code,
        Err(err) => return err,
    };

    // Hygiene: The double braces {{ }} create an isolated scope.
    // Variables `result` and `remaining` cannot collide with user code.
    // This is the idiomatic Rust way to ensure macro hygiene and avoid name collisions.
    let expanded = quote! {{
        let mut result: std::io::Result<()> = Ok(());
        let mut remaining = #input_expr;
        #(#generated)*
        result
    }};

    TokenStream::from(expanded)
}

/// Reads a line from stdin and parses it according to a format string, similar to C's `scanf`.
///
/// # Syntax
///
/// ```ignore
/// scanf!("format string", args...)
/// ```
///
/// - `format string`: String literal with `{}` or `{name}` placeholders
/// - `args...`: Mutable references for anonymous `{}` placeholders
///
/// # Behavior
///
/// 1. Flushes stdout (to display prompts if any)
/// 2. Reads a complete line from stdin (including newline)
/// 3. Parses the line according to the format string
///
/// # Returns
///
/// Returns `std::io::Result<()>`:
/// - `Ok(())` if reading and parsing were successful
/// - `Err(...)` if there was an I/O or parsing error
///
/// # Important Note
///
/// The newline at the end of the line is **not** included in the input to parse,
/// facilitating the parsing of simple lines.
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

    // Generate the parsing implementation
    let generated = match generate_scanf_implementation(format_lit, &explicit_args) {
        Ok(code) => code,
        Err(err) => return err,
    };

    // Hygiene: The double braces {{ }} create an isolated scope.
    // Variables `result`, `buffer`, `input`, and `remaining` cannot collide with user code.
    // This is the idiomatic Rust way to ensure macro hygiene.
    let expanded = quote! {{
        let mut result: std::io::Result<()> = Ok(());
        let mut buffer = String::new();
        let _ = std::io::Write::flush(&mut std::io::stdout());
        match std::io::stdin().read_line(&mut buffer) {
            Ok(_) => {
                // Trim trailing newline for consistent parsing
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
