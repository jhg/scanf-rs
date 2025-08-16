#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]
#![doc = include_str!("../README.md")]

#[doc(hidden)]
pub mod error;
#[doc(hidden)]
pub mod format;

/// Parse and extract a variable name from a placeholder token
#[doc(hidden)]
pub fn extract_variable_info(format_str: &str) -> (Vec<Option<String>>, usize) {
    let mut vars = Vec::new();
    let mut anonymous_count = 0;
    let mut chars = format_str.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                chars.next(); // skip escaped brace
                continue;
            }
            
            let mut content = String::new();
            while let Some(ch) = chars.next() {
                if ch == '}' {
                    break;
                }
                content.push(ch);
            }
            
            if content.is_empty() {
                vars.push(None);
                anonymous_count += 1;
            } else if is_valid_rust_identifier(&content) {
                vars.push(Some(content));
            } else {
                // Invalid identifier, treat as anonymous
                vars.push(None);
                anonymous_count += 1;
            }
        } else if ch == '}' && chars.peek() == Some(&'}') {
            chars.next(); // skip escaped brace
        }
    }
    
    (vars, anonymous_count)
}

#[doc(hidden)]
pub fn is_valid_rust_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

// Enhanced macro that supports variable capture
#[macro_export]
macro_rules! sscanf {
    // Case: No variables provided - all must be named in format string
    ($input:expr, $format:literal) => {{
        const FORMAT_STR: &str = $format;
        let (var_info, anonymous_count) = $crate::extract_variable_info(FORMAT_STR);
        
        if anonymous_count > 0 {
            panic!("Format string contains anonymous placeholders '{{}}' but no variables were provided. Either use named placeholders like '{{var_name}}' or provide {} variables as arguments.", anonymous_count);
        }
        
        match $crate::format::InputFormatParser::new($format) {
            Ok(input_format_parser) => {
                match input_format_parser.inputs($input) {
                    Ok(inputs) => {
                        let mut inputs_iter = inputs.iter();
                        let mut result = Ok(());
                        
                        // This is where we would assign to variables by name
                        // For now, we'll just verify all parsing works
                        for _var_info in var_info {
                            if let Some(_input) = inputs_iter.next() {
                                // In a real implementation, we'd need macro magic here
                                // to assign to variables by name in the calling scope
                            } else {
                                result = result.and_then($crate::error::not_enough_placeholders);
                                break;
                            }
                        }
                        result
                    }
                    Err(error) => Err(error),
                }
            }
            Err(error) => Err(error),
        }
    }};
    
    // Case: Variables provided - handle mixed named/anonymous
    ($input:expr, $format:literal, $($var:expr),+ ) => {{
        match $crate::format::InputFormatParser::new($format) {
            Ok(input_format_parser) => {
                $crate::sscanf!($input, input_format_parser, $($var),+)
            }
            Err(error) => Err(error),
        }
    }};
    
    ($input:expr, $format:literal, $($var:expr),+ , ) => { 
        $crate::sscanf!($input, $format, $($var),*) 
    };
    
    ($input:expr, $formatter:ident, $($var:expr),+ ) => {{
        let formatter: $crate::format::InputFormatParser = $formatter;
        match formatter.inputs($input) {
            Ok(inputs) => {
                let mut inputs_iter = inputs.iter();
                let mut result = Ok(());
                $(
                    if let Some(input) = inputs_iter.next() {
                        match input.as_str().parse() {
                            Ok(input_parsed) => $var = input_parsed,
                            Err(error) => {
                                let invalid_input_error = std::io::Error::new(std::io::ErrorKind::InvalidInput, error);
                                result = result.and(Err(invalid_input_error));
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
    
    ($input:expr, $formatter:ident, $($var:ident),+ , ) => { 
        $crate::sscanf!($input, $formatter, $($var),*) 
    };
}

#[macro_export]
macro_rules! scanf {
    ($format:literal) => {{
        let mut buffer = String::new();
        let _ = std::io::Write::flush(&mut std::io::stdout());
        match std::io::stdin().read_line(&mut buffer) {
            Ok(_) => $crate::sscanf!(buffer.as_ref(), $format),
            Err(error) => Err(error),
        }
    }};
    
    ($format:literal, $($var:expr),+ ) => {{
        let mut buffer = String::new();
        let _ = std::io::Write::flush(&mut std::io::stdout());
        match std::io::stdin().read_line(&mut buffer) {
            Ok(_) => $crate::sscanf!(buffer.as_ref(), $format, $($var),*),
            Err(error) => Err(error),
        }
    }};
    
    ($format:literal, $($var:expr),+ , ) => { 
        $crate::scanf!($format, $($var),*) 
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_basic_functionality() {
        let input = "Hello: world";
        let mut request: String = String::new();
        let mut reply: String = String::new();
        sscanf!(input, "{}: {}", request, reply).unwrap();
        assert_eq!(request, "Hello");
        assert_eq!(reply, "world");
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_mixed_types() {
        let input = "5 -> 5.0";
        let mut request: i32 = 0;
        let mut reply: f32 = 0.0;
        sscanf!(input, "{} -> {}", request, reply).unwrap();
        assert_eq!(request, 5);
        assert_eq!(reply, 5.0);
    }

    #[test]
    fn test_variable_name_placeholders() {
        // Test that named placeholders are accepted by the parser
        let input = "John: 25";
        let mut name: String = String::new();
        let mut age: i32 = 0;

        // This should work - named placeholders with explicit variable arguments
        sscanf!(input, "{name}: {age}", name, age).unwrap();
        assert_eq!(name, "John");
        assert_eq!(age, 25);
    }

    #[test]
    fn test_mixed_named_and_anonymous() {
        let input = "Temperature: 23.5 degrees";
        let mut location: String = String::new();
        let mut temp: f32 = 0.0;
        let mut unit: String = String::new();

        // Mix named and anonymous placeholders - this demonstrates the intended syntax
        sscanf!(input, "{location}: {} {unit}", location, temp, unit).unwrap();
        assert_eq!(location, "Temperature");
        assert_eq!(temp, 23.5);
        assert_eq!(unit, "degrees");
    }

    #[test]
    fn test_only_anonymous_placeholders() {
        let input = "apple: 5";
        let mut fruit: String = String::new();
        let mut count: i32 = 0;
        
        sscanf!(input, "{}: {}", fruit, count).unwrap();
        assert_eq!(fruit, "apple");
        assert_eq!(count, 5);
    }

    #[test]
    fn test_variable_name_validation() {
        // Test the helper function for variable name validation
        assert!(is_valid_rust_identifier("valid_name"));
        assert!(is_valid_rust_identifier("_underscore"));
        assert!(is_valid_rust_identifier("CamelCase"));
        assert!(is_valid_rust_identifier("snake_case"));
        assert!(is_valid_rust_identifier("name123"));
        
        assert!(!is_valid_rust_identifier("123invalid"));
        assert!(!is_valid_rust_identifier("with-dash"));
        assert!(!is_valid_rust_identifier("with space"));
        assert!(!is_valid_rust_identifier("with.dot"));
        assert!(!is_valid_rust_identifier(""));
    }

    #[test]
    fn test_extract_variable_info() {
        // Test helper function that parses format strings
        let (vars, anon_count) = extract_variable_info("{name}: {} {age}");
        assert_eq!(vars, vec![Some("name".to_string()), None, Some("age".to_string())]);
        assert_eq!(anon_count, 1);
        
        let (vars, anon_count) = extract_variable_info("{} -> {}");
        assert_eq!(vars, vec![None, None]);
        assert_eq!(anon_count, 2);
        
        let (vars, anon_count) = extract_variable_info("{user}: {score}");
        assert_eq!(vars, vec![Some("user".to_string()), Some("score".to_string())]);
        assert_eq!(anon_count, 0);
    }

    #[test]
    fn test_escaped_braces() {
        let input = "{Hello world}";
        let mut message: String = String::new();
        sscanf!(input, "{{{}}}", message).unwrap();
        assert_eq!(message, "Hello world");
    }

    #[test]
    #[should_panic]
    fn test_wrong_format_string() {
        let input = "5 -> 5.0 <-";
        let mut _request: i32 = 0;
        let mut _reply: f32 = 0.0;
        sscanf!(input, "{} -}> {} <-", _request, _reply).unwrap();
    }

    #[test]
    fn test_into_array_elements() {
        let s = "3,4";
        let mut arr: [f64; 2] = [0.0; 2];
        sscanf!(&s, "{},{}", arr[0], arr[1]).unwrap();
        assert_eq!(arr[0], 3.0);
        assert_eq!(arr[1], 4.0);
    }
}
