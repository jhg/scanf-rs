//! `fscanf!` reads one line from any `BufRead`, the way `sscanf!` parses a string. The reader
//! comes first and is borrowed mutably, as with `write!`.

use scanf::fscanf;
use std::io::{BufRead, BufReader, Cursor, ErrorKind};

mod readers {
    use super::*;

    #[test]
    fn cursor() {
        let mut input = Cursor::new("5 apples\n");
        let (count, fruit): (u32, String) = fscanf!(input, "{} {}").unwrap();
        assert_eq!((count, fruit.as_str()), (5, "apples"));
    }

    #[test]
    fn byte_slice() {
        let mut input: &[u8] = b"7\n";
        let n: i32 = fscanf!(input, "{}").unwrap();
        assert_eq!(n, 7);
        assert!(input.is_empty());
    }

    #[test]
    fn buffered_reader() {
        let mut input = BufReader::new("x=1\n".as_bytes());
        let mut x: i32 = 0;
        fscanf!(input, "x={x}").unwrap();
        assert_eq!(x, 1);
    }

    #[test]
    fn reader_behind_a_mutable_reference() {
        let mut cursor = Cursor::new("1\n2\n");
        let first: i32 = fscanf!(&mut cursor, "{}").unwrap();
        let second: i32 = fscanf!(&mut cursor, "{}").unwrap();
        assert_eq!((first, second), (1, 2));
    }

    #[test]
    fn generic_function_over_any_buf_read() {
        fn read_pair<R: BufRead>(reader: &mut R) -> std::io::Result<(i32, i32)> {
            fscanf!(reader, "{},{}")
        }

        let mut input = Cursor::new("3,4\n");
        assert_eq!(read_pair(&mut input).unwrap(), (3, 4));
    }

    #[test]
    fn reader_parameter_that_is_not_declared_mut() {
        fn read(reader: &mut impl BufRead) -> i32 {
            fscanf!(reader, "{}").unwrap()
        }

        assert_eq!(read(&mut Cursor::new("4\n")), 4);
    }

    #[test]
    fn temporary_reader() {
        let n: i32 = fscanf!(Cursor::new("9\n"), "{}").unwrap();
        assert_eq!(n, 9);
    }

    #[test]
    fn reader_named_like_an_internal_variable() {
        let mut buffer = Cursor::new("1\n");
        let mut remaining = Cursor::new("2\n");
        let a: i32 = fscanf!(buffer, "{}").unwrap();
        let b: i32 = fscanf!(remaining, "{}").unwrap();
        assert_eq!((a, b), (1, 2));
    }
}

mod lines {
    use super::*;

    #[test]
    fn each_call_consumes_one_line() {
        let mut input = Cursor::new("one\ntwo\nthree\n");
        let first: String = fscanf!(input, "{}").unwrap();
        let second: String = fscanf!(input, "{}").unwrap();
        assert_eq!((first.as_str(), second.as_str()), ("one", "two"));

        let mut rest = String::new();
        input.read_line(&mut rest).unwrap();
        assert_eq!(rest, "three\n");
    }

    #[test]
    fn calls_in_a_loop_read_every_line() {
        let mut input = Cursor::new("1 2\n3 4\n5 6\n");
        let mut total = 0;
        loop {
            let line: std::io::Result<(i32, i32)> = fscanf!(input, "{} {}");
            let Ok((a, b)) = line else { break };
            total += a * b;
        }
        assert_eq!(total, 2 + 12 + 30);
    }

    #[test]
    fn crlf_and_missing_final_newline() {
        let mut input = Cursor::new("a\r\nb");
        let first: String = fscanf!(input, "{}").unwrap();
        let second: String = fscanf!(input, "{}").unwrap();
        assert_eq!((first.as_str(), second.as_str()), ("a", "b"));
    }

    #[test]
    fn only_one_line_ending_is_removed() {
        let mut input = Cursor::new("a\r\r\n");
        let text: String = fscanf!(input, "{}").unwrap();
        assert_eq!(text, "a\r");
    }

    #[test]
    fn empty_line_is_an_empty_value() {
        let mut input = Cursor::new("\nnext\n");
        let empty: String = fscanf!(input, "{}").unwrap();
        assert_eq!(empty, "");
    }

    #[test]
    fn the_line_is_consumed_even_when_parsing_fails() {
        let mut input = Cursor::new("x\n2\n");
        let failed: std::io::Result<i32> = fscanf!(input, "{}");
        assert_eq!(failed.unwrap_err().kind(), ErrorKind::InvalidInput);
        let next: i32 = fscanf!(input, "{}").unwrap();
        assert_eq!(next, 2);
    }
}

mod end_of_input {
    use super::*;

