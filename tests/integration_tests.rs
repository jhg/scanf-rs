use scanf::sscanf;
use std::io::ErrorKind;

/// Unwraps the error of a failed parse, checking it is the `InvalidInput` kind every
/// parsing failure uses, and returns its message.
fn invalid_input_message<T: std::fmt::Debug>(result: std::io::Result<T>) -> String {
    let error = result.unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    error.to_string()
}

mod named_placeholders {
    use super::*;

    #[test]
    fn write_into_variables_of_the_same_name() {
        let input = "John: 25";
        let mut name: String = String::new();
        let mut age: i32 = 0;

        sscanf!(input, "{name}: {age}").unwrap();
        assert_eq!(name, "John");
        assert_eq!(age, 25);
    }

    #[test]
    fn capture_several_variables() {
        let input = "Charlie: 35.5 kg";
        let mut name3: String = String::new();
        let mut weight: f32 = 0.0;
        let mut unit3: String = String::new();

        sscanf!(input, "{name3}: {weight} {unit3}").unwrap();

        assert_eq!(name3, "Charlie");
        assert_eq!(weight, 35.5);
        assert_eq!(unit3, "kg");
    }

    #[test]
    fn names_do_not_have_to_match_the_text() {
        let input = "Score: 95, Player: Alice";
        let mut any_name: u32 = 0;
        let mut other_name: String = String::new();
        sscanf!(input, "Score: {any_name}, Player: {other_name}").unwrap();
        assert_eq!(any_name, 95);
        assert_eq!(other_name, "Alice");
    }

    #[test]
    fn single_placeholder() {
        let input = "42";
        let mut value: i32 = 0;
        sscanf!(input, "{value}").unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn leading_fixed_text() {
        let input = "Value: 100";
        let mut value: i32 = 0;
        sscanf!(input, "Value: {value}").unwrap();
        assert_eq!(value, 100);
    }

    #[test]
    fn names_with_digits() {
        let input = "42, 3.25";
        let mut var1: i32 = 0;
        let mut var2: f64 = 0.0;
        sscanf!(input, "{var1}, {var2}").unwrap();
        assert_eq!(var1, 42);
        assert_eq!(var2, 3.25);
    }

    #[test]
    fn names_with_underscores() {
        let input = "100 200";
        let mut _private_var: i32 = 0;
        let mut my_var: i32 = 0;
        sscanf!(input, "{_private_var} {my_var}").unwrap();
        assert_eq!(_private_var, 100);
        assert_eq!(my_var, 200);
    }

    #[test]
    fn unicode_names() {
        let input = "42";
        let mut número: i32 = 0;
        sscanf!(input, "{número}").unwrap();
        assert_eq!(número, 42);
    }

    #[test]
    fn record_with_several_fields() {
        let input = "User: john_doe, Age: 25, Score: 95.5";
        let mut username: String = String::new();
        let mut user_age: i32 = 0;
        let mut user_score: f64 = 0.0;
        sscanf!(
            input,
            "User: {username}, Age: {user_age}, Score: {user_score}"
        )
        .unwrap();
        assert_eq!(username, "john_doe");
        assert_eq!(user_age, 25);
        assert_eq!(user_score, 95.5);
    }

    #[test]
    fn empty_value_before_the_separator() {
        let input = ":end";
        let mut value: String = String::new();
        let mut marker: String = String::new();
        sscanf!(input, "{value}:{marker}").unwrap();
        assert_eq!(value, "");
        assert_eq!(marker, "end");
    }

    #[test]
    fn five_fields_split_on_the_same_separator() {
        let input = "a:b:c:d:e";
        let mut a = String::new();
        let mut b = String::new();
        let mut c = String::new();
        let mut d = String::new();
        let mut e = String::new();
        sscanf!(input, "{a}:{b}:{c}:{d}:{e}").unwrap();
        assert_eq!(a, "a");
        assert_eq!(b, "b");
        assert_eq!(c, "c");
        assert_eq!(d, "d");
        assert_eq!(e, "e");
    }

    #[test]
    fn ten_placeholders() {
        let input = "1 2 3 4 5 6 7 8 9 10";
        let mut n1: i32 = 0;
        let mut n2: i32 = 0;
        let mut n3: i32 = 0;
        let mut n4: i32 = 0;
        let mut n5: i32 = 0;
        let mut n6: i32 = 0;
        let mut n7: i32 = 0;
        let mut n8: i32 = 0;
        let mut n9: i32 = 0;
        let mut n10: i32 = 0;
        sscanf!(input, "{n1} {n2} {n3} {n4} {n5} {n6} {n7} {n8} {n9} {n10}").unwrap();
        assert_eq!(n1, 1);
        assert_eq!(
            n2 + n3 + n4 + n5 + n6 + n7 + n8 + n9,
            2 + 3 + 4 + 5 + 6 + 7 + 8 + 9
        );
        assert_eq!(n10, 10);
    }

    #[test]
    fn long_name() {
        let input = "42";
        let mut this_is_a_very_long_variable_name_but_still_valid: i32 = 0;
        sscanf!(input, "{this_is_a_very_long_variable_name_but_still_valid}").unwrap();
        assert_eq!(this_is_a_very_long_variable_name_but_still_valid, 42);
    }

    #[test]
    fn repeated_name_keeps_the_last_value() {
        let mut n: i32 = 0;
        sscanf!("1,2", "{n},{n}").unwrap();
        assert_eq!(n, 2);
    }
}

mod explicit_arguments {
    use super::*;

