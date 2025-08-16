#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]
#![doc = include_str!("../README.md")]

#[doc(hidden)]
pub mod format;

// Re-export the procedural macros
pub use scanf_proc_macro::{scanf, sscanf};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_basic_functionality() {
        let input = "Hello: world";
        let mut request: String = String::new();
        let mut reply: String = String::new();
        sscanf!(input, "{}: {}", &mut request, &mut reply).unwrap();
        assert_eq!(request, "Hello");
        assert_eq!(reply, "world");
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_mixed_types() {
        let input = "5 -> 5.0";
        let mut request: i32 = 0;
        let mut reply: f32 = 0.0;
        sscanf!(input, "{} -> {}", &mut request, &mut reply).unwrap();
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
        // Con el nuevo macro procedural, variables nombradas se capturan implícitamente
        sscanf!(input, "{name}: {age}").unwrap();
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
        sscanf!(input, "{location}: {} {unit}", &mut temp).unwrap();
        assert_eq!(location, "Temperature");
        assert_eq!(temp, 23.5);
        assert_eq!(unit, "degrees");
    }

    #[test]
    fn test_only_anonymous_placeholders() {
        let input = "apple: 5";
        let mut fruit: String = String::new();
        let mut count: i32 = 0;

        sscanf!(input, "{}: {}", &mut fruit, &mut count).unwrap();
        assert_eq!(fruit, "apple");
        assert_eq!(count, 5);
    }

    #[test]
    fn test_escaped_braces() {
        let input = "{Hello world}";
        let mut message: String = String::new();
        sscanf!(input, "{{{}}}", &mut message).unwrap();
        assert_eq!(message, "Hello world");
    }

    #[test]
    #[should_panic]
    fn test_wrong_format_string() {
        let input = "5 -> 5.0 <-";
        let mut _request: i32 = 0;
        let mut _reply: f32 = 0.0;
        // Forzamos error usando separador inexistente para provocar panic al unwrap
        sscanf!(input, "{} XXX_SEPARATOR {}", &mut _request, &mut _reply).unwrap();
    }

    #[test]
    fn test_into_array_elements() {
        let s = "3,4";
        let mut arr: [f64; 2] = [0.0; 2];
        sscanf!(s, "{},{}", &mut arr[0], &mut arr[1]).unwrap();
        assert_eq!(arr[0], 3.0);
        assert_eq!(arr[1], 4.0);
    }
}
