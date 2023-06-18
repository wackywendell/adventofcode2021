use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::anyhow;
use clap::Parser;
use log::debug;

use adventofcode2021::parse;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row(Vec<u8>);

impl FromStr for Row {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut row = Vec::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '0'..='9' => {
                    row.push(c.to_digit(10).unwrap() as u8);
                }

                // '#' => row.push(1),
                _ => return Err(anyhow!("Invalid character: {c}")),
            }
        }
        Ok(Row(row))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid(Vec<Row>);

impl Grid {
    pub fn get(&self, x: isize, y: isize) -> Option<u8> {
        if x < 0 || y < 0 {
            return None;
        }
        self.0
            .get(x as usize)
            .and_then(|row| row.0.get(y as usize).copied())
    }

    /// Returns a list of (x, y, value) tuples for all the minima in the grid.
    pub fn minima(&self) -> Vec<(usize, usize, u8)> {
        let mut points = Vec::new();
        for (x, row) in self.0.iter().enumerate() {
            for (y, &value) in row.0.iter().enumerate() {
                let neighbor_ixs = [
                    (x as isize - 1, y as isize),
                    (x as isize + 1, y as isize),
                    (x as isize, y as isize - 1),
                    (x as isize, y as isize + 1),
                ];

                let values: Vec<u8> = neighbor_ixs
                    .iter()
                    .flat_map(|&(x, y)| self.get(x, y))
                    .collect();
                debug!("({x}, {y}): {value} -> {values:?}");

                if neighbor_ixs
                    .iter()
                    .flat_map(|&(x, y)| self.get(x, y))
                    .all(|n| n > value)
                {
                    points.push((x, y, value));
                }
            }
        }

        points
    }

    pub fn risk_sum(&self) -> i64 {
        self.minima().iter().map(|&(_, _, v)| v as i64 + 1).sum()
    }
}

impl FromIterator<Row> for Grid {
    fn from_iter<T: IntoIterator<Item = Row>>(iter: T) -> Self {
        Grid(iter.into_iter().collect())
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day09.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let file = File::open(args.input).unwrap();
    let buf = BufReader::new(file);
    let grid: Grid = parse::buffer(buf).unwrap();

    println!("Part 1: {}", grid.risk_sum());
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE: &str = r###"
        2199943210
        3987894921
        9856789892
        8767896789
        9899965678
    "###;

    #[test]
    fn test_basic() {
        let grid: Grid = parse::buffer(EXAMPLE.as_bytes()).unwrap();
        assert_eq!(grid.0.len(), 5);

        let minima: Vec<u8> = grid.minima().iter().map(|&(_, _, v)| v).collect();

        assert_eq!(minima, vec![1, 0, 5, 5]);
        assert_eq!(grid.risk_sum(), 15);
    }
}
