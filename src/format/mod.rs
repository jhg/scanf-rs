use std::any::TypeId;
use std::io;

mod format_parser;

#[derive(Debug, PartialEq, Eq)]
enum InputFormatToken<'a> {
    Text(&'a str),
    Type(TypeId),
    GenericType,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InputFormat<'a> {
    elements: Vec<InputFormatToken<'a>>,
}

impl<'a> InputFormat<'a> {
    pub fn new(input: &'a str) -> io::Result<Self> {
        let (_, elements) = format_parser::tokenize(input).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid input format string: {}", e),
            )
        })?;
        return Ok(Self { elements });
    }

    pub fn input_strings(&self, input: &'a str) -> io::Result<Vec<&'a str>> {
        let mut input = input;
        let mut capture = false;
        let mut input_elements = Vec::new();
        for element in &self.elements {
            match *element {
                InputFormatToken::Type(_) | InputFormatToken::GenericType => {
                    if capture {
                        panic!("oooo")
                    } else {
                        capture = true
                    }
                }
                InputFormatToken::Text(text) => {
                    if capture {
                        capture = false;
                        if let Some(a) = input.find(text) {
                            let b = &input[..a];
                            input_elements.push(b);
                            input = input.strip_prefix(b).unwrap();
                            input = input.strip_prefix(text).unwrap();
                        } else {
                            panic!("");
                        }
                    } else {
                        input = input.strip_prefix(text).unwrap();
                    }
                }
            }
        }
        if capture {
            input_elements.push(input);
        }
        return Ok(input_elements);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple_generic_formatter() {
        let formatter = InputFormat::new("{} -> {}").unwrap();
        assert_eq!(
            formatter.elements,
            vec![
                InputFormatToken::GenericType,
                InputFormatToken::Text(" -> "),
                InputFormatToken::GenericType
            ]
        )
    }

    #[test]
    #[should_panic]
    fn test_wrong_formatter_unescaped_open_bracket() {
        let formatter = InputFormat::new("{} -{> {}").unwrap();
        println!("{:?}", formatter);
    }

    #[test]
    #[should_panic]
    fn test_wrong_formatter_unescaped_close_bracket() {
        let formatter = InputFormat::new("{} -}> {}").unwrap();
        println!("{:?}", formatter);
    }
}
