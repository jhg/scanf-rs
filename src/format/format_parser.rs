use std::any::TypeId;

use super::InputFormatToken;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alphanumeric0, char},
    error,
    multi::many0,
    sequence::delimited,
    IResult,
};

impl<'a> InputFormatToken<'a> {
    fn type_from_str(text: &'a str) -> Self {
        match text {
            "i32" => Self::Type(TypeId::of::<i32>()),
            "u32" => Self::Type(TypeId::of::<u32>()),
            "f32" => Self::Type(TypeId::of::<f32>()),
            "i64" => Self::Type(TypeId::of::<i64>()),
            "u64" => Self::Type(TypeId::of::<u64>()),
            "f64" => Self::Type(TypeId::of::<f64>()),
            "string" => Self::Type(TypeId::of::<String>()),
            "" => Self::GenericType,
            text => Self::Text(text), // TODO: Refactor this to return result and handle it.
        }
    }
}

pub(super) fn tokenize(input: &str) -> IResult<&str, Vec<InputFormatToken>> {
    let (remaining, mut tokens) = many0(input_format_token)(input)?;
    if !remaining.is_empty() {
        tokens.push(InputFormatToken::Text(remaining));
    }
    return Ok(("", tokens));
}

fn input_format_token(input: &str) -> IResult<&str, InputFormatToken> {
    alt((type_format, text))(input)
}

fn text(input: &str) -> IResult<&str, InputFormatToken> {
    let (remaining, text) = alt((tag("{{"), tag("}}"), take_until("{")))(input)?;
    return Ok((remaining, InputFormatToken::Text(text)));
}

fn type_format(input: &str) -> IResult<&str, InputFormatToken> {
    let (remaining, type_name) = delimited(char('{'), alphanumeric0, char('}'))(input)?;
    match InputFormatToken::type_from_str(type_name) {
        InputFormatToken::Text(_) => Err(nom::Err::Failure(error::Error {
            input,
            code: error::ErrorKind::AlphaNumeric,
        })),
        type_format => Ok((remaining, type_format)),
    }
}
