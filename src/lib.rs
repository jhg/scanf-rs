#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]
#![doc = include_str!("../README.md")]

#[doc(hidden)]
pub mod error;
#[doc(hidden)]
pub mod format;

#[cfg(test)]
mod examples;

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
        sscanf!(input, "{}: {}", request, reply).unwrap();
        assert_eq!(request, "Hello");
        assert_eq!(reply, "world");
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn string_and_float() {
        let input = "Candy->2.5";
        let mut product: String = String::new();
        let mut price: f64 = 0.0;
        sscanf!(input, "{}->{}", product, price,).unwrap();
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
        sscanf!(input, "{} -> {}", timeout, port).unwrap();
        assert_eq!(timeout, 5);
        assert_eq!(port, 1024);
    }

    #[test]
    fn string_between_brackets_ignored() {
        let input = "{Hello world}";
        let mut message: String = String::new();
        sscanf!(input, "{{{}}}", message).unwrap();
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
    fn into_array_elements() {
        let s = "3,4";
        let mut arr: [f64; 2] = [0.0; 2];
        sscanf!(&s, "{},{}", arr[0], arr[1]).unwrap();
    }

    #[test]
    fn test_variable_names_basic() {
        let input = "John: 25";
        let mut name: String = String::new();
        let mut age: i32 = 0;

        // Using variable names instead of types in format string
        sscanf!(input, "{name}: {age}", name, age).unwrap();
        assert_eq!(name, "John");
        assert_eq!(age, 25);
    }

    #[test]
    fn test_mixed_variable_names_and_generic() {
        let input = "Temperature: 23.5 degrees";
        let mut location: String = String::new();
        let mut temp: f32 = 0.0;
        let mut unit: String = String::new();

        // Mix variable names and generic placeholders
        sscanf!(input, "{location}: {} {unit}", location, temp, unit).unwrap();
        assert_eq!(location, "Temperature");
        assert_eq!(temp, 23.5);
        assert_eq!(unit, "degrees");
    }

    #[test]
    fn test_scanf_with_variable_names() {
        // Test that scanf! also works with variable names
        // Note: This would read from stdin in real usage, so we can't test it directly
        // But we can at least verify it compiles and the format string parses correctly
        let input_format = "{username}: {score}";
        let formatter = format::InputFormatParser::new(input_format).unwrap();

        // Verify that variable names are detected correctly
        let variable_names = formatter.get_variable_names();
        assert_eq!(variable_names, vec![Some("username"), Some("score")]);
    }

    #[test]
    fn test_type_syntax_rejection() {
        // Verify that old type syntax is no longer supported
        
        // These should fail because we treat 'i32' and 'string' as variable names
        // but they're not valid identifiers in a typical context
        let result1 = format::InputFormatParser::new("{i32}: {string}");
        // This should succeed because we treat them as variable names
        assert!(result1.is_ok());
        
        // Verify that the tokens are parsed as variable names
        let parser = result1.unwrap();
        let variable_names = parser.get_variable_names();
        assert_eq!(variable_names, vec![Some("i32"), Some("string")]);
        
        // But when actually using them, they work as variable names
        let input = "42: hello";
        let mut i32_val: i32 = 0;
        let mut string_val: String = String::new();
        
        // This should work because we no longer enforce type matching
        let result = sscanf!(input, "{i32}: {string}", i32_val, string_val);
        assert!(result.is_ok());
        assert_eq!(i32_val, 42);
        assert_eq!(string_val, "hello");
    }

    #[test]
    fn test_variable_name_validation() {
        // Test that invalid variable names are rejected
        let invalid_names = vec![
            "{123invalid}",  // starts with number
            "{with-dash}",   // contains dash  
            "{with space}",  // contains space
            "{with.dot}",    // contains dot
        ];
        
        for invalid_name in invalid_names {
            let result = format::InputFormatParser::new(invalid_name);
            assert!(result.is_err(), "Should reject invalid variable name: {}", invalid_name);
        }
        
        // Test that valid variable names are accepted
        let valid_names = vec![
            "{valid_name}",
            "{_underscore_start}",
            "{name123}",
            "{CamelCase}",
            "{snake_case}",
        ];
        
        for valid_name in valid_names {
            let result = format::InputFormatParser::new(valid_name);
            assert!(result.is_ok(), "Should accept valid variable name: {}", valid_name);
        }
    }
}
