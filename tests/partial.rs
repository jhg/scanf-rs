//! `fscanf!` looks at the buffered line without consuming it and consumes only what matched:
//! text after a final number stays for the next call, and a failed call consumes nothing.

use scanf::{fscanf, sscanf};
use std::io::{BufRead, Cursor, ErrorKind};

fn rest_of(input: &mut Cursor<&str>) -> String {
    let mut line = String::new();
    input.read_line(&mut line).unwrap();
    line
}

mod leftovers {
    use super::*;

    #[test]
    fn text_after_a_final_number_is_left_for_the_next_call() {
        let mut input = Cursor::new("12 abc\nnext\n");
        let n: i32 = fscanf!(input, "{}").unwrap();
        assert_eq!(n, 12);
        let word: String = fscanf!(input, " {}").unwrap();
        assert_eq!(word, "abc");
        let next: String = fscanf!(input, "{}").unwrap();
        assert_eq!(next, "next");
    }

    #[test]
    fn two_numbers_on_one_line_in_two_calls() {
        let mut input = Cursor::new("1 2\n3\n");
        let a: i32 = fscanf!(input, "{}").unwrap();
        let b: i32 = fscanf!(input, " {}").unwrap();
        let c: i32 = fscanf!(input, "{}").unwrap();
        assert_eq!((a, b, c), (1, 2, 3));
    }

    #[test]
    fn the_rest_after_several_placeholders_is_left_too() {
        let mut input = Cursor::new("1 and 2 more\n");
        let (a, b): (i32, i32) = fscanf!(input, "{} and {}").unwrap();
        assert_eq!((a, b), (1, 2));
        assert_eq!(rest_of(&mut input), " more\n");
    }

    #[test]
    fn a_string_takes_the_whole_line_and_its_ending() {
        let mut input = Cursor::new("a b c\nnext\n");
        let line: String = fscanf!(input, "{}").unwrap();
        assert_eq!(line, "a b c");
        assert_eq!(rest_of(&mut input), "next\n");
    }

    #[test]
    fn a_number_filling_the_line_consumes_its_ending() {
        let mut input = Cursor::new("7\r\n8\r\n");
        let a: i32 = fscanf!(input, "{}").unwrap();
        let b: i32 = fscanf!(input, "{}").unwrap();
        assert_eq!((a, b), (7, 8));
    }

    #[test]
    fn the_ending_goes_only_after_the_last_token_of_a_crlf_line() {
        let mut input = Cursor::new("5 x\r\nnext\n");
        let n: i32 = fscanf!(input, "{}").unwrap();
        let x: String = fscanf!(input, " {}").unwrap();
        assert_eq!((n, x.as_str()), (5, "x"));
        assert_eq!(rest_of(&mut input), "next\n");
    }

    #[test]
    fn a_format_ending_in_text_consumes_the_line() {
        let mut input = Cursor::new("3 items\nnext\n");
        let n: i32 = fscanf!(input, "{} items").unwrap();
        assert_eq!(n, 3);
        assert_eq!(rest_of(&mut input), "next\n");
    }

    #[test]
    fn named_placeholders_leave_the_rest_too() {
        let mut input = Cursor::new("4 tail\n");
        let mut n = 0;
        fscanf!(input, "{n}").unwrap();
        assert_eq!(n, 4);
        assert_eq!(rest_of(&mut input), " tail\n");
    }
}

mod failures_consume_nothing {
    use super::*;

    #[test]
    fn a_number_glued_to_text_is_an_error_and_stays() {
        let mut input = Cursor::new("12abc\n");
        let result: std::io::Result<i32> = fscanf!(input, "{}");
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
        assert_eq!(rest_of(&mut input), "12abc\n");
    }

    #[test]
    fn a_fraction_is_not_an_integer_prefix() {
        let mut input = Cursor::new("1.5 x\n");
        let result: std::io::Result<i32> = fscanf!(input, "{}");
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
        assert_eq!(rest_of(&mut input), "1.5 x\n");
    }

    #[test]
    fn a_mismatched_literal_stays() {
        let mut input = Cursor::new("x=1\nnext\n");
        let result: std::io::Result<i32> = fscanf!(input, "y={}");
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
        assert_eq!(rest_of(&mut input), "x=1\n");
    }

    #[test]
    fn text_after_the_last_literal_is_an_error_and_stays() {
        let mut input = Cursor::new("3 items more\n");
        let result: std::io::Result<i32> = fscanf!(input, "{} items");
        #[cfg(not(feature = "ignore-trailing-input"))]
        {
            assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
            assert_eq!(rest_of(&mut input), "3 items more\n");
        }
        #[cfg(feature = "ignore-trailing-input")]
        assert_eq!(result.unwrap(), 3);
    }

    #[test]
    fn a_failed_call_can_be_retried_after_skipping_the_line() {
        let mut input = Cursor::new("x\n2\n");
        let failed: std::io::Result<i32> = fscanf!(input, "{}");
        assert!(failed.is_err());
        let skipped = rest_of(&mut input);
        assert_eq!(skipped, "x\n");
        let next: i32 = fscanf!(input, "{}").unwrap();
        assert_eq!(next, 2);
    }

    #[test]
    fn the_error_shows_the_whole_remaining_input() {
        let mut input = Cursor::new("12abc\n");
        let result: std::io::Result<i32> = fscanf!(input, "{}");
        assert!(result.unwrap_err().to_string().contains("12abc"));
    }

    #[test]
    fn named_variables_stay_untouched() {
        let mut input = Cursor::new("1 zz\n");
        let (mut a, mut b) = (0, 0);
        assert!(fscanf!(input, "{a} {b}").is_err());
        assert_eq!((a, b), (0, 0));
    }
}

mod reader_kinds {
    use super::*;

    #[test]
    fn a_dyn_buf_read_parameter_that_is_not_declared_mut() {
        fn read(input: &mut dyn BufRead) -> (i32, String) {
            let n: i32 = fscanf!(input, "{}").unwrap();
            let rest: String = fscanf!(input, " {}").unwrap();
            (n, rest)
        }
        let mut input = Cursor::new("4 left\n");
        assert_eq!(read(&mut input), (4, "left".to_string()));
    }
}

mod sscanf_stays_strict {
    use super::*;

    #[test]
    fn text_after_a_final_number_is_an_error() {
        let result: std::io::Result<i32> = sscanf!("12 abc", "{}");
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
    }
}

mod lines_the_buffer_does_not_show_whole {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn a_last_line_without_ending_parses() {
        let mut input = Cursor::new("12");
        let n: i32 = fscanf!(input, "{}").unwrap();
        assert_eq!(n, 12);
    }

    #[test]
    fn a_line_longer_than_the_buffer_still_parses() {
        let mut input = BufReader::with_capacity(4, "12 abc\n".as_bytes());
        let n: i32 = fscanf!(input, "{}").unwrap();
        assert_eq!(n, 12);
    }
}
