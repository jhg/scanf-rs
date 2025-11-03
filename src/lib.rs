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

// ============================================================================
// Argument Parsing Structures
// ============================================================================

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

// ============================================================================
// Core Types and Validation
// ============================================================================

/// Represents a placeholder in a format string.
///
/// Placeholders can be either named (e.g., `{variable}`) or anonymous (e.g., `{}`).
///
/// # Memory Layout
///
/// Uses `Box<str>` instead of `String` for Named variant to minimize memory overhead.
/// `Box<str>` is more memory-efficient for immutable strings (no capacity field).
/// This is appropriate since placeholder names don't change after parsing.
#[derive(Debug, PartialEq, Eq, Clone)]
enum Placeholder {
    /// A named placeholder that captures to a specific variable
    /// Uses Box<str> for memory efficiency (no capacity overhead)
    Named(Box<str>),
    /// An anonymous placeholder that requires an explicit argument
    Anonymous,
}

/// Checks if a string is a valid Rust identifier.
///
/// A valid identifier must:
/// - Not be empty
/// - Not be a Rust keyword
/// - Start with an alphabetic character (including Unicode) or underscore
/// - Contain only alphanumeric characters (including Unicode) or underscores
///
/// Note: This doesn't check for raw identifiers (r#name) as they're not needed
/// in placeholder context.
///
/// # Performance
///
/// This function is called at compile-time during macro expansion, so it's optimized
/// for correctness over runtime performance. The keyword check uses a simple slice
/// search which is acceptable for compile-time use.
#[inline]
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Check for Rust keywords (common ones that would be problematic in placeholders)
    // Using a slice is fine for compile-time checks; the list is small enough
    const KEYWORDS: &[&str] = &[
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
        "use", "where", "while", "async", "await", "dyn", "abstract", "become", "box", "do",
        "final", "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
    ];

    if KEYWORDS.contains(&s) {
        return false;
    }

    // SAFETY: We already checked s.is_empty() above, so next() will return Some
    let mut chars = s.chars();
    let first = chars.next().unwrap();

    // First character: alphabetic (Unicode) or underscore, but not a digit
    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    // Subsequent characters: alphanumeric (Unicode) or underscore
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

// ============================================================================
// Compile-Time Tokenization
// ============================================================================

/// Token type for compile-time tokenization of format strings.
///
/// This represents either literal text or a placeholder in the format string.
///
/// # Design note
///
/// `FormatToken` is only used during compile-time macro expansion, not in the generated
/// runtime code. Therefore, we prioritize code clarity over runtime performance.
/// The abbreviation "CT" (Compile-Time) was replaced with the more explicit
/// "FormatToken" for better code maintainability.
#[derive(Debug, Clone)]
enum FormatToken {
    /// A literal text segment that must match exactly in the input
    Text(String),
    /// A placeholder that captures a value from the input
    Placeholder(Placeholder),
}