    #[test]
    fn empty_reader_is_unexpected_eof() {
        let mut input = Cursor::new("");
        let result: std::io::Result<String> = fscanf!(input, "{}");
        assert_eq!(result.unwrap_err().kind(), ErrorKind::UnexpectedEof);
    }

    #[test]
    fn after_the_last_line_is_unexpected_eof() {
        let mut input = Cursor::new("only\n");
        let line: String = fscanf!(input, "{}").unwrap();
        let again: std::io::Result<String> = fscanf!(input, "{}");
        assert_eq!(line, "only");
        assert_eq!(again.unwrap_err().kind(), ErrorKind::UnexpectedEof);
    }

    #[test]
    fn eof_leaves_named_variables_alone() {
        let mut input = Cursor::new("");
        let mut n: i32 = 4;
        assert_eq!(
            fscanf!(input, "{n}").unwrap_err().kind(),
            ErrorKind::UnexpectedEof
        );
        assert_eq!(n, 4);
    }

    #[test]
    fn invalid_utf8_is_an_invalid_data_error() {
        let mut input: &[u8] = b"\xff\xfe\n";
        let result: std::io::Result<String> = fscanf!(input, "{}");
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn errors_from_the_reader_are_propagated() {
        struct Broken;

        impl std::io::Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("disk on fire"))
            }
        }

        let mut input = BufReader::new(Broken);
        let result: std::io::Result<String> = fscanf!(input, "{}");
        let error = result.unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Other);
        assert!(error.to_string().contains("disk on fire"));
    }
}

mod forms {
    use super::*;

    #[test]
    fn returned_values() {
        let mut input = Cursor::new("Ana: 9.5\n");
        let (name, score): (String, f64) = fscanf!(input, "{}: {}").unwrap();
        assert_eq!((name.as_str(), score), ("Ana", 9.5));
    }

    #[test]
    fn named_placeholders() {
        let mut input = Cursor::new("Ana is 30\n");
        let mut name = String::new();
        let mut age: u32 = 0;
        fscanf!(input, "{name} is {age}").unwrap();
        assert_eq!((name.as_str(), age), ("Ana", 30));
    }

    #[test]
    fn named_and_returned_together() {
        let mut input = Cursor::new("7|Ana\n");
        let mut id: u32 = 0;
        let name: String = fscanf!(input, "{id}|{}").unwrap();
        assert_eq!((id, name.as_str()), (7, "Ana"));
    }

    #[test]
    fn explicit_arguments() {
        let mut input = Cursor::new("1,2\n");
        let (mut a, mut b): (i32, i32) = (0, 0);
        fscanf!(input, "{},{}", &mut a, &mut b).unwrap();
        assert_eq!((a, b), (1, 2));
    }

    #[test]
    fn question_mark_in_a_function() {
        fn read_age(input: &mut impl BufRead) -> std::io::Result<u32> {
            let age: u32 = fscanf!(input, "age={}")?;
            Ok(age)
        }

        assert_eq!(read_age(&mut Cursor::new("age=30\n")).unwrap(), 30);
        assert!(read_age(&mut Cursor::new("age=x\n")).is_err());
    }
}

mod matching_rules_are_those_of_sscanf {
    use super::*;

    #[test]
    #[cfg(not(feature = "ignore-trailing-input"))]
    fn text_after_the_last_literal_is_an_error_and_writes_nothing() {
        let mut input = Cursor::new("5 units extra\n");
        let mut n: i32 = 0;
        let message = fscanf!(input, "{n} units").unwrap_err().to_string();
        assert!(message.contains("\" extra\""), "{message}");
        assert_eq!(n, 0);
    }

    #[test]
    fn a_literal_at_the_end_matches_exactly() {
        let mut input = Cursor::new("5 units\n6 units\n");
        let first: i32 = fscanf!(input, "{} units").unwrap();
        let second: i32 = fscanf!(input, "{} units").unwrap();
        assert_eq!((first, second), (5, 6));
    }

    #[test]
    fn failures_write_nothing() {
        let mut input = Cursor::new("1,x\n");
        let (mut a, mut b): (i32, i32) = (9, 9);
        assert!(fscanf!(input, "{a},{b}").is_err());
        assert_eq!((a, b), (9, 9));
    }

    #[test]
    fn error_messages_name_the_placeholder() {
        let mut input = Cursor::new("1,y\n");
        let result: std::io::Result<(i32, i32)> = fscanf!(input, "{},{}");
        let message = result.unwrap_err().to_string();
        assert!(message.contains("anonymous placeholder #2"), "{message}");
    }

    #[test]
    fn the_rest_of_the_line_after_the_last_placeholder_belongs_to_it() {
        let mut input = Cursor::new("key=a=b c\nnext\n");
        let value: String = fscanf!(input, "key={}").unwrap();
        assert_eq!(value, "a=b c");
        let next: String = fscanf!(input, "{}").unwrap();
        assert_eq!(next, "next");
    }
}
