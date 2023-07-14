pub use nom::branch::alt;
pub use nom::bytes::complete::tag;
pub use nom::character::complete::{char, digit1, one_of};
pub use nom::combinator::{all_consuming, map, map_res, opt, recognize, value};
pub use nom::error::Error as NomError;
pub use nom::multi::separated_list1;
pub use nom::multi::{many0, many1};
pub use nom::sequence::{delimited, pair, preceded, tuple};

pub type ErrorRef<'a> = nom::Err<NomError<&'a str>>;
pub type Error = nom::Err<NomError<String>>;

pub type IResult<'a, V> = nom::IResult<&'a str, V>;

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
