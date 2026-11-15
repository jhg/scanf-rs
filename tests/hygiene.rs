//! Variables of the macro never clash with the user's, nor do shadowed prelude names.

use scanf::sscanf;
use std::io::ErrorKind;

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