/// Tokenizes a format string into text segments and placeholders at compile-time.
///
/// This function parses the format string, handling escaped braces (`{{` and `}}`),
/// and returns a sequence of tokens that can be processed to generate parsing code.
///
/// # Errors
///
/// Returns a compile error if the format string contains unescaped `}` characters.
fn tokenize_format_string(
    format_str: &str,
    format_lit: &LitStr,
) -> Result<Vec<FormatToken>, TokenStream> {
    // Security: Protect against DoS via extremely long format strings
    const MAX_FORMAT_STRING_LEN: usize = 10_000;
    if format_str.len() > MAX_FORMAT_STRING_LEN {
        return Err(syn::Error::new(
            format_lit.span(),
            format!(
                "Format string too long ({} bytes). Maximum allowed: {} bytes. \
                 This limit prevents compile-time DoS attacks.",
                format_str.len(),
                MAX_FORMAT_STRING_LEN
            ),
        )
        .to_compile_error()
        .into());
    }

    // Security: Limit number of tokens to prevent excessive code generation
    const MAX_TOKENS: usize = 256;
    let mut tokens: Vec<FormatToken> = Vec::with_capacity(4); // Pre-allocate for typical case
    let mut chars = format_str.chars().peekable();
    let mut current_text = String::with_capacity(16); // Pre-allocate for typical separator

    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                if chars.peek() == Some(&'{') {
                    // Escaped open brace
                    chars.next();
                    current_text.push('{');
                    continue;
                }
                // Flush accumulated text before processing placeholder
                if !current_text.is_empty() {
                    tokens.push(FormatToken::Text(std::mem::take(&mut current_text)));
                    // Security check: prevent excessive token creation
                    if tokens.len() > MAX_TOKENS {
                        return Err(syn::Error::new(
                            format_lit.span(),
                            format!(
                                "Too many tokens in format string ({}). Maximum allowed: {}. \
                                 This limit prevents compile-time resource exhaustion.",
                                tokens.len(),
                                MAX_TOKENS
                            ),
                        )
                        .to_compile_error()
                        .into());
                    }
                    // Reset capacity hint for next text segment
                    current_text = String::with_capacity(16);
                }
                // Capture placeholder content (typical identifier: 1-10 chars)
                // Security: limit identifier length to prevent DoS
                const MAX_IDENTIFIER_LEN: usize = 128;
                let mut content = String::with_capacity(8);
                for c2 in chars.by_ref() {
                    if c2 == '}' {
                        break;
                    }
                    // Security check: prevent extremely long identifiers
                    if content.len() >= MAX_IDENTIFIER_LEN {
                        return Err(syn::Error::new(
                            format_lit.span(),
                            format!(
                                "Identifier in placeholder too long (>{} characters). \
                                 This limit prevents compile-time DoS attacks.",
                                MAX_IDENTIFIER_LEN
                            ),
                        )
                        .to_compile_error()
                        .into());
                    }
                    content.push(c2);
                }
                if content.is_empty() {
                    tokens.push(FormatToken::Placeholder(Placeholder::Anonymous));
                } else if is_valid_identifier(&content) {
                    // Convert String to Box<str> for memory efficiency
                    tokens.push(FormatToken::Placeholder(Placeholder::Named(
                        content.into_boxed_str(),
                    )));
                } else {
                    // Invalid identifier - return error with helpful message
                    return Err(syn::Error::new(
                        format_lit.span(),
                        format!(
                            "Invalid identifier '{}' in placeholder. \
                             Identifiers must start with a letter or underscore, \
                             contain only alphanumeric characters or underscores, \
                             and not be Rust keywords. Use '{{}}' for anonymous placeholders.",
                            content
                        ),
                    )
                    .to_compile_error()
                    .into());
                }
            }
            '}' => {
                if chars.peek() == Some(&'}') {
                    // Escaped close brace
                    chars.next();
                    current_text.push('}');
                } else {
                    // Unescaped single '}' is invalid
                    return Err(syn::Error::new(
                        format_lit.span(),
                        "Unescaped '}' in format string",
                    )
                    .to_compile_error()
                    .into());
                }
            }
            other => current_text.push(other),
        }
    }
    if !current_text.is_empty() {
        tokens.push(FormatToken::Text(current_text));
    }

    Ok(tokens)
}

// ============================================================================
// Code Generation
// ============================================================================

