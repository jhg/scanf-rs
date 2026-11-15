//! The `trim-whitespace` feature: behavior with it on and with it off.

use scanf::sscanf;
use std::io::ErrorKind;

/// Unwraps the error of a failed parse, checking it is the `InvalidInput` kind every
/// parsing failure uses, and returns its message.
fn invalid_input_message<T: std::fmt::Debug>(result: std::io::Result<T>) -> String {
    let error = result.unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    error.to_string()
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
