use core::str::FromStr;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use clap::Parser;
use log::debug;
use parse_display::Display;

use adventofcode2021::parse;

use nom::{
    branch::alt,
    character::complete::{char, digit1},
    combinator::{complete, map},
    sequence::tuple,
    IResult,
};

fn parse_int(input: &str) -> IResult<&str, i64> {
    let (input, digits) = digit1(input)?;
    let n = digits.parse().unwrap();
    Ok((input, n))
}

fn parse_snailfish_pair(input: &str) -> IResult<&str, SnailfishNumber> {
    let (input, (_, a, _, b, _)) = tuple((
        char('['),
        parse_snailfish,
        char(','),
        parse_snailfish,
        char(']'),
    ))(input)?;

    Ok((input, SnailfishNumber::Pair(Box::new(a), Box::new(b))))
}

fn parse_snailfish(input: &str) -> IResult<&str, SnailfishNumber> {
    alt((
        map(parse_int, SnailfishNumber::Number),
        parse_snailfish_pair,
    ))(input)
}

#[derive(Display, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SnailfishNumber {
    #[display("{0}")]
    Number(i64),
    #[display("[{0},{1}]")]
    Pair(Box<SnailfishNumber>, Box<SnailfishNumber>),
}

impl SnailfishNumber {
    pub fn reduce(self) -> Self {
        loop {
        if let Some((_, s, _)) = self.explode_recursive(3) {
            return s;
        }
    }

    fn add_left(self, n: i64) -> Self {
        match self {
            SnailfishNumber::Number(n2) => SnailfishNumber::Number(n2 + n),
            SnailfishNumber::Pair(a, b) => SnailfishNumber::Pair(Box::new(a.add_left(n)), b),
        }
    }

    fn add_right(self, n: i64) -> Self {
        match self {
            SnailfishNumber::Number(n2) => SnailfishNumber::Number(n2 + n),
            SnailfishNumber::Pair(a, b) => SnailfishNumber::Pair(a, Box::new(b.add_right(n))),
        }
    }

    fn explode_recursive(self, n: usize) -> Option<(i64, SnailfishNumber, i64)> {
        let (a, b) = match (n, self) {
            (_, SnailfishNumber::Number(n)) => return None,
            (0, SnailfishNumber::Pair(a, b)) => {
                let a = match *a {
                    SnailfishNumber::Number(n) => n,
                    p => panic!("Expected number, got {p}"),
                };
                let b = match *b {
                    SnailfishNumber::Number(n) => n,
                    p => panic!("Expected number, got {p}"),
                };

                return Some((a, From::from(0), b));
            }
            (_, SnailfishNumber::Pair(a, b)) => (a, b),
        };

        if let Some((l, a2, r)) = a.clone().explode_recursive(n - 1) {
            let b2 = b.add_left(r);
            return Some((l, From::from((a2, b2)), 0));
        }

        if let Some((l, b2, r)) = b.explode_recursive(n - 1) {
            let a2 = a.add_right(l);
            return Some((0, From::from((a2, b2)), r));
        }

        None
    }
}

impl From<i64> for SnailfishNumber {
    fn from(n: i64) -> Self {
        SnailfishNumber::Number(n)
    }
}

impl<A: Into<SnailfishNumber>, B: Into<SnailfishNumber>> From<(A, B)> for SnailfishNumber {
    fn from((a, b): (A, B)) -> Self {
        SnailfishNumber::Pair(Box::new(a.into()), Box::new(b.into()))
    }
}

impl FromStr for SnailfishNumber {
    type Err = nom::Err<nom::error::Error<String>>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match complete(parse_snailfish)(s) {
            Ok(("", n)) => Ok(n),
            Err(e) => Err(e.to_owned()),
            _ => unreachable!(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day18.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let file = File::open(args.input).unwrap();
    let buf = BufReader::new(file);
    let nums: Vec<i64> = parse::buffer(buf).unwrap();

    println!("Found {length} lines: {nums:?}", length = nums.len());
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE: &str = r###"
        [[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]
        [7,[[[3,7],[4,3]],[[6,3],[8,8]]]]
        [[2,[[0,8],[3,4]]],[[[6,7],1],[7,[1,6]]]]
        [[[[2,4],7],[6,[0,5]]],[[[6,8],[2,8]],[[2,1],[4,5]]]]
        [7,[5,[[3,8],[1,4]]]]
        [[2,[2,2]],[8,[8,1]]]
        [2,9]
        [1,[[[9,3],9],[[9,0],[0,7]]]]
        [[[5,[7,4]],7],1]
        [[[[4,2],2],6],[8,7]]
    "###;

    #[test]
    fn test_parse() {
        let n: SnailfishNumber = "1".parse().unwrap();
        assert_eq!(n, SnailfishNumber::Number(1));

        let n: SnailfishNumber = "[1,2]".parse().unwrap();
        assert_eq!(
            n,
            SnailfishNumber::Pair(
                Box::new(SnailfishNumber::Number(1)),
                Box::new(SnailfishNumber::Number(2))
            )
        );
        assert_eq!(n, SnailfishNumber::from((1, 2)));

        let n: SnailfishNumber = "[1,[2,3]]".parse().unwrap();
        assert_eq!(n, SnailfishNumber::from((1, (2, 3))));
        let n: SnailfishNumber = "[[1,2],3]".parse().unwrap();
        assert_eq!(n, SnailfishNumber::from(((1, 2), 3)));

        let _n: SnailfishNumber = "[[4,5],[2,6]]".parse().unwrap();

        let _n: SnailfishNumber = "[[[4,5],[2,6]],[9,5]]".parse().unwrap();

        let n: SnailfishNumber = "[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]".parse().unwrap();
        assert_eq!(
            n,
            SnailfishNumber::from((((0, (4, 5)), (0, 0)), (((4, 5), (2, 6)), (9, 5))))
        );
    }

    #[test]
    fn test_basic() {
        let nums: Vec<SnailfishNumber> = parse::buffer(EXAMPLE.as_bytes()).unwrap();
        assert_eq!(nums.len(), 10);
    }
}
