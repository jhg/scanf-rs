#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod codegen;
mod parsing;
mod tokenization;
mod types;
mod validation;

use proc_macro::TokenStream;
use syn::parse_macro_input;

use codegen::{Source, expand};
use parsing::{ScanfArgs, SourceArgs};

/// Parse a string with a format string, similar to C's `sscanf`.
///
/// Syntax: `sscanf!(input, "format")`
///
/// Placeholders: `{}` is returned, and `{name}` is written into the variable `name`. Types
/// must implement `FromStr`.
///
/// Returns `io::Result<T>` or `io::Result<(T1, T2, ..)>` with the values of the `{}`
/// placeholders, or `io::Result<()>` when the format has only named ones.
///
/// # Examples
///
/// ```
/// use scanf::sscanf;
///
/// // Returned values: the types come from the annotation
/// let age: u32 = sscanf!("age=30", "age={}").unwrap();
/// let (name, score): (String, f64) = sscanf!("Ana: 9.5", "{}: {}").unwrap();
/// assert_eq!(age, 30);
/// assert_eq!(name, "Ana");
/// assert_eq!(score, 9.5);
///
/// // Named placeholders: written into the variables of the same name
/// let mut x: i32 = 0;
/// let mut y: i32 = 0;
/// sscanf!("x=10, y=20", "x={x}, y={y}").unwrap();
/// assert_eq!(x, 10);
/// assert_eq!(y, 20);
///
/// // A failed parse is an `InvalidInput` error, and nothing is written to the variables.
/// let (mut a, mut b): (i32, i32) = (0, 0);
/// let error = sscanf!("7,x", "{a},{b}").unwrap_err();
/// assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
/// assert_eq!((a, b), (0, 0));
/// ```
#[proc_macro]
pub fn sscanf(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as SourceArgs);

    expand(&args.format, &args.args, &Source::Expression(&args.source))
}

/// Read one line from a reader and parse it with a format string, like `sscanf!` does with a
/// string.
///
/// Syntax: `fscanf!(reader, "format")`
///
/// The reader is any `BufRead` (a `Cursor`, a `&[u8]`, a `BufReader`, a locked stdin) and is
/// borrowed mutably, as the writer of `write!` is, so it is not moved and can be used again.
/// Each call consumes one line, removing its line ending (`\n`, `\r\n` or a lone `\r`), and
/// fails with `UnexpectedEof` when there is none left. If the line does not match the format
/// it is consumed anyway. Formats, placeholders and results work as in `sscanf!`.
///
/// # Examples
///
/// ```
/// use scanf::fscanf;
/// use std::io::Cursor;
///
/// let mut input = Cursor::new("Ana: 9.5\nBob: 7\n");
///
/// let (name, score): (String, f64) = fscanf!(input, "{}: {}").unwrap();
/// assert_eq!((name.as_str(), score), ("Ana", 9.5));
///
/// let mut name = String::new();
/// let mut score: f64 = 0.0;
/// fscanf!(input, "{name}: {score}").unwrap();
/// assert_eq!((name.as_str(), score), ("Bob", 7.0));
///
/// let end: std::io::Result<String> = fscanf!(input, "{}");
/// assert_eq!(end.unwrap_err().kind(), std::io::ErrorKind::UnexpectedEof);
/// ```
#[proc_macro]
pub fn fscanf(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as SourceArgs);
    expand(&args.format, &args.args, &Source::Reader(&args.source))
}

