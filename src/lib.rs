#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]
#![doc = include_str!("../README.md")]

#[doc(hidden)]
pub mod error;
#[doc(hidden)]
pub mod format;

#[macro_export]
macro_rules! sscanf {
    ($input:expr, $format:literal, $($var:expr),+ ) => {{
        match $crate::format::InputFormatParser::new($format) {
            Ok(input_format_parser) => {
                $crate::sscanf!($input, input_format_parser, $($var),+)
            }
            Err(error) => Err(error),
        }
    }};
    ($input:expr, $format:literal, $($var:expr),+ , ) => { $crate::sscanf!($input, $format, $($var),*) };
    ($input:expr, $formatter:ident, $($var:expr),+ ) => {{
        // This hint the required type for the variable passed if a compile error is show.
        let formatter: $crate::format::InputFormatParser = $formatter;
        match formatter.inputs($input) {
            Ok(inputs) => {
                let mut inputs_iter = inputs.iter();
                let mut result = Ok(());
                $(
                    if let Some(input) = inputs_iter.next() {
                        if !input.is_required_type_of_var(&$var) {
                            result = result.and_then($crate::error::placeholder_type_does_not_match);
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
                        result = result.and_then($crate::error::not_enough_placeholders);
                    }
                )*
                result
            }
            Err(error) => Err(error),
        }
    }};
    ($input:expr, $formatter:ident, $($var:ident),+ , ) => { $crate::sscanf!($input, $formatter, $($var),*) };
}

#[macro_export]
macro_rules! scanf {
    ($format:literal, $($var:expr),+ ) => {{
        let mut buffer = String::new();
        // In some use cases the output between scanf calls was not showed without this flush.
		let _ = std::io::Write::flush(&mut std::io::stdout());
        match std::io::stdin().read_line(&mut buffer) {
            Ok(_) => $crate::sscanf!(buffer.as_ref(), $format, $($var),*),
            Err(error) => Err(error),
        }
    }};
    ($format:literal, $($var:expr),+ , ) => { $crate::scanf!($format, $($var),*) };
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
    fn u8_and_u16() {
        let input = "5 -> 1024";
        let mut timeout: u8 = 0;
        let mut port: u16 = 0;
        sscanf!(input, "{u8} -> {u16}", timeout, port).unwrap();
        assert_eq!(timeout, 5);
        assert_eq!(port, 1024);
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

    #[test]
    fn into_array_elements() {
        let s = "3,4";
        let mut arr: [f64; 2] = [0.0; 2];
        sscanf!(&s, "{},{}", arr[0], arr[1]).unwrap();
    }
}
