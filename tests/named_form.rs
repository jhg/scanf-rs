//! The named form (`sscanf!(text, "{name}: {age}")`): each placeholder assigns to the variable
//! of the same name. Complements the basics in `integration_tests.rs` with every value type,
//! the places a call can appear and the failure behavior.

use scanf::sscanf;
use std::io::ErrorKind;

mod value_types {
    use super::*;

    #[test]
    fn every_integer_width() {
        let (mut a, mut b, mut c, mut d): (i8, u8, i16, u16) = (0, 0, 0, 0);
        let (mut e, mut f, mut g, mut h): (i32, u32, i64, u64) = (0, 0, 0, 0);
        let (mut i, mut j, mut k, mut l): (i128, u128, isize, usize) = (0, 0, 0, 0);
        sscanf!("-8 255 -300 65535", "{a} {b} {c} {d}").unwrap();
        sscanf!("-1 1 -2 2", "{e} {f} {g} {h}").unwrap();
        sscanf!(
            "-3 340282366920938463463374607431768211455 -4 4",
            "{i} {j} {k} {l}"
        )
        .unwrap();
        assert_eq!((a, b, c, d), (-8, 255, -300, 65_535));
        assert_eq!((e, f, g, h), (-1, 1, -2, 2));
        assert_eq!((i, j, k, l), (-3, u128::MAX, -4, 4));
    }

    #[test]
    fn floats_booleans_and_chars() {
        let (mut float, mut double): (f32, f64) = (0.0, 0.0);
        let (mut yes, mut no, mut letter) = (false, true, ' ');
        sscanf!(
            "2.5;-3.25e2;true;false;é",
            "{float};{double};{yes};{no};{letter}"
        )
        .unwrap();
        assert_eq!(
            (float, double, yes, no, letter),
            (2.5, -325.0, true, false, 'é')
        );
    }

    #[test]
    fn strings_keep_everything_up_to_the_separator() {
        let mut first = String::new();
        let mut rest = String::new();
        sscanf!("hello world: more: text", "{first}: {rest}").unwrap();
        assert_eq!(first, "hello world");
        assert_eq!(rest, "more: text");
    }

    #[test]
    fn standard_library_types_with_from_str() {
        let mut address = std::net::Ipv4Addr::UNSPECIFIED;
        let mut path = std::path::PathBuf::new();
        sscanf!("ip=10.0.0.1 file=/tmp/a", "ip={address} file={path}").unwrap();
        assert_eq!(address, std::net::Ipv4Addr::new(10, 0, 0, 1));
        assert_eq!(path, std::path::PathBuf::from("/tmp/a"));
    }

    #[test]
    fn custom_type_with_from_str() {
        #[derive(Debug, Default, PartialEq)]
        struct Celsius(f64);

        impl std::str::FromStr for Celsius {
            type Err = std::num::ParseFloatError;

            fn from_str(text: &str) -> Result<Self, Self::Err> {
                text.trim_end_matches('C').parse().map(Celsius)
            }
        }

        let mut temperature = Celsius::default();
        sscanf!("now: 21.5C", "now: {temperature}").unwrap();
        assert_eq!(temperature, Celsius(21.5));
    }

    #[test]
    fn the_type_is_the_one_the_variable_was_declared_with() {
        let mut as_text = String::new();
        let mut as_number: u32 = 0;
        sscanf!("42", "{as_text}").unwrap();
        sscanf!("42", "{as_number}").unwrap();
        assert_eq!(as_text, "42");
        assert_eq!(as_number, 42);
    }
}

mod format_shapes {
    use super::*;

    #[test]
    fn only_the_placeholder() {
        let mut n: u8 = 0;
        sscanf!("9", "{n}").unwrap();
        assert_eq!(n, 9);
    }

    #[test]
    fn text_before_between_and_after() {
        let (mut a, mut b): (i32, i32) = (0, 0);
        sscanf!("<1|2>", "<{a}|{b}>").unwrap();
        assert_eq!((a, b), (1, 2));
    }

    #[test]
    fn every_kind_of_separator() {
        let (mut a, mut b) = (String::new(), String::new());
        sscanf!("x y", "{a} {b}").unwrap();
        assert_eq!((a.as_str(), b.as_str()), ("x", "y"));
        sscanf!("x\ty", "{a}\t{b}").unwrap();
        assert_eq!((a.as_str(), b.as_str()), ("x", "y"));
        sscanf!("x-->y", "{a}-->{b}").unwrap();
        assert_eq!((a.as_str(), b.as_str()), ("x", "y"));
        sscanf!("x→y", "{a}→{b}").unwrap();
        assert_eq!((a.as_str(), b.as_str()), ("x", "y"));
    }

    #[test]
    fn escaped_braces() {
        let mut inner = String::new();
        sscanf!("{hi}", "{{{inner}}}").unwrap();
        assert_eq!(inner, "hi");
    }

    #[test]
    fn empty_value_before_the_separator() {
        let (mut a, mut b) = (String::from("old"), String::new());
        sscanf!(",x", "{a},{b}").unwrap();
        assert_eq!((a.as_str(), b.as_str()), ("", "x"));
    }

    #[test]
    fn variables_can_be_reused_across_calls() {
        let mut n: i32 = 0;
        let mut total = 0;
        for text in ["n=1", "n=2", "n=3"] {
            sscanf!(text, "n={n}").unwrap();
            total += n;
        }
        assert_eq!(total, 6);
    }