    #[test]
    fn strings() {
        let input = "Hello: world";
        let mut request: String = String::new();
        let mut reply: String = String::new();
        sscanf!(input, "{}: {}", &mut request, &mut reply).unwrap();
        assert_eq!(request, "Hello");
        assert_eq!(reply, "world");
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn mixed_types() {
        let input = "5 -> 5.0";
        let mut request: i32 = 0;
        let mut reply: f32 = 0.0;
        sscanf!(input, "{} -> {}", &mut request, &mut reply).unwrap();
        assert_eq!(request, 5);
        assert_eq!(reply, 5.0);
    }

    #[test]
    fn string_then_number() {
        let input = "apple: 5";
        let mut fruit: String = String::new();
        let mut count: i32 = 0;

        sscanf!(input, "{}: {}", &mut fruit, &mut count).unwrap();
        assert_eq!(fruit, "apple");
        assert_eq!(count, 5);
    }

    #[test]
    fn name_and_age() {
        let input = "Bob: 30";
        let mut name: String = String::new();
        let mut age: i32 = 0;

        sscanf!(input, "{}: {}", &mut name, &mut age).unwrap();

        assert_eq!(name, "Bob");
        assert_eq!(age, 30);
    }

    #[test]
    fn into_array_elements() {
        let s = "3,4";
        let mut arr: [f64; 2] = [0.0; 2];
        sscanf!(s, "{},{}", &mut arr[0], &mut arr[1]).unwrap();
        assert_eq!(arr[0], 3.0);
        assert_eq!(arr[1], 4.0);
    }

    #[test]
    fn trailing_fixed_text() {
        let input = "100 units";
        let mut value: i32 = 0;
        sscanf!(input, "{} units", &mut value).unwrap();
        assert_eq!(value, 100);
    }

    #[test]
    fn negative_numbers() {
        let input = "-42, -3.25";
        let mut int_val: i32 = 0;
        let mut float_val: f64 = 0.0;
        sscanf!(input, "{}, {}", &mut int_val, &mut float_val).unwrap();
        assert_eq!(int_val, -42);
        assert!((float_val + 3.25).abs() < f64::EPSILON * 100.0);
    }

    #[test]
    fn negative_value_for_unsigned_type_is_an_error() {
        let input = "-1";
        let mut value: u32 = 0;
        let result = sscanf!(input, "{}", &mut value);
        assert!(result.is_err());
        assert_eq!(value, 0);
    }

    #[test]
    fn booleans() {
        let input = "true false";
        let mut a: bool = false;
        let mut b: bool = true;
        sscanf!(input, "{} {}", &mut a, &mut b).unwrap();
        assert!(a);
        assert!(!b);
    }

    #[test]
    fn chars() {
        let input = "a b";
        let mut first: char = ' ';
        let mut second: char = ' ';
        sscanf!(input, "{} {}", &mut first, &mut second).unwrap();
        assert_eq!(first, 'a');
        assert_eq!(second, 'b');
    }

    #[test]
    fn argument_with_a_cast() {
        let mut n: i32 = 0;
        sscanf!("5", "{}", &mut n as &mut i32).unwrap();
        assert_eq!(n, 5);
    }

    #[test]
    fn argument_that_is_a_struct_field() {
        struct Point {
            x: i32,
            y: i32,
        }
        let mut point = Point { x: 0, y: 0 };
        sscanf!("3,4", "{},{}", &mut point.x, &mut point.y).unwrap();
        assert_eq!((point.x, point.y), (3, 4));
    }

    #[test]
    fn argument_that_is_a_dereferenced_reference() {
        let mut n: i32 = 0;
        let target = &mut n;
        sscanf!("9", "{}", &mut *target).unwrap();
        assert_eq!(n, 9);
    }

    #[test]
    fn argument_that_is_indexed_with_a_variable() {
        let mut values = vec![0_i32; 3];
        let index = 1;
        sscanf!("8", "{}", &mut values[index]).unwrap();
        assert_eq!(values, [0, 8, 0]);
    }

    #[test]
    fn trailing_comma_after_the_arguments() {
        let (mut a, mut b): (i32, i32) = (0, 0);
        sscanf!("1,2", "{},{}", &mut a, &mut b,).unwrap();
        assert_eq!((a, b), (1, 2));
    }
}

mod mixed_placeholders {
    use super::*;

    #[test]
    fn named_around_an_anonymous_one() {
        let input = "Temperature: 23.5 degrees";
        let mut location: String = String::new();
        let mut temp: f32 = 0.0;
        let mut unit: String = String::new();

        sscanf!(input, "{location}: {} {unit}", &mut temp).unwrap();
        assert_eq!(location, "Temperature");
        assert_eq!(temp, 23.5);
        assert_eq!(unit, "degrees");
    }

    #[test]
    fn named_before_and_after_an_anonymous_one() {
        let input = "Alice: 25 years";
        let mut name: String = String::new();
        let mut age: i32 = 0;
        let mut unit: String = String::new();

        sscanf!(input, "{name}: {} {unit}", &mut age).unwrap();

        assert_eq!(name, "Alice");
        assert_eq!(age, 25);
        assert_eq!(unit, "years");
    }
}

mod return_mode {
    use super::*;

    #[test]
    fn single_value() {
        let s = "42";
        let r: i32 = sscanf!(s, "{}").unwrap();
        assert_eq!(r, 42);
    }

