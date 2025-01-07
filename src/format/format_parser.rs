use std::any::{Any, TypeId};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alphanumeric0, char},
    error::{self, context},
    multi::many0,
    sequence::delimited,
    IResult,
};

#[derive(Debug, PartialEq, Eq)]
pub enum InputFormatToken<'a> {
    Text(&'a str),
    Type(TypeId),
    GenericType,
}

impl<'a> InputFormatToken<'a> {
    pub(super) fn typed<T: ?Sized + Any>() -> Self {
        Self::Type(TypeId::of::<T>())
    }

    fn type_from_name(text: &str) -> std::io::Result<Self> {
        match text {
            "" => Ok(Self::GenericType),
            "string" => Ok(Self::typed::<String>()),
            "i8" => Ok(Self::typed::<i8>()),
            "u8" => Ok(Self::typed::<u8>()),
            "i16" => Ok(Self::typed::<i16>()),
            "u16" => Ok(Self::typed::<u16>()),
            "i32" => Ok(Self::typed::<i32>()),
            "u32" => Ok(Self::typed::<u32>()),
            "f32" => Ok(Self::typed::<f32>()),
            "i64" => Ok(Self::typed::<i64>()),
            "u64" => Ok(Self::typed::<u64>()),
            "f64" => Ok(Self::typed::<f64>()),
            text => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("type {:?} is not accepted for format input", text),
            )),
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
    alt((type_placeholder_token, text_token))(input)
}

fn text_token(input: &str) -> IResult<&str, InputFormatToken> {
    let (remaining, text) = alt((
        tag("{{"),
        tag("}}"),
        take_until("{{"),
        take_until("}}"),
        take_until("{"),
        take_until("}"),
    ))(input)?;
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

fn type_placeholder_token(input: &str) -> IResult<&str, InputFormatToken> {
    let mut type_parser = context("input tag", delimited(char('{'), alphanumeric0, char('}')));
    let (remaining, type_name) = type_parser(input)?;
    return match InputFormatToken::type_from_name(type_name) {
        Ok(type_token) => Ok((remaining, type_token)),
        Err(_) => Err(nom::Err::Failure(error::Error {
            input: type_name,
            code: error::ErrorKind::Tag,
        })),
    };
}
