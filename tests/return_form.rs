//! The return form (`let n: i32 = sscanf!(text, "{}").unwrap()`): the same scenarios the
//! other test files cover with variables and `&mut` arguments, written the way the form is
//! meant to be used. The type always comes from the annotation, never from the macro.

use scanf::sscanf;
use std::io::ErrorKind;

mod single_values {
    use super::*;

    #[test]
    fn every_integer_width() {
        let a: i8 = sscanf!("-8", "{}").unwrap();
        let b: u8 = sscanf!("255", "{}").unwrap();
        let c: i16 = sscanf!("-300", "{}").unwrap();
        let d: u16 = sscanf!("65535", "{}").unwrap();
        let e: i64 = sscanf!("-1", "{}").unwrap();
        let f: u64 = sscanf!("18446744073709551615", "{}").unwrap();
        let g: i128 = sscanf!("-170141183460469231731687303715884105728", "{}").unwrap();
        let h: usize = sscanf!("7", "{}").unwrap();
        let i: isize = sscanf!("-7", "{}").unwrap();
        assert_eq!((a, b, c, d, e, f), (-8, 255, -300, 65_535, -1, u64::MAX));
        assert_eq!((g, h, i), (i128::MIN, 7, -7));
    }

    #[test]
    fn floats_booleans_and_chars() {
        let float: f32 = sscanf!("2.5", "{}").unwrap();
        let double: f64 = sscanf!("-3.25e2", "{}").unwrap();
        let yes: bool = sscanf!("true", "{}").unwrap();
        let no: bool = sscanf!("false", "{}").unwrap();
        let letter: char = sscanf!("é", "{}").unwrap();
        assert_eq!(
            (float, double, yes, no, letter),
            (2.5, -325.0, true, false, 'é')
        );
    }

    #[test]
    fn strings() {
        let text: String = sscanf!("hello world", "{}").unwrap();
        let empty: String = sscanf!("", "{}").unwrap();
        assert_eq!(text, "hello world");
        assert_eq!(empty, "");
    }

    #[test]
    fn leading_and_trailing_text() {
        let after: i32 = sscanf!("Value: 100", "Value: {}").unwrap();
        let before: i32 = sscanf!("100 units", "{} units").unwrap();
        let around: i32 = sscanf!("[100]", "[{}]").unwrap();
        assert_eq!((after, before, around), (100, 100, 100));
    }

    #[test]
    fn escaped_braces_around_a_value() {
        let message: String = sscanf!("{hi}", "{{{}}}").unwrap();
        assert_eq!(message, "hi");
    }

    #[test]
    fn standard_library_types_with_from_str() {
        let address: std::net::Ipv4Addr = sscanf!("ip=10.0.0.1", "ip={}").unwrap();
        let path: std::path::PathBuf = sscanf!("file=/tmp/a", "file={}").unwrap();
        assert_eq!(address, std::net::Ipv4Addr::new(10, 0, 0, 1));
        assert_eq!(path, std::path::PathBuf::from("/tmp/a"));
    }

    #[test]
    fn custom_type_with_from_str() {
        #[derive(Debug, PartialEq)]
        struct Celsius(f64);

        impl std::str::FromStr for Celsius {
            type Err = std::num::ParseFloatError;

            fn from_str(text: &str) -> Result<Self, Self::Err> {
                text.trim_end_matches('C').parse().map(Celsius)
            }
        }

        let temperature: Celsius = sscanf!("now: 21.5C", "now: {}").unwrap();
        assert_eq!(temperature, Celsius(21.5));
    }
}

mod several_values {
    use super::*;

