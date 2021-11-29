#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]

//! # scanf! & sscanf!
//!
//! Similar to C's but without undefined behavior. **Currently it'll panic if an error occur**,
//! but to **return a `Result` is in progress**.
//!
//! ## Examples
//!
//! ```no_run
//! use scanf::scanf;
//!
//! let product: String;
//! let price: f32;
//! println!("Insert product and price (product: price):");
//! scanf!("{}: {}", product, price);
//! println!("Price of {} is {:.2}", product, price);
//! ```
//!
//! ```
//! use scanf::sscanf;
//!
//! let input: &str = "Candy: 2.75";
//! let product: String;
//! let price: f32;
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
//! let product: String;
//! let price: f32;
//! println!("Insert product and price (product: price):");
//! scanf!("{string}: {f32}", product, price);
//! # println!("Price of {} is {:.2}", product, price);
//! ```
//!
//! Examples has been compiled and `sscanf`'s examples also ran as tests.
//! If you have problems using the example code, please, [create an issue](https://github.com/jhg/scanf-rs/issues?q=is%3Aissue).

#[doc(hidden)]
pub mod format;

#[macro_export]
macro_rules! sscanf {
    ($input:tt, $format:literal, $($var:ident),+ ) => {
        let formatter = $crate::format::InputFormat::new($format);
        let inputs = formatter.input_strings($input);
        let mut inputs_iter = inputs.iter();
        $(
            $var = inputs_iter.next().unwrap().trim().parse().unwrap();
        )*
    };
    ($input:tt, $format:literal, $($var:ident),+ , ) => { sscanf!($input, $format, $($var),*) };
}

#[macro_export]
macro_rules! scanf {
    ($format:literal, $($var:ident),+ ) => {
        use std::io::Write;
        let mut buffer = String::new();
		std::io::stdout().flush(); // In some use cases the output between scanf calls was not showed without this flush.
        std::io::stdin().read_line(&mut buffer).unwrap();
        let input = buffer.as_ref();
        $crate::sscanf!(input, $format, $($var),*);
    };
    ($format:literal, $($var:ident),+ , ) => { scanf!($format, $($var),*) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings() {
        let input = "Hello: world";
        let request: String;
        let reply: String;
        sscanf!(input, "{string}: {}", request, reply);
        assert_eq!(request, "Hello");
        assert_eq!(reply, "world");
    }

    #[test]
    fn string_and_float() {
        let input = "Candy->2.5";
        let product: String;
        let price: f64;
        sscanf!(input, "{string}->{f64}", product, price,);
        assert_eq!(product, "Candy");
        assert_eq!(price, 2.5);
    }

    #[test]
    fn generic() {
        let input = "5 -> 5.0";
        let request: i32;
        let reply: f32;
        sscanf!(input, "{} -> {}", request, reply);
        assert_eq!(request, 5);
        assert_eq!(reply, 5.0);
    }

    #[test]
    #[should_panic]
    fn wrong_format_string() {
        let input = "5 -> 5.0";
        let request: i32;
        let reply: f32;
        sscanf!(input, "{} -{> {}", request, reply);
        assert_eq!(request, 5);
        assert_eq!(reply, 5.0);
    }
}
