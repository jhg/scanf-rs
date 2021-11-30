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
        let (_, elements) = format_parser::tokenize(input).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid input format string: {}", error),
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
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Can not split input correctly because the consecutive placeholder",
                        ));
                    } else {
                        capture = true;
                    }
                }
                InputFormatToken::Text(text) => {
                    if let Some(text_start_offset) = input.find(text) {
                        if capture {
                            capture = false;
                            let input_element = &input[..text_start_offset];
                            input_elements.push(input_element);
                            input = &input[(text_start_offset + text.len())..];
                        } else {
                            input = &input[text.len()..];
                        }
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("Can not find text separator {:?}", text),
                        ));
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
    fn test_formatter_simple_generic() {
        let formatter = InputFormat::new("{}").unwrap();
        assert_eq!(formatter.elements, vec![InputFormatToken::GenericType])
    }

    #[test]
    fn test_formatter_two_generic_with_separator() {
        let formatter = InputFormat::new("{} -> {}").unwrap();
        assert_eq!(
            formatter.elements,
            vec![
                InputFormatToken::GenericType,
                InputFormatToken::Text(" -> "),
                InputFormatToken::GenericType,
            ]
        )
    }

    #[test]
    #[should_panic]
    fn test_wrong_formatter_unescaped_open_bracket() {
        InputFormat::new("{} -{> {}").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_wrong_formatter_unescaped_close_bracket() {
        InputFormat::new("{} -}> {}").unwrap();
    }

    #[test]
    fn test_formatter_two_generic_without_separator() {
        let formatter = InputFormat::new("{}{}").unwrap();
        assert_eq!(
            formatter.elements,
            vec![InputFormatToken::GenericType, InputFormatToken::GenericType,]
        )
    }
}
