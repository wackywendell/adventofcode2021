use std::fmt::Display;

use std::path::PathBuf;
use std::str::FromStr;

use bitvec::vec::BitVec;
use clap::Parser;
use log::debug;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Unexpected character {char}")]
pub struct ParseError {
    char: char,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row(BitVec);

impl FromStr for Row {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bits = BitVec::new();
        for c in s.chars() {
            match c {
                '.' => bits.push(false),
                '#' => bits.push(true),
                c => return Err(ParseError { char: c }),
            }
        }
        Ok(Self(bits))
    }
}

pub struct RowRef<'a>(&'a BitVec);

impl<'a> Display for RowRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bit in self.0 {
            write!(f, "{}", if *bit { '#' } else { '.' })?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    algo: Row,
    blank: bool,
    data: Vec<BitVec>,
}

pub fn to_chars(bits: &BitVec) -> impl Iterator<Item = char> + '_ {
    bits.iter().map(|b| if *b { '#' } else { '.' })
}

impl Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Blank: {}", if self.blank { '#' } else { '.' })?;
        writeln!(f, "Algo: {}", RowRef(&self.algo.0))?;
        for row in &self.data {
            writeln!(f, "{}", RowRef(row))?;
        }
        Ok(())
    }
}

impl FromStr for Image {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();

        let first = loop {
            match lines.next() {
                Some(line) if line.trim().is_empty() => continue,
                Some(line) => break line,
                None => return Err(ParseError { char: '\0' })?,
            }
        };
        let algo = Row::from_str(first.trim())?;

        let mut lines = lines.peekable();
        while let Some(p) = lines.peek().copied() {
            if p.trim().is_empty() {
                lines.next();
                continue;
            }
            break;
        }

        let data = lines
            .map(|line| line.trim().parse::<Row>().unwrap().0)
            .filter(|algo| !algo.is_empty())
            .collect();
        Ok(Self {
            blank: false,
            data,
            algo,
        })
    }
}

impl Image {
    pub fn pixel(&self, x: isize, y: isize) -> bool {
        if x < 0 || y < 0 {
            return self.blank;
        }

        self.data
            .get(y as usize)
            .and_then(|v| v.get(x as usize).map(|r| *r))
            .unwrap_or(self.blank)
    }

    pub fn get_value(&self, x: isize, y: isize) -> u16 {
        let mut value = 0;

        for ny in y - 1..=y + 1 {
            for nx in x - 1..=x + 1 {
                value <<= 1;
                if self.pixel(nx, ny) {
                    value |= 1;
                }
            }
        }

        value
    }

    pub fn stepped(&self, x: isize, y: isize) -> bool {
        let value = self.get_value(x, y);
        self.algo.0[value as usize]
    }

    pub fn step(&mut self) {
        let mut data = Vec::new();

        for y in -1..=(self.data.len() as isize) {
            let mut new_vec: BitVec = BitVec::new();
            for x in -1..=self.data[0].len() as isize {
                new_vec.push(self.stepped(x, y));
            }

            data.push(new_vec);
        }

        let blank_value = if self.blank { 0b111_111_111 } else { 0 };
        let blank = self.algo.0[blank_value as usize];

        self.data = data;
        self.blank = blank;
    }

    pub fn count(&self) -> usize {
        self.data.iter().map(|v| v.count_ones()).sum()
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day20.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let s = std::fs::read_to_string(&args.input).unwrap();

    let mut image: Image = s.parse().unwrap();
    debug!("Initial image {}:\n{}", image.count(), image);
    image.step();
    image.step();
    println!("After 2 steps: {}", image.count());

    for _ in 2..50 {
        image.step();
    }
    println!("After 50 steps: {}", image.count());
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    use bitvec::prelude as bv;

    #[allow(unused_imports)]
    use super::*;

    const ALGO: &str = "..#.#..#####.#.#.#.###.##.....###.##.#..###.####..#####..#....#..#..##..###..######.###...####..#..#####..##..#.#####...##.#.#..#.##..#.#......#.###.######.###.####...#.##.##..#..#..#####.....#.#....###..#.##......#.....#..#..#..##..#...##.######.####.####.#.#...#.......#..#.#.#...####.##.#......#..#...##.#.##..#...##.#.##..###.#......#.#.......#.#.#.####.###.##...#.....####.#..#..#.##.#....##..#.####....##...##..#...#......#.#.......#.......##..####..#...#.#.#...##..#.#..###..#####........#..####......#..#";

    #[test]
    fn test_algo() {
        let algo = Row::from_str("..#.###").unwrap();
        assert_eq!(algo.0.len(), 7);
        let expected: BitVec = bv::bitvec![usize, bv::Lsb0; 0, 0, 1, 0, 1, 1, 1];
        assert_eq!(algo.0, expected);

        let algo = Row::from_str(ALGO).unwrap();
        assert_eq!(algo.0.len(), 512);
    }

    const EXAMPLE: &str = r###"
        #..#.
        #....
        ##..#
        ..#..
        ..###"###;

    #[test]
    fn test_parse() {
        let image = Image::from_str(&format!("{ALGO}\n{EXAMPLE}")).unwrap();

        assert_eq!(image.algo.0.len(), 512);
        assert_eq!(image.data[0].len(), 5);
        assert_eq!(image.data.len(), 5);
    }

    #[allow(clippy::bool_assert_comparison)]
    #[test]
    fn test_step() {
        let mut image = Image::from_str(&format!("{ALGO}\n{EXAMPLE}")).unwrap();

        assert_eq!(image.pixel(0, 0), true);
        assert_eq!(image.pixel(1, 0), false);
        assert_eq!(image.pixel(2, 0), false);
        assert_eq!(image.pixel(3, 0), true);
        assert_eq!(image.pixel(4, 0), false);

        let val = image.get_value(2, 2);
        assert_eq!(val, 0b000_100_010, "{val:b}");

        let val = image.get_value(0, 0);
        assert_eq!(val, 0b000_010_010, "{val:b}");

        let expected_str_1 = r###"
            .##.##.
            #..#.#.
            ##.#..#
            ####..#
            .#..##.
            ..##..#
            ...#.#."###;
        let expected1 = Image::from_str(&format!("{ALGO}\n{expected_str_1}")).unwrap();

        // #..\n.#.\n.## -> 100_010_011
        assert_eq!(expected1.pixel(3, 4), false);
        assert_eq!(
            image.get_value(2, 3),
            0b100_010_011,
            "{val:b}",
            val = image.get_value(2, 3)
        );
        assert_eq!(image.stepped(2, 3), false);

        image.step();

        assert_eq!(image.data, expected1.data, "Got {image}");
        assert_eq!(image, expected1, "Got {image}");

        let expected_str_2 = r###"
            .......#.
            .#..#.#..
            #.#...###
            #...##.#.
            #.....#.#
            .#.#####.
            ..#.#####
            ...##.##.
            ....###.."###;

        image.step();

        let expected2 = Image::from_str(&format!("{ALGO}\n{expected_str_2}")).unwrap();
        assert_eq!(image, expected2, "Got {image}");
        assert_eq!(image.count(), 35);

        for _ in 2..50 {
            image.step();
        }
        assert_eq!(image.count(), 3351);
    }
}
