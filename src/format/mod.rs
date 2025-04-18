use std::{
    any::{Any, TypeId},
    io,
};

mod format_parser;
use format_parser::InputFormatToken;

#[derive(Debug, PartialEq, Eq)]
enum InputType {
    Type(TypeId),
    GenericType,
}

impl InputType {
    fn typed<T: ?Sized + Any>() -> Self {
        // NOTE: in the future maybe can be const fn.
        Self::Type(TypeId::of::<T>())
    }
}

pub struct InputElement<'a> {
    input: &'a str,
    required_type: InputType,
}

impl<'a> InputElement<'a> {
    fn new(input: &'a str, required_type: InputType) -> Self {
        Self {
            input: if required_type != InputType::typed::<String>() {
                input.trim()
            } else {
                input
            },
            required_type,
        }
    }

    #[inline]
    pub const fn as_str(&self) -> &'a str {
        self.input
    }

    #[inline]
    pub fn is_required_type_of_var<T: ?Sized + Any>(&self, _var: &T) -> bool {
        match self.required_type {
            InputType::GenericType => true,
            InputType::Type(type_id) => type_id == TypeId::of::<T>(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InputFormatParser<'a> {
    tokens: Vec<InputFormatToken<'a>>,
}

impl<'a> InputFormatParser<'a> {
    pub fn new(input_format: &'a str) -> io::Result<Self> {
        let (_, tokens) = format_parser::tokenize(input_format).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid input format string: {}", error),
            )
        })?;
        return Ok(Self { tokens });
    }

    #[inline]
    fn count_placeholders(&self) -> usize {
        self.tokens
            .iter()
            .filter(|token| !matches!(token, InputFormatToken::Text(_)))
            .count()
    }

    pub fn inputs(&self, input: &'a str) -> io::Result<Vec<InputElement<'a>>> {
        let mut input = input;
        let mut capture = None;
        let mut input_elements = Vec::with_capacity(self.count_placeholders());

        for token in &self.tokens {
            if let &InputFormatToken::Text(text) = token {
                if let Some(text_start_offset) = input.find(text) {
                    if let Some(required_type) = capture {
                        capture = None;
                        let input_text = &input[..text_start_offset];
                        let input_element = InputElement::new(input_text, required_type);
                        input_elements.push(input_element);
                    }
                    input = &input[(text_start_offset + text.len())..];
                    continue;
                }
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Can not find text separator {:?}", text),
                ));
            }

            if capture.is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Can not split input correctly because the consecutive placeholder",
                ));
            }

            if let &InputFormatToken::Type(type_id) = token {
                capture = Some(InputType::Type(type_id));
            } else {
                capture = Some(InputType::GenericType);
            }
        }

        if let Some(required_type) = capture {
            let input_element = InputElement::new(input, required_type);
            input_elements.push(input_element);
        }

        return Ok(input_elements);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatter_simple_generic() {
        let formatter = InputFormatParser::new("{}").unwrap();
        assert_eq!(formatter.tokens, vec![InputFormatToken::GenericType])
    }

    #[test]
    fn test_formatter_two_generic_with_separator() {
        let formatter = InputFormatParser::new("{} -> {}").unwrap();
        assert_eq!(
            formatter.tokens,
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
        InputFormatParser::new("{} -{> {}").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_wrong_formatter_unescaped_close_bracket() {
        InputFormatParser::new("{} -}> {}").unwrap();
    }

    #[test]
    fn test_formatter_two_generic_without_separator() {
        let formatter = InputFormatParser::new("{}{}").unwrap();
        assert_eq!(
            formatter.tokens,
            vec![InputFormatToken::GenericType, InputFormatToken::GenericType,]
        )
    }

    #[test]
    fn test_formatter_number_and_string_without_separator() {
        let formatter = InputFormatParser::new("{i32}{string}").unwrap();
        assert_eq!(
            formatter.tokens,
            vec![
                InputFormatToken::typed::<i32>(),
                InputFormatToken::typed::<String>(),
            ]
        )
    }
}
