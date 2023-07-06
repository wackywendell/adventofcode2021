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
    pub fn reduce(&mut self) {
        loop {
            if self.explode_recursive(4).is_some() {
                continue;
            }
            if self.split_recursive() {
                continue;
            }

            break;
        }
    }

    fn add_left(&mut self, n: i64) {
        match self {
            SnailfishNumber::Number(n2) => *n2 += n,
            SnailfishNumber::Pair(a, _b) => a.add_left(n),
        }
    }

    fn add_right(&mut self, n: i64) {
        match self {
            SnailfishNumber::Number(n2) => *n2 += n,
            SnailfishNumber::Pair(_a, b) => b.add_right(n),
        }
    }

    // Recursively "explode" a number.
    //
    // Returns (replacement, left, right), where:
    // - replacement is the number that should replace this one, if any
    // - left is the number that should be added to the first number to the
    //   left, if it exists
    // - right is the number that should be added to the first number to the
    //   right, if it exists
    //
    // returns None if no explode has occurred, and Some((None, 0, 0)) if the
    // explosion has completed within.
    fn explode_recursive(&mut self, n: usize) -> Option<(Option<i64>, i64, i64)> {
        match (n, self) {
            (_, SnailfishNumber::Number(_)) => None,
            (0, SnailfishNumber::Pair(a, b)) => {
                // We need to explode this pair
                let a = match a.as_mut() {
                    &mut SnailfishNumber::Number(n) => n,
                    p => panic!("Expected number, got {p}"),
                };
                let b = match b.as_mut() {
                    &mut SnailfishNumber::Number(n) => n,
                    p => panic!("Expected number, got {p}"),
                };

                Some((Some(0), a, b))
            }
            (_, SnailfishNumber::Pair(a, b)) => {
                // Recurse into a pair
                if let Some((rep, l, r)) = a.explode_recursive(n - 1) {
                    b.add_left(r);
                    if let Some(a2) = rep {
                        *a = Box::new(SnailfishNumber::from(a2));
                    }
                    return Some((None, l, 0));
                }

                if let Some((rep, l, r)) = b.explode_recursive(n - 1) {
                    a.add_right(l);
                    if let Some(b2) = rep {
                        *b = Box::new(SnailfishNumber::from(b2));
                    }
                    return Some((None, 0, r));
                }

                None
            }
        }
    }

    fn split_value(n: i64) -> SnailfishNumber {
        let half = n / 2;
        let other = n - half;
        SnailfishNumber::from((half, other))
    }

    // Split the number at most once, returning true if successful
    fn split_recursive(&mut self) -> bool {
        match *self {
            SnailfishNumber::Number(n) => {
                if n < 10 {
                    return false;
                }

                *self = SnailfishNumber::split_value(n);
                true
            }
            SnailfishNumber::Pair(ref mut a, ref mut b) => {
                if a.split_recursive() {
                    return true;
                }

                b.split_recursive()
            }
        }
    }

    pub fn add(&mut self, other: SnailfishNumber) {
        let mut temp = SnailfishNumber::from(0);
        std::mem::swap(&mut temp, self);

        *self = SnailfishNumber::from((temp, other));

        self.reduce();
    }

    pub fn sum<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let mut sum = iter
            .next()
            .unwrap_or_else(|| panic!("Cannot sum empty iterator"));

        for n in iter {
            sum.add(n);
        }

        sum
    }

    pub fn magnitude(&self) -> i64 {
        match self {
            SnailfishNumber::Number(n) => *n,
            SnailfishNumber::Pair(a, b) => 3 * a.magnitude() + 2 * b.magnitude(),
        }
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
    let nums: Vec<SnailfishNumber> = parse::buffer(buf).unwrap();
    let length = nums.len();
    let sum = SnailfishNumber::sum(nums);
    let mag = sum.magnitude();

    println!("Found {length} numbers summing to {sum} with magnitude {mag}");
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
    fn test_basic() {
        let nums: Vec<SnailfishNumber> = parse::buffer(EXAMPLE.as_bytes()).unwrap();
        assert_eq!(nums.len(), 10);
    }

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
    fn test_explode() {
        let cases = vec![
            ("[[[[[9,8],1],2],3],4]", "[[[[0,9],2],3],4]"),
            ("[7,[6,[5,[4,[3,2]]]]]", "[7,[6,[5,[7,0]]]]"),
            ("[[6,[5,[4,[3,2]]]],1]", "[[6,[5,[7,0]]],3]"),
            (
                "[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]",
                "[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]",
            ),
            (
                "[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]",
                "[[3,[2,[8,0]]],[9,[5,[7,0]]]]",
            ),
        ];

        for (input, expected) in cases {
            let mut n = SnailfishNumber::from_str(input).unwrap();
            n.explode_recursive(4);
            let expected = SnailfishNumber::from_str(expected).unwrap();
            assert_eq!(n, expected);
        }
    }

    #[test]
    fn test_reduce() {
        let input = "[[[[[4,3],4],4],[7,[[8,4],9]]],[1,1]]";
        let expected = "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]";
        let mut n = SnailfishNumber::from_str(input).unwrap();
        n.reduce();
        let expected = SnailfishNumber::from_str(expected).unwrap();
        assert_eq!(n, expected);
    }

    const ADD_EXAMPLES: [(&str, &str); 4] = [
        (
            r"[1,1]
            [2,2]
            [3,3]
            [4,4]",
            "[[[[1,1],[2,2]],[3,3]],[4,4]]",
        ),
        (
            r"[1,1]
            [2,2]
            [3,3]
            [4,4]
            [5,5]",
            "[[[[3,0],[5,3]],[4,4]],[5,5]]",
        ),
        (
            r"[1,1]
            [2,2]
            [3,3]
            [4,4]
            [5,5]
            [6,6]",
            "[[[[5,0],[7,4]],[5,5]],[6,6]]",
        ),
        (
            r"[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]
            [7,[[[3,7],[4,3]],[[6,3],[8,8]]]]
            [[2,[[0,8],[3,4]]],[[[6,7],1],[7,[1,6]]]]
            [[[[2,4],7],[6,[0,5]]],[[[6,8],[2,8]],[[2,1],[4,5]]]]
            [7,[5,[[3,8],[1,4]]]]
            [[2,[2,2]],[8,[8,1]]]
            [2,9]
            [1,[[[9,3],9],[[9,0],[0,7]]]]
            [[[5,[7,4]],7],1]
            [[[[4,2],2],6],[8,7]]",
            "[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]",
        ),
    ];

    #[test]
    fn test_add() {
        for (input, expected) in ADD_EXAMPLES {
            let nums: Vec<SnailfishNumber> = parse::buffer(input.as_bytes()).unwrap();
            let n = SnailfishNumber::sum(nums);
            let expected = SnailfishNumber::from_str(expected).unwrap();
            assert_eq!(n, expected);
        }
    }

    #[test]
    fn test_magnitude() {
        let cases: Vec<(&str, i64)> = vec![
            ("[[1,2],[[3,4],5]]", 143),
            ("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]", 1384),
            ("[[[[1,1],[2,2]],[3,3]],[4,4]]", 445),
            ("[[[[3,0],[5,3]],[4,4]],[5,5]]", 791),
            ("[[[[5,0],[7,4]],[5,5]],[6,6]]", 1137),
            (
                "[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]",
                3488,
            ),
        ];

        for (input, expected) in cases {
            let n = SnailfishNumber::from_str(input).unwrap();
            let magnitude = n.magnitude();
            assert_eq!(magnitude, expected);
        }
    }

    #[test]
    fn test_homework() {
        let input = r"
            [[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]
            [[[5,[2,8]],4],[5,[[9,9],0]]]
            [6,[[[6,2],[5,6]],[[7,6],[4,7]]]]
            [[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]
            [[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]
            [[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]
            [[[[5,4],[7,7]],8],[[8,3],8]]
            [[9,3],[[9,9],[6,[4,9]]]]
            [[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]
            [[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]";
        let nums: Vec<SnailfishNumber> = parse::buffer(input.as_bytes()).unwrap();
        let n = SnailfishNumber::sum(nums);

        assert_eq!(n.magnitude(), 4140);
    }
}
