use std::ops::Deref;

pub use nom::branch::alt;
pub use nom::bytes::complete::tag;
pub use nom::character::complete::{char, digit1, one_of};
pub use nom::combinator::{all_consuming, map, map_res, opt, recognize, value};
// #[cfg(not(debug_assertions))]
// pub use nom::error::Error as NomError;
// #[cfg(debug_assertions)]
pub use nom::error::VerboseError as NomError;
pub use nom::multi::separated_list1;
pub use nom::multi::{many0, many1};
pub use nom::sequence::{delimited, pair, preceded, tuple};

use nom::error::convert_error as nom_convert_error;

pub type ErrorRef<'a> = nom::Err<NomError<&'a str>>;
pub type Error = nom::Err<NomError<String>>;

pub type IResult<'a, V> = Result<(&'a str, V), nom::Err<NomError<&'a str>>>;

pub fn simplify<'a, V>(input: &'a str, r: Result<(&'a str, V), ErrorRef>) -> anyhow::Result<V> {
    match r {
        Ok((_, v)) => Ok(v),
        Err(nom::Err::Error(e)) => Err(convert_error(input, e)),
        Err(nom::Err::Failure(e)) => Err(convert_error(input, e)),
        Err(nom::Err::Incomplete(_)) => Err(anyhow::anyhow!("Incomplete input")),
    }
}

// #[cfg(debug_assertions)]
pub fn convert_error<I: Deref<Target = str>>(
    input: I,
    e: nom::error::VerboseError<I>,
) -> anyhow::Error {
    anyhow::anyhow!("Error parsing: {}", nom_convert_error(input, e))
}

// #[cfg(not(debug_assertions))]
#[allow(dead_code)]
fn convert_simple_error<
    'i,
    I: Deref<Target = str> + std::fmt::Debug + std::fmt::Display + Send + Sync + 'i,
>(
    input: I,
    e: nom::error::Error<I>,
) -> anyhow::Error {
    anyhow::anyhow!("Error parsing input '{}': {}", input, e)
}

// Matches whitespace: ' ', '\n'
pub fn ws(input: &str) -> IResult<&str> {
    recognize(many0(one_of(" \n")))(input)
}

// Matches a newline with optional whitespace after it
pub fn newline_ws(input: &str) -> IResult<&str> {
    recognize(pair(char('\n'), many0(char(' '))))(input)
}

// Matches 0 or more newlines
pub fn newlines0(input: &str) -> IResult<&str> {
    recognize(many0(newline_ws))(input)
}
// Matches 1 or more newlines
pub fn newlines1(input: &str) -> IResult<&str> {
    recognize(many1(newline_ws))(input)
}

// Matches an integer
pub fn int(input: &str) -> IResult<i64> {
    map_res(recognize(pair(opt(char('-')), digit1)), str::parse::<i64>)(input)
}
