use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::char,
    error::{self, context},
    multi::many0,
    sequence::delimited,
};

#[derive(Debug, PartialEq, Eq)]
pub enum InputFormatToken<'a> {
    Text(&'a str),
    GenericType,
    Variable(&'a str),
}

impl<'a> InputFormatToken<'a> {
    fn type_from_name(text: &'a str) -> std::io::Result<Self> {
        match text {
            "" => Ok(Self::GenericType),
            text => {
                // All non-empty content is treated as a variable name
                if is_valid_identifier(text) {
                    Ok(Self::Variable(text))
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("'{}' is not a valid variable name", text),
                    ))
                }
            }
        }
    }
}

pub(super) fn tokenize(input: &str) -> IResult<&str, Vec<InputFormatToken>> {
    let (remaining, mut tokens) = many0(input_format_token).parse(input)?;

    if !remaining.is_empty() {
        tokens.push(InputFormatToken::Text(remaining));
    }

    return Ok(("", tokens));
}

fn input_format_token(input: &str) -> IResult<&str, InputFormatToken> {
    alt((type_placeholder_token, text_token)).parse(input)
}

fn type_placeholder_token(input: &str) -> IResult<&str, InputFormatToken> {
    let mut type_parser = context(
        "input tag",
        delimited(char('{'), identifier_or_empty, char('}')),
    );
    let (remaining, type_name) = type_parser.parse(input)?;

    return match InputFormatToken::type_from_name(type_name) {
        Ok(type_token) => Ok((remaining, type_token)),
        Err(_) => Err(nom::Err::Failure(error::Error {
            input: type_name,
            code: error::ErrorKind::Tag,
        })),
    };
}

fn identifier_or_empty(input: &str) -> IResult<&str, &str> {
    alt((identifier, tag(""))).parse(input)
}

fn identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_').parse(input)
}

fn text_token(input: &str) -> IResult<&str, InputFormatToken> {
    let (remaining, text) =
        alt((tag("{{"), tag("}}"), take_until("{"), take_until("}"))).parse(input)?;

    let text_token = InputFormatToken::Text(unescape_text(text)?);
    return Ok((remaining, text_token));
}

fn unescape_text(text: &str) -> Result<&str, nom::Err<nom::error::Error<&str>>> {
    let unescape_text = match text {
        "{{" => "{",
        "}}" => "}",
        text => {
            if text.contains('}') {
                return Err(nom::Err::Failure(error::Error {
                    input: text,
                    code: error::ErrorKind::Tag,
                }));
            }
            text
        }
    };
    return Ok(unescape_text);
}

fn is_valid_identifier(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }

    let mut chars = text.chars();
    let first_char = chars.next().unwrap();

    // First character must be a letter or underscore
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }

    // Remaining characters must be alphanumeric or underscore
    chars.all(|c| c.is_alphanumeric() || c == '_')
}