/// Read from stdin and parse with a format string, similar to C's `scanf`.
///
/// Syntax: `scanf!("format")`
///
/// Flushes stdout, reads a line and parses it (the line ending is removed). Fails with
/// `UnexpectedEof` when stdin has no more input.
///
/// It locks stdin for the duration of the call, so it must not be used while the same thread
/// holds a `stdin().lock()`: the lock is not reentrant and the call would wait forever. In that
/// situation read from the lock you hold with `fscanf!` instead. The format works as in `sscanf!`, and the
/// result is the same: the values of the `{}` placeholders, or `()` for named ones.
///
/// # Examples
///
/// ```no_run
/// use scanf::scanf;
///
/// // Returned values
/// print!("Enter your age: ");
/// let age: u32 = scanf!("{}").unwrap();
/// let (name, score): (String, f64) = scanf!("{}: {}").unwrap();
///
/// // Named placeholders
/// let mut city = String::new();
/// let mut population: u64 = 0;
/// scanf!("{city}: {population}").unwrap();
/// ```
#[proc_macro]
pub fn scanf(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as ScanfArgs);
    expand(&args.format, &args.args, &Source::Stdin)
}

/// Misuse that must not compile. Each case is a doctest, so it runs under `cargo test --doc`;
/// the wording of the messages is covered by the unit tests instead.
#[cfg(doctest)]
mod compile_failures {
    /// ```compile_fail
    /// use scanf::sscanf;
    ///
    /// sscanf!("5", "");
    /// ```
    fn empty_format() {}

    /// ```compile_fail
    /// use scanf::sscanf;
    ///
    /// let mut a: i32 = 0;
    /// let mut b: i32 = 0;
    /// sscanf!("12", "{}{}", &mut a, &mut b);
    /// ```
    fn consecutive_placeholders() {}

    /// ```compile_fail
    /// use scanf::sscanf;
    ///
    /// let mut a: i32 = 0;
    /// sscanf!("5", "a}b", &mut a);
    /// ```
    fn unescaped_closing_brace() {}

    /// ```compile_fail
    /// use scanf::sscanf;
    ///
    /// let mut a: i32 = 0;
    /// sscanf!("5", "{", &mut a);
    /// ```
    fn unclosed_placeholder() {}

    /// ```compile_fail
    /// use scanf::sscanf;
    ///
    /// sscanf!("5", "{1x}");
    /// ```
    fn identifier_starting_with_a_digit() {}

    /// ```compile_fail
    /// use scanf::sscanf;
    ///
    /// sscanf!("5", "{let}");
    /// ```
    fn keyword_as_identifier() {}

    /// ```compile_fail
    /// use scanf::sscanf;
    ///
    /// sscanf!("5", "{_}");
    /// ```
    fn lone_underscore_as_identifier() {}

    /// ```compile_fail
    /// use scanf::sscanf;
    ///
    /// sscanf!("5", "{x²}");
    /// ```
    fn identifier_with_a_non_identifier_character() {}

    /// ```compile_fail
    /// use scanf::sscanf;
    ///
    /// let mut a: i32 = 0;
    /// sscanf!("1,2", "{},{}", &mut a);
    /// ```
    fn too_few_arguments() {}

    /// ```compile_fail
    /// use scanf::sscanf;
    ///
    /// let mut a: i32 = 0;
    /// let mut b: i32 = 0;
    /// sscanf!("1", "{}", &mut a, &mut b);
    /// ```
    fn too_many_arguments() {}

    /// ```compile_fail
    /// use scanf::sscanf;
    ///
    /// let mut a: i32 = 0;
    /// let format = "{}";
    /// sscanf!("5", format, &mut a);
    /// ```
    fn format_that_is_not_a_literal() {}

    /// ```compile_fail
    /// use scanf::sscanf;
    ///
    /// let mut a: i32 = 0;
    /// sscanf!("5" "{}", &mut a);
    /// ```
    fn missing_comma_after_the_input() {}

    /// ```compile_fail,E0425
    /// use scanf::sscanf;
    ///
    /// sscanf!("5", "{missing}");
    /// ```
    fn undeclared_named_variable() {}

    /// ```compile_fail,E0614
    /// use scanf::sscanf;
    ///
    /// let a: i32 = 0;
    /// sscanf!("5", "{}", a);
    /// ```
    fn argument_that_cannot_be_dereferenced() {}
}
