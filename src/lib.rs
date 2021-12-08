#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]

//! # scanf! & sscanf!
//!
//! Similar to C's but with memory safety.
//!
//! ## Examples
//!
//! ```no_run
//! use scanf::scanf;
//!
//! let mut product: String = String::new();
//! let mut price: f32 = 0.0;
//! println!("Insert product and price (product: price):");
//! if scanf!("{}: {}", product, price).is_ok() {
//!     println!("Price of {} is {:.2}", product, price);
//! }
//! ```
//!
//! ```
//! use scanf::sscanf;
//!
//! let input: &str = "Candy: 2.75";
//! let mut product: String = String::new();
//! let mut price: f32 = 0.0;
//! println!("Insert product and price (product: price):");
//! sscanf!(input, "{}: {}", product, price);
//! println!("Price of {} is {:.2}", product, price);
//! # assert_eq!(product, "Candy");
//! # assert_eq!(price, 2.75);
//! ```
//!
//! It's possible to indicate the type in the format string:
//!
//! ```no_run
//! # use scanf::scanf;
//! let mut product: String = String::new();
//! let mut price: f32 = 0.0;
//! println!("Insert product and price (product: price):");
//! scanf!("{string}: {f32}", product, price);
//! # println!("Price of {} is {:.2}", product, price);
//! ```
//!
//! Also escape brackets:
//!
//! ```
//! # use scanf::sscanf;
//! let input: &str = "{Candy}";
//! let mut product: String = String::new();
//! sscanf!(input, "{{{}}}", product);
//! assert_eq!(product, "Candy");
//! ```
//!
//! Examples has been compiled and `sscanf`'s examples also ran as tests.
//! If you have problems using the example code, please, [create an issue](https://github.com/jhg/scanf-rs/issues?q=is%3Aissue).

#[doc(hidden)]
pub mod format;

#[macro_export]
macro_rules! sscanf {
    ($input:expr, $format:literal, $($var:ident),+ ) => {{
        match $crate::format::InputFormat::new($format).and_then(|formatter| formatter.inputs($input)) {
            Ok(inputs) => {
                let mut inputs_iter = inputs.iter();
                let mut result = Ok(());
                $(
                    if let Some(input) = inputs_iter.next() {
                        if !input.is_required_type_of_var(&$var) {
                            result = result.and(Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Placeholder type does not match the variable type")));
                        } else {
                            match input.as_str().parse() {
                                Ok(input_parsed) => $var = input_parsed,
                                Err(error) => {
                                    let invalid_input_error = std::io::Error::new(std::io::ErrorKind::InvalidInput, error);
                                    result = result.and(Err(invalid_input_error));
                                }
                            }
                        }
                    } else {
                        result = result.and(Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "There is not enough input placeholders for all variables."
                        )));
                    }
                )*
                result
            }
            Err(error) => Err(error),
        }
    }};
    ($input:expr, $format:literal, $($var:ident),+ , ) => { sscanf!($input, $format, $($var),*) };
}

#[macro_export]
macro_rules! scanf {
    ($format:literal, $($var:ident),+ ) => {{
        let mut buffer = String::new();
        // In some use cases the output between scanf calls was not showed without this flush.
		let _ = std::io::Write::flush(&mut std::io::stdout());
        match std::io::stdin().read_line(&mut buffer) {
            Ok(_) => $crate::sscanf!(buffer.as_ref(), $format, $($var),*),
            Err(error) => Err(error),
        }
    }};
    ($format:literal, $($var:ident),+ , ) => { scanf!($format, $($var),*) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings() {
        let input = "Hello: world";
        let mut request: String = String::new();
        let mut reply: String = String::new();
        sscanf!(input, "{string}: {string}", request, reply).unwrap();
        assert_eq!(request, "Hello");
        assert_eq!(reply, "world");
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn string_and_float() {
        let input = "Candy->2.5";
        let mut product: String = String::new();
        let mut price: f64 = 0.0;
        sscanf!(input, "{string}->{f64}", product, price,).unwrap();
        assert_eq!(product, "Candy");
        assert_eq!(price, 2.5);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn generic() {
        let input = "5 -> 5.0";
        let mut request: i32 = 0;
        let mut reply: f32 = 0.0;
        sscanf!(input, "{} -> {}", request, reply).unwrap();
        assert_eq!(request, 5);
        assert_eq!(reply, 5.0);
    }

    #[test]
    fn string_between_brackets_ignored() {
        let input = "{Hello world}";
        let mut message: String = String::new();
        sscanf!(input, "{{{string}}}", message).unwrap();
        assert_eq!(message, "Hello world");
    }

    #[test]
    fn string_generic_between_brackets_ignored() {
        let input = "{Hello world}";
        let mut message: String = String::new();
        sscanf!(input, "{{{}}}", message).unwrap();
        assert_eq!(message, "Hello world");
    }

    #[test]
    #[should_panic]
    fn wrong_format_string() {
        let input = "5 -> 5.0 <-";
        let mut _request: i32 = 0;
        let mut _reply: f32 = 0.0;
        sscanf!(input, "{} -}> {} <-", _request, _reply).unwrap();
    }

    #[test]
    #[should_panic]
    fn wrong_format_two_generic_without_separator() {
        let input = "Hello";
        let mut _word1: String = String::new();
        let mut _word2: String = String::new();
        sscanf!(input, "{}{}", _word1, _word2).unwrap();
    }

    #[test]
    #[should_panic]
    fn wrong_format_placeholder_type() {
        let input = "5 -> 5.0";
        let mut _request: i32 = 0;
        let mut _reply: f32 = 0.0;
        sscanf!(input, "{u64} -> {f64}", _request, _reply).unwrap();
    }
}