    #[test]
    fn pairs_with_every_kind_of_separator() {
        let by_space: (i32, i32) = sscanf!("1 2", "{} {}").unwrap();
        let by_comma: (i32, i32) = sscanf!("1,2", "{},{}").unwrap();
        let by_word: (i32, i32) = sscanf!("1 and 2", "{} and {}").unwrap();
        let by_arrow: (String, String) = sscanf!("a→b", "{}→{}").unwrap();
        assert_eq!(by_space, (1, 2));
        assert_eq!(by_comma, (1, 2));
        assert_eq!(by_word, (1, 2));
        assert_eq!(by_arrow, ("a".to_string(), "b".to_string()));
    }

    #[test]
    fn mixed_types() {
        let (name, age, score, active): (String, u8, f64, bool) =
            sscanf!("Ana;31;9.5;true", "{};{};{};{}").unwrap();
        assert_eq!((name.as_str(), age, score, active), ("Ana", 31, 9.5, true));
    }

    #[test]
    fn record_with_labels() {
        let (user, age, score): (String, u32, f64) = sscanf!(
            "User: john_doe, Age: 25, Score: 95.5",
            "User: {}, Age: {}, Score: {}"
        )
        .unwrap();
        assert_eq!((user.as_str(), age, score), ("john_doe", 25, 95.5));
    }

    #[test]
    fn five_values() {
        let values: (char, char, char, char, char) =
            sscanf!("a:b:c:d:e", "{}:{}:{}:{}:{}").unwrap();
        assert_eq!(values, ('a', 'b', 'c', 'd', 'e'));
    }

    #[test]
    fn ten_values() {
        let (a, _, _, _, _, _, _, _, _, j): (i32, i32, i32, i32, i32, i32, i32, i32, i32, i32) =
            sscanf!("1 2 3 4 5 6 7 8 9 10", "{} {} {} {} {} {} {} {} {} {}").unwrap();
        assert_eq!((a, j), (1, 10));
    }

    #[test]
    fn empty_values_between_separators() {
        let (first, middle, last): (String, String, String) = sscanf!("a,,b", "{},{},{}").unwrap();
        assert_eq!(
            (first.as_str(), middle.as_str(), last.as_str()),
            ("a", "", "b")
        );
    }

    #[test]
    fn the_first_separator_wins() {
        let (head, tail): (String, String) = sscanf!("a:b:c", "{}:{}").unwrap();
        assert_eq!((head.as_str(), tail.as_str()), ("a", "b:c"));
    }

    #[test]
    fn values_are_returned_in_format_order() {
        let (a, b, c): (u8, u16, u32) = sscanf!("1/2/3", "{}/{}/{}").unwrap();
        assert_eq!((a, b, c), (1, 2, 3));
    }

    #[test]
    fn named_placeholders_do_not_take_part_in_the_tuple() {
        let mut id: u32 = 0;
        let (name, score): (String, f64) = sscanf!("7|Ana|9.5", "{id}|{}|{}").unwrap();
        assert_eq!((id, name.as_str(), score), (7, "Ana", 9.5));
    }
}

mod where_the_call_can_appear {
    use super::*;