    #[test]
    fn a_name_used_twice_keeps_the_last_value() {
        let mut n: i32 = 0;
        sscanf!("1,2", "{n},{n}").unwrap();
        assert_eq!(n, 2);
    }

    #[test]
    fn the_same_text_with_different_formats() {
        let (mut a, mut b): (u32, u32) = (0, 0);
        sscanf!("10:20", "{a}:{b}").unwrap();
        assert_eq!((a, b), (10, 20));
        sscanf!("10:20", "{a}:20").unwrap();
        assert_eq!(a, 10);
        sscanf!("10:20", "10:{b}").unwrap();
        assert_eq!(b, 20);
    }
}

mod where_the_call_can_appear {
    use super::*;

    #[test]
    fn question_mark_in_a_function() {
        fn read_point(text: &str) -> std::io::Result<(i32, i32)> {
            let (mut x, mut y): (i32, i32) = (0, 0);
            sscanf!(text, "({x}, {y})")?;
            Ok((x, y))
        }

        assert_eq!(read_point("(3, -4)").unwrap(), (3, -4));
        assert_eq!(
            read_point("3, -4").unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
    }

    #[test]
    fn variables_declared_in_an_outer_scope() {
        let mut age: u32 = 0;
        {
            let mut name = String::new();
            sscanf!("Ana is 30", "{name} is {age}").unwrap();
            assert_eq!(name, "Ana");
        }
        assert_eq!(age, 30);
    }

    #[test]
    fn closure_that_captures_the_variable() {
        let mut value: i32 = 0;
        let mut read = |text: &str| sscanf!(text, "v={value}");
        read("v=5").unwrap();
        read("v=6").unwrap();
        assert_eq!(value, 6);
    }

    #[test]
    fn fields_of_a_struct_through_locals() {
        #[derive(Debug, PartialEq)]
        struct Person {
            name: String,
            age: u32,
        }

        let (mut name, mut age) = (String::new(), 0);
        sscanf!("Ana is 30", "{name} is {age}").unwrap();
        assert_eq!(
            Person { name, age },
            Person {
                name: "Ana".to_string(),
                age: 30
            }
        );
    }

    #[test]
    fn inside_an_iterator_chain() {
        let total: i32 = ["1,2", "3,4", "5,6"]
            .iter()
            .map(|line| {
                let (mut a, mut b): (i32, i32) = (0, 0);
                sscanf!(*line, "{a},{b}").unwrap();
                a * b
            })
            .sum();
        assert_eq!(total, 2 + 12 + 30);
    }

    #[test]
    fn with_a_shadowed_variable() {
        let n = String::from("not a number");
        let mut n: i32 = n.len().try_into().unwrap();
        sscanf!("7", "{n}").unwrap();
        assert_eq!(n, 7);
    }

    #[test]
    fn owned_string_input() {
        let line = String::from("5 6");
        let (mut a, mut b): (i32, i32) = (0, 0);
        sscanf!(&line, "{a} {b}").unwrap();
        assert_eq!(a + b, 11);
    }
}

mod failures {
    use super::*;

    #[test]
    fn wrong_type_is_invalid_input() {
        let mut n: i32 = 0;
        assert_eq!(
            sscanf!("abc", "{n}").unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
    }

    #[test]
    fn out_of_range_is_invalid_input() {
        let mut n: u8 = 0;
        assert_eq!(
            sscanf!("256", "{n}").unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(n, 0);
    }

    #[test]
    fn missing_text_is_invalid_input() {
        let (mut a, mut b): (i32, i32) = (0, 0);
        assert_eq!(
            sscanf!("1;2", "{a},{b}").unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
    }

    #[test]
    fn leading_text_that_does_not_match_leaves_variables_alone() {
        let mut n: i32 = -1;
        assert!(sscanf!("hello 5", "world {n}").is_err());
        assert_eq!(n, -1);
    }

    #[test]
    fn the_message_names_the_variable_and_the_text() {
        let (mut a, mut b): (i32, i32) = (0, 0);
        let message = sscanf!("1,y", "{a},{b}").unwrap_err().to_string();
        assert!(message.contains("variable 'b'"), "{message}");
        assert!(message.contains("\"y\""), "{message}");
    }

    #[test]
    fn values_before_the_failure_are_written_and_later_ones_are_not() {
        let (mut a, mut b, mut c): (i32, i32, i32) = (0, 0, 0);
        assert!(sscanf!("1,x,3", "{a},{b},{c}").is_err());
        assert_eq!((a, b, c), (1, 0, 0));
    }

    #[test]
    fn the_first_failing_placeholder_is_reported() {
        let (mut a, mut b): (i32, i32) = (0, 0);
        let message = sscanf!("x,y", "{a},{b}").unwrap_err().to_string();
        assert!(message.contains("variable 'a'"), "{message}");
    }
}

#[cfg(feature = "trim-whitespace")]
mod with_trim_whitespace {
    use super::*;

    #[test]
    fn numbers_are_trimmed_and_strings_are_not() {
        let mut name = String::new();
        let mut age: u32 = 0;
        sscanf!("  Ana  , 30 ", "{name},{age}").unwrap();
        assert_eq!((name.as_str(), age), ("  Ana  ", 30));
    }

    #[test]
    fn two_numbers_are_still_not_one_number() {
        let mut n: i32 = 0;
        assert!(sscanf!("1 2", "{n}").is_err());
    }
}
