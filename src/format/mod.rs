use std::{any::Any, io};

mod format_parser;
use format_parser::InputFormatToken;

pub struct InputElement<'a> {
    input: &'a str,
    variable_name: Option<&'a str>,
}

impl<'a> InputElement<'a> {
    fn new(input: &'a str, variable_name: Option<&'a str>) -> Self {
        Self {
            // No longer need special handling for String type since we removed type checking
            input: input.trim(),
            variable_name,
        }
    }

    #[inline]
    pub const fn as_str(&self) -> &'a str {
        self.input
    }

    #[inline]
    #[deprecated(
        note = "Type checking is no longer needed since we removed type syntax. The compiler enforces type compatibility."
    )]
    pub fn is_required_type_of_var<T: ?Sized + Any>(&self, _var: &T) -> bool {
        // Since we removed type checking, all placeholders accept any type
        // The compiler will enforce type compatibility anyway
        true
    }

    #[inline]
    pub fn variable_name(&self) -> Option<&'a str> {
        self.variable_name
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

    #[inline]
    pub fn get_variable_names(&self) -> Vec<Option<&'a str>> {
        self.tokens
            .iter()
            .filter_map(|token| match token {
                InputFormatToken::Variable(name) => Some(Some(*name)),
                InputFormatToken::Anonymous => Some(None),
                InputFormatToken::Text(_) => None,
            })
            .collect()
    }

    pub fn inputs(&self, input: &'a str) -> io::Result<Vec<InputElement<'a>>> {
        let mut input = input;
        let mut capture: Option<Option<&'a str>> = None;
        let mut input_elements = Vec::with_capacity(self.count_placeholders());

        for token in &self.tokens {
            if let &InputFormatToken::Text(text) = token {
                if let Some(text_start_offset) = input.find(text) {
                    if let Some(variable_name) = capture {
                        capture = None;
                        let input_text = &input[..text_start_offset];
                        let input_element = InputElement::new(input_text, variable_name);
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

            match token {
                InputFormatToken::Anonymous => {
                    capture = Some(None);
                }
                InputFormatToken::Variable(name) => {
                    capture = Some(Some(name));
                }
                InputFormatToken::Text(_) => unreachable!(),
            }
        }

        if let Some(variable_name) = capture {
            let input_element = InputElement::new(input, variable_name);
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
        assert_eq!(formatter.tokens, vec![InputFormatToken::Anonymous])
    }

    #[test]
    fn test_formatter_two_generic_with_separator() {
        let formatter = InputFormatParser::new("{} -> {}").unwrap();
        assert_eq!(
            formatter.tokens,
            vec![
                InputFormatToken::Anonymous,
                InputFormatToken::Text(" -> "),
                InputFormatToken::Anonymous,
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
            vec![InputFormatToken::Anonymous, InputFormatToken::Anonymous,]
        )
    }

    #[test]
    fn test_formatter_number_and_string_without_separator() {
        // This test now uses variable names instead of types
        let formatter = InputFormatParser::new("{number}{text}").unwrap();
        assert_eq!(
            formatter.tokens,
            vec![
                InputFormatToken::Variable("number"),
                InputFormatToken::Variable("text"),
            ]
        )
    }

    #[test]
    fn test_formatter_variable_names() {
        let formatter = InputFormatParser::new("{name}: {value}").unwrap();
        assert_eq!(
            formatter.tokens,
            vec![
                InputFormatToken::Variable("name"),
                InputFormatToken::Text(": "),
                InputFormatToken::Variable("value"),
            ]
        )
    }

    #[test]
    fn test_formatter_mixed_variable_and_generic() {
        // This test now uses variable names and anonymous placeholders
        let formatter = InputFormatParser::new("{name}: {}").unwrap();
        assert_eq!(
            formatter.tokens,
            vec![
                InputFormatToken::Variable("name"),
                InputFormatToken::Text(": "),
                InputFormatToken::Anonymous,
            ]
        )
    }
}