    #[test]
    fn question_mark_in_a_function() {
        fn read_point(text: &str) -> std::io::Result<(i32, i32)> {
            let (x, y): (i32, i32) = sscanf!(text, "({}, {})")?;
            Ok((x, y))
        }

        assert_eq!(read_point("(3, -4)").unwrap(), (3, -4));
        assert_eq!(
            read_point("3, -4").unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
    }

    #[test]
    fn function_return_value() {
        fn read_age(text: &str) -> std::io::Result<u32> {
            sscanf!(text, "age={}")
        }

        assert_eq!(read_age("age=30").unwrap(), 30);
        assert!(read_age("age=x").is_err());
    }

    #[test]
    fn struct_construction() {
        #[derive(Debug, PartialEq)]
        struct Person {
            name: String,
            age: u32,
        }

        let (name, age) = sscanf!("Ana is 30", "{} is {}").unwrap();
        assert_eq!(
            Person { name, age },
            Person {
                name: "Ana".to_string(),
                age: 30
            }
        );
    }

    #[test]
    fn let_else_on_failure() {
        let Ok(n): std::io::Result<i32> = sscanf!("12", "{}") else {
            panic!("should parse");
        };
        assert_eq!(n, 12);
    }

    #[test]
    fn match_on_the_result() {
        let parsed: std::io::Result<u8> = sscanf!("x", "{}");
        let text = match parsed {
            Ok(_) => "number",
            Err(_) => "not a number",
        };
        assert_eq!(text, "not a number");
    }

    #[test]
    fn inside_an_iterator_chain() {
        let total: i32 = ["1,2", "3,4", "5,6"]
            .iter()
            .map(|line| {
                let (a, b): (i32, i32) = sscanf!(*line, "{},{}").unwrap();
                a * b
            })
            .sum();
        assert_eq!(total, 2 + 12 + 30);
    }

    #[test]
    fn collecting_results() {
        let numbers: Result<Vec<u32>, _> = ["1", "2", "x"]
            .iter()
            .map(|text| sscanf!(*text, "{}"))
            .collect::<Result<Vec<u32>, _>>();
        assert_eq!(numbers.unwrap_err().kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn argument_of_another_call() {
        assert_eq!(i32::abs(sscanf!("-5", "{}").unwrap()), 5);
    }

    #[test]
    fn in_a_const_free_closure() {
        let parse = |text: &str| -> std::io::Result<(String, u8)> { sscanf!(text, "{}={}") };
        assert_eq!(parse("a=1").unwrap(), ("a".to_string(), 1));
    }

    #[test]
    fn repeated_calls_in_a_loop() {
        let mut sum = 0;
        for text in ["1", "2", "3"] {
            let n: i32 = sscanf!(text, "{}").unwrap();
            sum += n;
        }
        assert_eq!(sum, 6);
    }

    #[test]
    fn owned_string_input() {
        let line = String::from("5 6");
        let (a, b): (i32, i32) = sscanf!(&line, "{} {}").unwrap();
        assert_eq!(a + b, 11);
    }
}

mod failures {
    use super::*;

    #[test]
    fn wrong_type_is_invalid_input() {
        let result: std::io::Result<i32> = sscanf!("abc", "{}");
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn out_of_range_is_invalid_input() {
        let result: std::io::Result<u8> = sscanf!("256", "{}");
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn negative_for_unsigned_is_invalid_input() {
        let result: std::io::Result<u32> = sscanf!("-1", "{}");
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn missing_text_is_invalid_input() {
        let result: std::io::Result<(i32, i32)> = sscanf!("1;2", "{},{}");
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn leading_text_that_does_not_match_is_invalid_input() {
        let result: std::io::Result<i32> = sscanf!("hello 5", "world {}");
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn the_second_value_failing_fails_the_whole_call() {
        let result: std::io::Result<(i32, i32)> = sscanf!("1,y", "{},{}");
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn the_message_names_the_placeholder_and_the_text() {
        let result: std::io::Result<(i32, i32)> = sscanf!("1,y", "{},{}");
        let message = result.unwrap_err().to_string();
        assert!(message.contains("anonymous placeholder #2"), "{message}");
        assert!(message.contains("\"y\""), "{message}");
    }

    #[test]
    fn nothing_is_returned_on_failure_so_nothing_can_be_half_used() {
        let result: std::io::Result<(String, i32)> = sscanf!("ok,x", "{},{}");
        assert!(result.is_err());
    }
}

#[cfg(feature = "trim-whitespace")]
mod with_trim_whitespace {
    use super::*;

    #[test]
    fn numbers_are_trimmed_and_strings_are_not() {
        let (name, age): (String, u32) = sscanf!("  Ana  , 30 ", "{},{}").unwrap();
        assert_eq!((name.as_str(), age), ("  Ana  ", 30));
    }

    #[test]
    fn two_numbers_are_still_not_one_number() {
        let result: std::io::Result<i32> = sscanf!("1 2", "{}");
        assert!(result.is_err());
    }
}
