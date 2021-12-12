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
        match $crate::format::InputFormatParser::new($format).and_then(|formatter| formatter.inputs($input)) {
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
    ($input:expr, $format:literal, $($var:ident),+ , ) => { $crate::sscanf!($input, $format, $($var),*) };
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
    ($format:literal, $($var:ident),+ , ) => { $crate::scanf!($format, $($var),*) };
}