    #[test]
    fn tuple() {
        let input = "Name: Ana, Age: 30";
        let (name, age): (String, u32) = sscanf!(input, "Name: {}, Age: {}").unwrap();
        assert_eq!(name, "Ana");
        assert_eq!(age, 30);
    }

    #[test]
    fn three_values() {
        let (name, count, ratio): (String, i32, f64) = sscanf!("a|1|2.5", "{}|{}|{}").unwrap();
        assert_eq!((name.as_str(), count, ratio), ("a", 1, 2.5));
    }

    #[test]
    fn invalid_value_is_an_error() {
        let result: std::io::Result<u32> = sscanf!("abc", "{}");
        assert!(result.is_err());
    }

    #[test]
    fn named_placeholder_alongside_returned_values() {
        let mut id: u32 = 0;
        let (name, score): (String, f64) = sscanf!("7|Ana|9.5", "{id}|{}|{}").unwrap();
        assert_eq!((id, name.as_str(), score), (7, "Ana", 9.5));
    }

    #[test]
    fn only_the_anonymous_placeholders_are_returned() {
        let mut id: u32 = 0;
        let name: String = sscanf!("7|Ana", "{id}|{}").unwrap();
        assert_eq!((id, name.as_str()), (7, "Ana"));
    }

    #[test]
    fn named_placeholders_alone_return_unit() {
        let mut n: i32 = 0;
        let result: std::io::Result<()> = sscanf!("5", "{n}");
        assert!(result.is_ok());
        assert_eq!(n, 5);
    }

    #[test]
    fn trailing_comma_without_arguments_still_returns_the_value() {
        let n: i32 = sscanf!("5", "{}",).unwrap();
        assert_eq!(n, 5);
    }

    #[test]
    fn escaped_braces_only_return_unit() {
        let result: std::io::Result<()> = sscanf!("{}", "{{}}");
        assert!(result.is_ok());
    }
}

mod format_syntax {
    use super::*;

    #[test]
    fn escaped_braces() {
        let input = "{Hello world}";
        let mut message: String = String::new();
        sscanf!(input, "{{{}}}", &mut message).unwrap();
        assert_eq!(message, "Hello world");
    }

    #[test]
    fn several_escaped_braces_in_a_row() {
        let input = "{{test}}";
        let mut value: String = String::new();
        sscanf!(input, "{{{{{value}}}}}",).unwrap();
        assert_eq!(value, "test");
    }

    #[test]
    fn whitespace_in_separators_is_preserved() {
        let input = "10  20";
        let mut a: i32 = 0;
        let mut b: i32 = 0;
        sscanf!(input, "{}  {}", &mut a, &mut b).unwrap();
        assert_eq!(a, 10);
        assert_eq!(b, 20);
    }

    #[test]
    fn punctuation_separator() {
        let input = "10|20";
        let mut a: i32 = 0;
        let mut b: i32 = 0;
        sscanf!(input, "{}|{}", &mut a, &mut b).unwrap();
        assert_eq!(a, 10);
        assert_eq!(b, 20);
    }

    #[test]
    fn multi_character_separator() {
        let input = "apple-->banana";
        let mut a: String = String::new();
        let mut b: String = String::new();
        sscanf!(input, "{}-->{}", &mut a, &mut b).unwrap();
        assert_eq!(a, "apple");
        assert_eq!(b, "banana");
    }

    #[test]
    fn unicode_separator() {
        let input = "10→20";
        let mut a: i32 = 0;
        let mut b: i32 = 0;
        sscanf!(input, "{}→{}", &mut a, &mut b).unwrap();
        assert_eq!(a, 10);
        assert_eq!(b, 20);
    }

    #[test]
    fn empty_input_parses_as_an_empty_string() {
        let input = "";
        let mut value: String = String::new();
        let result = sscanf!(input, "{value}");
        assert!(result.is_ok());
        assert_eq!(value, "");
    }

    #[test]
    fn input_trimmed_by_the_caller() {
        let input = "42  ";
        let mut value: i32 = 0;
        sscanf!(input.trim_end(), "{value}").unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn only_literal_text() {
        let input = "exact match";
        let result = sscanf!(input, "exact match");
        assert!(result.is_ok());
    }

    #[test]
    fn only_literal_text_that_does_not_match() {
        let input = "wrong text";
        let result = sscanf!(input, "exact match");
        assert!(result.is_err());
    }

    #[test]
    fn literal_text_longer_than_the_input() {
        assert!(sscanf!("exa", "exact").is_err());
        assert!(sscanf!("", "x").is_err());
    }

    #[test]
    fn raw_string_format() {
        let (left, right): (String, String) = sscanf!(r"a\b", r"{}\{}").unwrap();
        assert_eq!((left.as_str(), right.as_str()), ("a", "b"));
    }

    #[test]
    fn format_split_with_a_backslash_continuation() {
        let (a, b): (i32, i32) = sscanf!(
            "1:2",
            "{}\
             :{}"
        )
        .unwrap();
        assert_eq!((a, b), (1, 2));
    }
}

mod parsing_behavior {
    use super::*;

    #[test]
    fn the_first_separator_wins() {
        let (first, rest): (String, String) = sscanf!("a:b:c", "{}:{}").unwrap();
        assert_eq!((first.as_str(), rest.as_str()), ("a", "b:c"));
    }

    #[test]
    fn empty_slice_before_a_separator_is_a_valid_string() {
        let (first, second): (String, String) = sscanf!(":x", "{}:{}").unwrap();
        assert_eq!((first.as_str(), second.as_str()), ("", "x"));
    }