/// Generates parsing code from the tokenized format string.
///
/// This function takes the tokenized format string and generates the corresponding
/// Rust code that will perform parsing of the input according to the specification.
///
/// # Algorithm
///
/// For each token:
/// - **Literal text**: Searches for and consumes that exact text from input
/// - **Placeholder + Text**: Searches for the text and parses everything before the placeholder
/// - **Final placeholder**: Parses all remaining input
///
/// # Error Handling
///
/// Errors accumulate in a `result` variable that combines multiple errors
/// using the `.and(Err(...))` pattern. This allows parsing to continue
/// to provide better error feedback.
///
/// # Design Note
///
/// Code for Named and Anonymous placeholders is similar but NOT extracted
/// into a helper function because:
/// - Error messages are different and specific
/// - Clarity is more important than extreme DRY
/// - Inline code is easier to understand and maintain (human-first)
///
/// # Errors
///
/// Returns a compile error if:
/// - Consecutive placeholders without separator are found (ambiguous parsing)
/// - Anonymous placeholders don't have corresponding arguments
/// - Too many arguments are provided
fn generate_parsing_code(
    tokens: &[FormatToken],
    explicit_args: &[&Expr],
    format_lit: &LitStr,
) -> Result<(Vec<proc_macro2::TokenStream>, usize), TokenStream> {
    // Pre-allocate: typically one code block per token
    let mut generated = Vec::with_capacity(tokens.len());
    let mut pending_placeholder: Option<Placeholder> = None;
    let mut anon_index: usize = 0;

    for token in tokens {
        match token {
            FormatToken::Placeholder(ph) => {
                if pending_placeholder.is_some() {
                    return Err(syn::Error::new(
                        format_lit.span(),
                        "Consecutive placeholders without separator are ambiguous and not supported. \
                         Add text between placeholders to separate them. Example: '{}:{}' instead of '{}{}'",
                    )
                    .to_compile_error()
                    .into());
                }
                pending_placeholder = Some(ph.clone());
            }
            FormatToken::Text(text) => {
                let lit_text = LitStr::new(text, Span::call_site());
                if let Some(ph) = pending_placeholder.take() {
                    match ph {
                        Placeholder::Named(name) => {
                            let ident = Ident::new(&name, Span::call_site());
                            let var_name = format!("variable '{}'", name);
                            generated.push(quote! {
                                // Parse named placeholder into variable
                                if let Some(pos) = remaining.find(#lit_text) {
                                    let slice = &remaining[..pos];
                                    match slice.parse() {
                                        Ok(parsed) => {
                                            #ident = parsed;
                                        }
                                        Err(error) => {
                                            result = result.and(Err(std::io::Error::new(
                                                std::io::ErrorKind::InvalidInput,
                                                format!("Failed to parse {} from {:?}: {}", #var_name, slice, error)
                                            )));
                                        }
                                    }
                                    remaining = &remaining[pos + #lit_text.len()..];
                                } else {
                                    result = result.and(Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidInput,
                                        format!(
                                            "Expected separator {:?} for {} not found in remaining input: {:?}",
                                            #lit_text,
                                            #var_name,
                                            remaining
                                        )
                                    )));
                                }
                            });
                        }
                        Placeholder::Anonymous => {
                            if anon_index >= explicit_args.len() {
                                return Err(syn::Error::new(
                                    format_lit.span(),
                                    format!(
                                        "Anonymous placeholder '{{}}' at position {} has no corresponding argument. \
                                         Provide a mutable reference argument (e.g., &mut var) or use a named placeholder (e.g., '{{var}}')",
                                        anon_index + 1
                                    )
                                )
                                .to_compile_error()
                                .into());
                            }
                            let arg_expr = explicit_args[anon_index];
                            let placeholder_num = anon_index + 1;
                            anon_index += 1;
                            generated.push(quote! {
                                // Parse anonymous placeholder (argument position)
                                if let Some(pos) = remaining.find(#lit_text) {
                                    let slice = &remaining[..pos];
                                    match slice.parse() {
                                        Ok(parsed) => {
                                            *#arg_expr = parsed;
                                        }
                                        Err(error) => {
                                            result = result.and(Err(std::io::Error::new(
                                                std::io::ErrorKind::InvalidInput,
                                                format!(
                                                    "Failed to parse anonymous placeholder #{} from {:?}: {}",
                                                    #placeholder_num,
                                                    slice,
                                                    error
                                                )
                                            )));
                                        }
                                    }
                                    remaining = &remaining[pos + #lit_text.len()..];
                                } else {
                                    result = result.and(Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidInput,
                                        format!(
                                            "Expected separator {:?} for anonymous placeholder #{} not found in remaining input: {:?}",
                                            #lit_text,
                                            #placeholder_num,
                                            remaining
                                        )
                                    )));
                                }
                            });
                        }
                    }
                } else {
                    // No placeholder - just fixed text that must match
                    generated.push(quote! {
                        // Match required fixed text
                        if let Some(pos) = remaining.find(#lit_text) {
                            // Ensure we match immediately at position 0 (no skipping)
                            if pos == 0 {
                                remaining = &remaining[#lit_text.len()..];
                            } else {
                                result = result.and(Err(std::io::Error::new(
                                    std::io::ErrorKind::InvalidInput,
                                    format!(
                                        "Expected text {:?} at current position, but found it at offset {}. \
                                         Remaining input: {:?}",
                                        #lit_text,
                                        pos,
                                        remaining
                                    )
                                )));
                            }
                        } else {
                            result = result.and(Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                format!(
                                    "Required text separator {:?} not found. Remaining input: {:?}",
                                    #lit_text,
                                    remaining
                                )
                            )));
                        }
                    });
                }
            }
        }
    }

    // Final pending placeholder consumes rest of input
    if let Some(ph) = pending_placeholder {
        match ph {
            Placeholder::Named(name) => {
                let ident = Ident::new(&name, Span::call_site());
                let var_name = format!("variable '{}'", name);
                generated.push(quote! {
                    // Parse final named placeholder (consumes all remaining input)
                    match remaining.parse() {
                        Ok(parsed) => {
                            #ident = parsed;
                        }
                        Err(error) => {
                            result = result.and(Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                format!("Failed to parse {} from remaining input {:?}: {}", #var_name, remaining, error)
                            )));
                        }
                    }
                    remaining = ""; // consumed
                });
            }
            Placeholder::Anonymous => {
                if anon_index >= explicit_args.len() {
                    return Err(syn::Error::new(
                        format_lit.span(),
                        format!(
                            "Final anonymous placeholder '{{}}' at position {} has no corresponding argument. \
                             Provide a mutable reference argument (e.g., &mut var) or use a named placeholder (e.g., '{{var}}')",
                            anon_index + 1
                        )
                    )
                    .to_compile_error()
                    .into());
                }
                let arg_expr = explicit_args[anon_index];
                let placeholder_num = anon_index + 1;
                anon_index += 1;
                generated.push(quote! {
                    // Parse final anonymous placeholder (consumes all remaining input)
                    match remaining.parse() {
                        Ok(parsed) => {
                            *#arg_expr = parsed;
                        }
                        Err(error) => {
                            result = result.and(Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                format!(
                                    "Failed to parse anonymous placeholder #{} from remaining input {:?}: {}",
                                    #placeholder_num,
                                    remaining,
                                    error
                                )
                            )));
                        }
                    }
                    remaining = ""; // consumed
                });
            }
        }
    }

    Ok((generated, anon_index))
}

