use std::any::TypeId;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alphanumeric0, char},
    error::{self, context},
    multi::many0,
    sequence::delimited,
    IResult,
};

use super::InputFormatToken;

impl<'a> InputFormatToken<'a> {
    fn type_from_str(text: &'a str) -> std::io::Result<Self> {
        match text {
            "i32" => Ok(Self::Type(TypeId::of::<i32>())),
            "u32" => Ok(Self::Type(TypeId::of::<u32>())),
            "f32" => Ok(Self::Type(TypeId::of::<f32>())),
            "i64" => Ok(Self::Type(TypeId::of::<i64>())),
            "u64" => Ok(Self::Type(TypeId::of::<u64>())),
            "f64" => Ok(Self::Type(TypeId::of::<f64>())),
            "string" => Ok(Self::Type(TypeId::of::<String>())),
            "" => Ok(Self::GenericType),
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
    alt((type_format, text))(input)
}

fn text(input: &str) -> IResult<&str, InputFormatToken> {
    let (remaining, text) = alt((
        tag("{{"),
        tag("}}"),
        take_until("{{"),
        take_until("}}"),
        take_until("{"),
        take_until("}"),
    ))(input)?;
    let text_token = InputFormatToken::Text(match text {
        "{{" => "{",
        "}}" => "}",
        text => {
            if text.contains("}") {
                return Err(nom::Err::Failure(error::Error {
                    input: text,
                    code: error::ErrorKind::Tag,
                }));
            }
            text
        }
    });
    return Ok((remaining, text_token));
}

fn type_format(input: &str) -> IResult<&str, InputFormatToken> {
    let mut type_parser = context("input tag", delimited(char('{'), alphanumeric0, char('}')));
    let (remaining, type_name) = type_parser(input)?;
    return match InputFormatToken::type_from_str(type_name) {
        Ok(type_token) => Ok((remaining, type_token)),
        Err(_) => Err(nom::Err::Failure(error::Error {
            input: type_name,
            code: error::ErrorKind::Tag,
        })),
    };
}