    #[test]
    fn empty_slice_before_a_separator_is_not_a_number() {
        let result: std::io::Result<(i32, i32)> = sscanf!(",5", "{},{}");
        assert!(result.is_err());
    }

    #[test]
    fn consecutive_separators_give_an_empty_middle_value() {
        let (a, b, c): (String, String, String) = sscanf!("a,,b", "{},{},{}").unwrap();
        assert_eq!((a.as_str(), b.as_str(), c.as_str()), ("a", "", "b"));
    }

    #[test]
    fn nothing_after_the_last_separator() {
        let (number, rest): (i32, String) = sscanf!("5,", "{},{}").unwrap();
        assert_eq!((number, rest.as_str()), (5, ""));

        let result: std::io::Result<(i32, i32)> = sscanf!("5,", "{},{}");
        assert!(result.is_err());
    }

    #[test]
    fn input_ending_before_the_separator_is_an_error() {
        let result: std::io::Result<(i32, i32)> = sscanf!("5", "{},{}");
        assert!(result.is_err());
    }

    #[test]
    fn multibyte_separators_and_values() {
        let (left, right): (String, String) = sscanf!("héllo→wörld", "{}→{}").unwrap();
        assert_eq!((left.as_str(), right.as_str()), ("héllo", "wörld"));
    }

    #[test]
    fn multibyte_char_value() {
        let single: char = sscanf!("é;", "{};").unwrap();
        assert_eq!(single, 'é');

        let result: std::io::Result<char> = sscanf!("éé;", "{};");
        assert!(result.is_err());
    }

    #[test]
    #[cfg(not(feature = "ignore-trailing-input"))]
    fn text_after_the_last_literal_is_an_error() {
        let mut n: i32 = 0;
        let message = invalid_input_message(sscanf!("5 units extra", "{} units", &mut n));
        assert!(message.contains("\" extra\""), "{message}");
        assert_eq!(n, 0);
    }

    #[test]
    #[cfg(not(feature = "ignore-trailing-input"))]
    fn text_after_the_last_literal_is_an_error_in_every_form() {
        let returned: std::io::Result<i32> = sscanf!("5 units extra", "{} units");
        assert_eq!(returned.unwrap_err().kind(), ErrorKind::InvalidInput);

        let mut n: i32 = 0;
        assert!(sscanf!("5 units extra", "{n} units").is_err());

        assert!(sscanf!("exact match!", "exact match").is_err());
        assert!(sscanf!("a:b:", "a:b").is_err());
    }

    #[test]
    #[cfg(not(feature = "ignore-trailing-input"))]
    fn trailing_whitespace_after_the_last_literal_is_an_error() {
        let result: std::io::Result<i32> = sscanf!("5 units ", "{} units");
        assert!(result.is_err());
    }

    #[test]
    fn input_that_ends_exactly_at_the_last_literal_is_accepted() {
        let n: i32 = sscanf!("5 units", "{} units").unwrap();
        assert_eq!(n, 5);
        sscanf!("exact match", "exact match").unwrap();
    }

    #[test]
    fn a_final_placeholder_still_takes_the_rest_of_the_input() {
        let rest: String = sscanf!("key=a=b c", "key={}").unwrap();
        assert_eq!(rest, "a=b c");
    }

    #[test]
    fn nothing_is_written_when_a_later_placeholder_fails() {
        let (mut a, mut b): (i32, i32) = (0, 0);
        let result = sscanf!("7,x", "{a},{b}");
        assert!(result.is_err());
        assert_eq!((a, b), (0, 0));
    }

    #[test]
    #[cfg(not(feature = "ignore-trailing-input"))]
    fn nothing_is_written_when_the_trailing_text_does_not_match() {
        let (mut a, mut b): (i32, i32) = (1, 2);
        assert!(sscanf!("7,8 extra", "{a},{b} units").is_err());
        assert!(sscanf!("7,8 units extra", "{a},{b} units").is_err());
        assert_eq!((a, b), (1, 2));
    }

    #[test]
    fn nothing_is_written_when_a_separator_is_missing() {
        let (mut a, mut b): (i32, i32) = (1, 2);
        assert!(sscanf!("7;8", "{a},{b}").is_err());
        assert_eq!((a, b), (1, 2));
    }

    #[test]
    fn nothing_is_written_through_explicit_arguments_either() {
        let (mut a, mut b): (i32, i32) = (1, 2);
        assert!(sscanf!("7,x", "{},{}", &mut a, &mut b).is_err());
        assert_eq!((a, b), (1, 2));
    }

    #[test]
    #[cfg(not(feature = "ignore-trailing-input"))]
    fn nothing_is_written_through_explicit_arguments_when_trailing_text_does_not_match() {
        let (mut a, mut b): (i32, i32) = (1, 2);
        assert!(sscanf!("7,8 extra", "{},{} units", &mut a, &mut b).is_err());
        assert_eq!((a, b), (1, 2));
    }

    #[test]
    fn nothing_is_written_to_named_placeholders_mixed_with_explicit_ones() {
        let (mut name, mut age): (String, i32) = (String::from("old"), 3);
        assert!(sscanf!("Ana: x", "{name}: {}", &mut age).is_err());
        assert_eq!((name.as_str(), age), ("old", 3));
    }

    #[test]
    fn every_value_is_written_when_all_of_them_parse() {
        let (mut a, mut b, mut c): (i32, String, f64) = (0, String::new(), 0.0);
        sscanf!("7,mid,2.5", "{a},{b},{c}").unwrap();
        assert_eq!((a, b.as_str(), c), (7, "mid", 2.5));
    }

