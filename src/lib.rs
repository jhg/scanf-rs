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
///
/// // A failed parse is an `InvalidInput` error. Values parsed before the failure are
/// // already written, so check the result before using the variables.
/// let (mut a, mut b): (i32, i32) = (0, 0);
/// let error = sscanf!("7,x", "{a},{b}").unwrap_err();
/// assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
/// assert_eq!((a, b), (7, 0));
/// ```
#[proc_macro]
pub fn sscanf(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as SscanfArgs);

    expand(&args.format, &args.args, &Source::Expression(&args.input))
}

/// Read from stdin and parse with a format string, similar to C's `scanf`.
///
/// Syntax: `scanf!("format", args...)`
///
/// Flushes stdout, reads a line and parses it (the line ending is removed). Fails with
/// `UnexpectedEof` when stdin has no more input. Takes the same arguments as
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