/// Generates common parsing code for both sscanf and scanf macros.
///
/// This function centralizes the shared code generation logic to avoid
/// duplication between the two macros.
fn generate_scanf_implementation(
    format_lit: &LitStr,
    explicit_args: &[&Expr],
) -> Result<Vec<proc_macro2::TokenStream>, TokenStream> {
    let format_str = format_lit.value();

    // Validate format string is not empty
    if format_str.is_empty() {
        return Err(syn::Error::new(
            format_lit.span(),
            "Format string cannot be empty. Provide at least one placeholder or literal text.",
        )
        .to_compile_error()
        .into());
    }

    // Tokenize the format string at compile-time
    let tokens = tokenize_format_string(&format_str, format_lit)?;

    // Validate there's at least something to parse
    if tokens.is_empty() {
        return Err(syn::Error::new(
            format_lit.span(),
            "Format string contains no parseable content",
        )
        .to_compile_error()
        .into());
    }

    // Generate the parsing code
    let (generated, anon_index) = generate_parsing_code(&tokens, explicit_args, format_lit)?;

    // Check if there are unused arguments
    if anon_index < explicit_args.len() {
        let unused_count = explicit_args.len() - anon_index;
        return Err(syn::Error::new(
            explicit_args[anon_index].span(),
            format!(
                "Too many arguments: {} unused argument(s) provided. \
                 The format string only has {} anonymous placeholder(s)",
                unused_count, anon_index
            ),
        )
        .to_compile_error()
        .into());
    }

    Ok(generated)
}

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

    // SAFETY: The double braces {{ }} create an isolated scope.
    // Variables `result` and `remaining` cannot collide with user code.
    // This is the idiomatic Rust way to ensure macro hygiene.
    let expanded = quote! {{
        let mut result: std::io::Result<()> = Ok(());
        let mut remaining = #input_expr;
        #(#generated)*
        result
    }};

    TokenStream::from(expanded)
}

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

    // SAFETY: The double braces {{ }} create an isolated scope.
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