    #[test]
    fn argument_expressions_are_evaluated_once() {
        let mut slots = [0_i32; 3];
        let mut calls = 0;
        let mut next = || {
            calls += 1;
            calls - 1
        };
        sscanf!("5,6", "{},{}", &mut slots[next()], &mut slots[next()]).unwrap();
        assert_eq!(slots, [5, 6, 0]);
        assert_eq!(calls, 2);
    }

    #[test]
    fn unsigned_byte_limits() {
        let largest: u8 = sscanf!("255", "{}").unwrap();
        assert_eq!(largest, u8::MAX);

        let too_large: std::io::Result<u8> = sscanf!("256", "{}");
        assert!(too_large.is_err());
        let negative: std::io::Result<u8> = sscanf!("-1", "{}");
        assert!(negative.is_err());
    }

    #[test]
    fn signed_integer_limits() {
        let min: i32 = sscanf!("-2147483648", "{}").unwrap();
        let max: i32 = sscanf!("2147483647", "{}").unwrap();
        assert_eq!((min, max), (i32::MIN, i32::MAX));

        let too_large: std::io::Result<i32> = sscanf!("2147483648", "{}");
        assert!(too_large.is_err());
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn float_notations() {
        let thousand: f64 = sscanf!("1e3", "{}").unwrap();
        let plus_five: f64 = sscanf!("+5", "{}").unwrap();
        let negative_zero: f64 = sscanf!("-0", "{}").unwrap();
        assert_eq!(thousand, 1000.0);
        assert_eq!(plus_five, 5.0);
        assert!(negative_zero == 0.0 && negative_zero.is_sign_negative());
    }

    #[test]
    fn custom_from_str_type_reports_its_own_error() {
        #[derive(Debug)]
        struct Even(u32);

        impl std::str::FromStr for Even {
            type Err = String;

            fn from_str(text: &str) -> Result<Self, Self::Err> {
                match text.parse::<u32>() {
                    Ok(n) if n % 2 == 0 => Ok(Even(n)),
                    Ok(n) => Err(format!("{n} is odd")),
                    Err(error) => Err(error.to_string()),
                }
            }
        }

        let Even(value) = sscanf!("8", "{}").unwrap();
        assert_eq!(value, 8);

        let result: std::io::Result<Even> = sscanf!("7", "{}");
        assert!(invalid_input_message(result).contains("7 is odd"));
    }
}

#[cfg(feature = "ignore-trailing-input")]
mod ignore_trailing_input {
    use super::*;

    #[test]
    fn input_after_the_last_literal_is_ignored() {
        let mut n: i32 = 0;
        sscanf!("5 units extra", "{} units", &mut n).unwrap();
        assert_eq!(n, 5);
    }

    #[test]
    fn in_every_form() {
        let returned: i32 = sscanf!("5 units extra", "{} units").unwrap();
        assert_eq!(returned, 5);

        let mut n: i32 = 0;
        sscanf!("6 units extra", "{n} units").unwrap();
        assert_eq!(n, 6);

        sscanf!("exact match!", "exact match").unwrap();
        sscanf!("a:b:", "a:b").unwrap();
    }

    #[test]
    fn trailing_whitespace_is_ignored() {
        let n: i32 = sscanf!("5 units \n", "{} units").unwrap();
        assert_eq!(n, 5);
    }

    #[test]
    fn values_are_written_when_everything_before_the_end_parses() {
        let (mut a, mut b): (i32, i32) = (0, 0);
        sscanf!("7,8 units extra", "{a},{b} units").unwrap();
        assert_eq!((a, b), (7, 8));
    }

    #[test]
    fn the_text_before_and_between_must_still_match() {
        let mut n: i32 = 0;
        assert!(sscanf!("5 meters extra", "{n} units").is_err());
        assert!(sscanf!("x 5 units", "v {n} units").is_err());
        assert!(sscanf!("5", "{n} units").is_err());
        assert_eq!(n, 0);
    }

    #[test]
    fn a_failure_still_writes_nothing() {
        let (mut a, mut b): (i32, i32) = (1, 2);
        assert!(sscanf!("7,x units extra", "{a},{b} units").is_err());
        assert_eq!((a, b), (1, 2));
    }
}

mod errors {
    use super::*;

    #[test]
    fn wrong_separator_is_an_invalid_input_error() {
        let mut request: i32 = 0;
        let mut reply: f32 = 0.0;
        let result = sscanf!(
            "5 -> 5.0 <-",
            "{} XXX_SEPARATOR {}",
            &mut request,
            &mut reply
        );
        let message = invalid_input_message(result);
        assert!(message.contains("XXX_SEPARATOR"), "{message}");
        assert_eq!((request, reply), (0, 0.0));
    }

    #[test]
    fn type_mismatch_leaves_earlier_values_unwritten() {
        let input = "Score: 95, Player: Alice";
        let mut score: String = String::new();
        let mut _player: u32 = 0;
        let result = sscanf!(input, "Score: {score}, Player: {_player}");
        assert!(result.is_err());
        assert_eq!(score, "");
    }

    #[test]
    fn missing_separator_reports_the_placeholder_and_the_remaining_input() {
        let result: std::io::Result<(i32, i32)> = sscanf!("abc", "{},{}");
        let message = invalid_input_message(result);
        assert!(message.contains("anonymous placeholder #1"), "{message}");
        assert!(message.contains("\",\""), "{message}");
        assert!(message.contains("\"abc\""), "{message}");
    }

    #[test]
    fn parse_failure_before_a_separator_reports_the_slice() {
        let mut n: i32 = 0;
        let mut m: i32 = 0;
        let message = invalid_input_message(sscanf!("x,2", "{n},{m}"));
        assert!(message.contains("variable 'n'"), "{message}");
        assert!(message.contains("\"x\""), "{message}");
        assert!(message.contains("invalid digit"), "{message}");
        assert_eq!((n, m), (0, 0));
    }

    #[test]
    fn parse_failure_in_the_final_slice_reports_the_remaining_input() {
        let result: std::io::Result<(i32, i32)> = sscanf!("1,y", "{},{}");
        let message = invalid_input_message(result);
        assert!(message.contains("anonymous placeholder #2"), "{message}");
        assert!(message.contains("remaining input \"y\""), "{message}");
    }

    #[test]
    fn mismatched_leading_text_reports_the_remaining_input() {
        let result: std::io::Result<i32> = sscanf!("hello 5", "world {}");
        let message = invalid_input_message(result);
        assert!(message.contains("Expected text \"world \""), "{message}");
        assert!(message.contains("\"hello 5\""), "{message}");
    }

    #[test]
    fn anonymous_placeholders_are_numbered_apart_from_named_ones() {
        let mut first_named: String = String::new();
        let mut second_named: String = String::new();
        let (mut a, mut b, mut c): (i32, i32, i32) = (0, 0, 0);
        let result = sscanf!(
            "1,x,3,y,z",
            "{},{first_named},{},{second_named},{}",
            &mut a,
            &mut b,
            &mut c
        );
        let message = invalid_input_message(result);
        assert!(message.contains("anonymous placeholder #3"), "{message}");
        assert_eq!((a, b, c), (0, 0, 0));
        assert_eq!((first_named.as_str(), second_named.as_str()), ("", ""));
    }

    #[test]
    fn the_first_failing_placeholder_is_reported() {
        let result: std::io::Result<(i32, i32)> = sscanf!("x,y", "{},{}");
        let message = invalid_input_message(result);
        assert!(message.contains("anonymous placeholder #1"), "{message}");
        assert!(!message.contains("#2"), "{message}");
    }

    #[test]
    fn named_placeholders_are_described_by_variable_name() {
        let mut n: i32 = 0;
        let message = invalid_input_message(sscanf!("nope", "{n}"));
        assert!(message.contains("variable 'n'"), "{message}");
    }
}

mod hygiene {
    use super::*;

    #[test]
    fn named_placeholders_with_the_names_of_internal_variables() {
        let mut remaining: i32 = 0;
        let mut parsed: i32 = 0;
        let mut pos: i32 = 0;
        let mut slice: i32 = 0;
        let mut buffer: i32 = 0;
        sscanf!("1,2,3,4,5", "{remaining},{parsed},{pos},{slice},{buffer}").unwrap();
        assert_eq!((remaining, parsed, pos, slice, buffer), (1, 2, 3, 4, 5));
    }

    #[test]
    fn explicit_arguments_named_like_internal_variables() {
        let mut remaining: i32 = 0;
        let mut parsed: i32 = 0;
        sscanf!("7;8", "{};{}", &mut remaining, &mut parsed).unwrap();
        assert_eq!((remaining, parsed), (7, 8));
    }

    #[test]
    fn user_variables_are_left_alone() {
        let input = "test";
        let mut value: String = String::new();
        let remaining: String = String::from("should not be modified");
        let buffer: String = String::from("should not be modified");

        sscanf!(input, "{}", &mut value).unwrap();

        assert_eq!(value, "test");
        assert_eq!(remaining, "should not be modified");
        assert_eq!(buffer, "should not be modified");
    }

    #[test]
    fn variables_named_like_return_mode_outputs() {
        let out0 = 1;
        let out1 = 2;
        let error = 3;
        let value = 4;
        let (a, b): (i32, i32) = sscanf!("10,20", "{},{}").unwrap();
        assert_eq!((a, b, out0, out1, error, value), (10, 20, 1, 2, 3, 4));
    }

    #[test]
    fn named_placeholders_called_like_return_mode_outputs() {
        let mut out0: i32 = 0;
        let mut out1: i32 = 0;
        let (a, b): (i32, i32) = sscanf!("1|2|3|4", "{out0}|{}|{out1}|{}").unwrap();
        assert_eq!((out0, out1, a, b), (1, 3, 2, 4));
    }

    #[test]
    fn input_variable_named_remaining() {
        let remaining = "5";
        let n: i32 = sscanf!(remaining, "{}").unwrap();
        assert_eq!(n, 5);
    }

    #[test]
    fn input_variable_named_like_the_internal_input_binding() {
        let input = "6";
        let n: i32 = sscanf!(input, "{}").unwrap();
        assert_eq!(n, 6);
    }

    #[test]
    fn two_calls_in_one_scope() {
        let a: i32 = sscanf!("1", "{}").unwrap();
        let b: i32 = sscanf!("2", "{}").unwrap();
        let (c, d): (i32, i32) = sscanf!("3,4", "{},{}").unwrap();
        assert_eq!((a, b, c, d), (1, 2, 3, 4));
    }

    #[test]
    fn question_mark_around_the_call() {
        fn parse_pair(text: &str) -> std::io::Result<(i32, i32)> {
            let (a, b): (i32, i32) = sscanf!(text, "{},{}")?;
            Ok((a, b))
        }

        assert_eq!(parse_pair("1,2").unwrap(), (1, 2));
        assert_eq!(
            parse_pair("1;2").unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
    }
}

mod shadowed_prelude {
    use super::*;

    #[allow(dead_code)]
    enum Mine {
        Ok(i32),
        Err,
    }

    #[test]
    fn result_variants_shadowed_by_a_glob_import() {
        use Mine::*;

        let mut n: i32 = 0;
        let success = sscanf!("5", "{}", &mut n);
        let failure = sscanf!("x", "{}", &mut n);
        assert!(success.is_ok());
        assert!(failure.is_err());
        assert_eq!(n, 5);
        assert!(matches!((Ok(1), Err), (Mine::Ok(1), Mine::Err)));
    }

    #[test]
    fn return_mode_with_shadowed_result_variants() {
        use Mine::*;

        let (a, b): (i32, i32) = sscanf!("1,2", "{},{}").unwrap();
        assert_eq!((a, b), (1, 2));
        assert!(matches!(Ok(0), Mine::Ok(0)));
        assert!(matches!(Err, Mine::Err));
    }

    #[test]
    #[allow(dead_code)]
    fn local_types_named_like_prelude_types() {
        struct Result;
        struct Option;
        struct String;

        let mut n: i32 = 0;
        sscanf!("5", "{}", &mut n).unwrap();
        assert_eq!(n, 5);
    }

    #[test]
    fn local_format_macro() {
        #[allow(unused_macros)]
        macro_rules! format {
            ($($arg:tt)*) => {
                compile_error!("the generated code must not use this macro")
            };
        }

        let mut n: i32 = 0;
        sscanf!("5", "{}", &mut n).unwrap();
        assert_eq!(n, 5);
    }
}

mod input_expressions {
    use super::*;

    fn padded_line() -> String {
        String::from("  5  ")
    }

    #[test]
    fn input_expression_borrowing_a_temporary_compiles() {
        let n: i32 = sscanf!(padded_line().trim(), "{}").unwrap();
        assert_eq!(n, 5);
    }

    #[test]
    fn temporary_input_with_only_literal_text() {
        sscanf!(padded_line().trim(), "5").unwrap();
    }

    #[test]
    fn temporary_input_with_explicit_arguments_and_named_placeholders() {
        let mut n: i32 = 0;
        sscanf!(padded_line().trim(), "{}", &mut n).unwrap();
        let mut m: i32 = 0;
        sscanf!(padded_line().trim(), "{m}").unwrap();
        assert_eq!((n, m), (5, 5));
    }

    #[test]
    fn accepted_input_types() {
        let owned = String::from("5");
        let borrowed: &str = "5";
        let double_borrowed: &&str = &borrowed;

        let from_literal: i32 = sscanf!("5", "{}").unwrap();
        let from_str: i32 = sscanf!(borrowed, "{}").unwrap();
        let from_string_ref: i32 = sscanf!(&owned, "{}").unwrap();
        let from_as_str: i32 = sscanf!(owned.as_str(), "{}").unwrap();
        let from_deref: i32 = sscanf!(&*owned, "{}").unwrap();
        let from_double_ref: i32 = sscanf!(double_borrowed, "{}").unwrap();
        let from_temporary_string: i32 = sscanf!(&String::from("5"), "{}").unwrap();
        let from_temporary_str: i32 = sscanf!(String::from("5").as_str(), "{}").unwrap();

        assert_eq!(
            [
                from_literal,
                from_str,
                from_string_ref,
                from_as_str,
                from_deref,
                from_double_ref,
                from_temporary_string,
                from_temporary_str,
            ],
            [5; 8]
        );
    }

    #[test]
    fn input_expression_is_evaluated_once() {
        let calls = std::cell::Cell::new(0);
        let source = || {
            calls.set(calls.get() + 1);
            "1,2"
        };
        let (a, b): (i32, i32) = sscanf!(source(), "{},{}").unwrap();
        assert_eq!((a, b), (1, 2));
        assert_eq!(calls.get(), 1);

        let result: std::io::Result<(i32, i32)> = sscanf!(source(), "{};{}");
        assert!(result.is_err());
        assert_eq!(calls.get(), 2);
    }
}

mod trim_whitespace {
    use super::*;

    #[cfg(feature = "trim-whitespace")]
    mod enabled {
        use super::*;

        #[test]
        fn number_tolerates_surrounding_whitespace() {
            let mut n: i32 = 0;
            sscanf!(" 42 ", "{n}").unwrap();
            assert_eq!(n, 42);
        }

        #[test]
        fn numbers_tolerate_repeated_separator_whitespace() {
            let (mut a, mut b): (u32, f64) = (0, 0.0);
            sscanf!("1   2.5 ", "{a} {b}").unwrap();
            assert_eq!(a, 1);
            assert_eq!(b, 2.5);
        }

        #[test]
        fn failure_after_trimming_reports_the_error_of_the_untrimmed_text() {
            // " " fails as an invalid digit, while the trimmed "" would fail as empty.
            let mut n: i32 = 0;
            let error = sscanf!(" ", "{}", &mut n).unwrap_err();
            assert!(error.to_string().contains("invalid digit"), "{error}");
        }

        #[test]
        fn string_keeps_surrounding_whitespace() {
            let mut s = String::new();
            let mut n: i32 = 0;
            sscanf!("  hi  ,  7 ", "{s},{n}").unwrap();
            assert_eq!(s, "  hi  ");
            assert_eq!(n, 7);
        }

        #[test]
        fn invalid_number_with_whitespace_still_fails() {
            let mut n: i32 = 0;
            assert!(sscanf!(" 4x ", "{n}").is_err());
        }

        #[test]
        fn two_numbers_are_not_a_single_number() {
            let mut n: i32 = 0;
            assert!(sscanf!("1 2", "{n}").is_err());
        }

        #[test]
        fn return_mode_trims() {
            let n: i32 = sscanf!(" 42 ", "{}").unwrap();
            assert_eq!(n, 42);
        }

        #[test]
        fn tabs_newlines_and_unicode_whitespace_are_trimmed() {
            let tabs: i32 = sscanf!("\t42\t", "{}").unwrap();
            let newlines: i32 = sscanf!("\n42\r\n", "{}").unwrap();
            let unicode: i32 = sscanf!("\u{2003}42\u{a0}", "{}").unwrap();
            assert_eq!((tabs, newlines, unicode), (42, 42, 42));
        }

        #[test]
        fn leading_or_trailing_whitespace_alone_is_trimmed() {
            let leading: i32 = sscanf!(" 42", "{}").unwrap();
            let trailing: i32 = sscanf!("42 ", "{}").unwrap();
            assert_eq!((leading, trailing), (42, 42));
        }

        #[test]
        fn every_number_in_a_list_is_trimmed() {
            let (a, b, c): (i32, i32, i32) = sscanf!(" 1 , 2 ,3 ", "{},{},{}").unwrap();
            assert_eq!((a, b, c), (1, 2, 3));
        }

        #[test]
        fn bool_with_leading_whitespace() {
            let flag: bool = sscanf!(" true", "{}").unwrap();
            assert!(flag);
        }

        #[test]
        fn char_with_leading_whitespace_is_trimmed() {
            let letter: char = sscanf!(" a", "{}").unwrap();
            assert_eq!(letter, 'a');
        }

        #[test]
        fn a_whitespace_char_stays_whitespace() {
            let space: char = sscanf!(" ", "{}").unwrap();
            assert_eq!(space, ' ');
        }

        #[test]
        fn returned_string_keeps_its_padding() {
            let text: String = sscanf!(" hi ", "{}").unwrap();
            assert_eq!(text, " hi ");
        }

        #[test]
        fn failure_reports_the_untrimmed_input() {
            let mut n: i32 = 0;
            let message = invalid_input_message(sscanf!(" x ", "{n}"));
            assert!(message.contains("\" x \""), "{message}");
        }

        #[test]
        fn failure_in_a_separated_slice_reports_the_untrimmed_slice() {
            let result: std::io::Result<(i32, i32)> = sscanf!(" x ,1", "{},{}");
            let message = invalid_input_message(result);
            assert!(message.contains("\" x \""), "{message}");
        }
    }

    #[cfg(not(feature = "trim-whitespace"))]
    mod disabled {
        use super::*;

        #[test]
        fn number_whitespace_is_strict_by_default() {
            let mut n: i32 = 0;
            assert!(sscanf!(" 42 ", "{n}").is_err());
        }

        #[test]
        fn return_mode_is_strict() {
            let result: std::io::Result<i32> = sscanf!(" 42 ", "{}");
            assert!(result.is_err());
        }

        #[test]
        fn tabs_newlines_and_unicode_whitespace_are_errors() {
            let tabs: std::io::Result<i32> = sscanf!("\t42\t", "{}");
            let newlines: std::io::Result<i32> = sscanf!("\n42\r\n", "{}");
            let unicode: std::io::Result<i32> = sscanf!("\u{2003}42\u{a0}", "{}");
            assert!(tabs.is_err() && newlines.is_err() && unicode.is_err());
        }

        #[test]
        fn leading_or_trailing_whitespace_alone_is_an_error() {
            let leading: std::io::Result<i32> = sscanf!(" 42", "{}");
            let trailing: std::io::Result<i32> = sscanf!("42 ", "{}");
            assert!(leading.is_err() && trailing.is_err());
        }

        #[test]
        fn padded_numbers_in_a_list_are_errors() {
            let result: std::io::Result<(i32, i32, i32)> = sscanf!(" 1 , 2 ,3 ", "{},{},{}");
            assert!(result.is_err());
        }

        #[test]
        fn exact_numbers_in_a_list_still_parse() {
            let (a, b, c): (i32, i32, i32) = sscanf!("1,2,3", "{},{},{}").unwrap();
            assert_eq!((a, b, c), (1, 2, 3));
        }

        #[test]
        fn bool_with_leading_whitespace_is_an_error() {
            let flag: std::io::Result<bool> = sscanf!(" true", "{}");
            assert!(flag.is_err());
        }

        #[test]
        fn char_with_leading_whitespace_is_an_error() {
            let letter: std::io::Result<char> = sscanf!(" a", "{}");
            assert!(letter.is_err());
        }

        #[test]
        fn a_whitespace_char_stays_whitespace() {
            let space: char = sscanf!(" ", "{}").unwrap();
            assert_eq!(space, ' ');
        }

        #[test]
        fn returned_string_keeps_its_padding() {
            let text: String = sscanf!(" hi ", "{}").unwrap();
            assert_eq!(text, " hi ");
        }

        #[test]
        fn failure_reports_the_untrimmed_input() {
            let mut n: i32 = 0;
            let message = invalid_input_message(sscanf!(" x ", "{n}"));
            assert!(message.contains("\" x \""), "{message}");
        }
    }
}
