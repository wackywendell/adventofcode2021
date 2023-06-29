use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::anyhow;
use clap::Parser;
use log::debug;

use adventofcode2021::parse;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Row(Vec<i8>);
impl FromStr for Row {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let nums: Result<Vec<i8>, anyhow::Error> = s
            .trim()
            .chars()
            .map(|s| {
                s.to_digit(10)
                    .map(|n| n as i8)
                    .ok_or_else(|| anyhow!("Invalid digit: {s}"))
            })
            .collect();

        Ok(Self(nums?))
    }
}

impl From<Row> for Vec<i8> {
    fn from(value: Row) -> Self {
        value.0
    }
}

impl From<Vec<i8>> for Row {
    fn from(v: Vec<i8>) -> Self {
        Self(v)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    // Size, inclusive - this is a position in size
    size: (isize, isize),
    pos: HashMap<(isize, isize), i8>,
}

impl<I: Into<Vec<i8>>> FromIterator<I> for Grid {
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let mut pos = HashMap::new();
        let mut size = (0, 0);
        for (y, row) in iter
            .into_iter()
            .map(Into::into)
            .filter(|r| !r.is_empty())
            .enumerate()
        {
            size.1 = size.1.max(row.len() as isize - 1);
            for (x, val) in row.into_iter().enumerate() {
                pos.insert((x as isize, y as isize), val);
            }
            size.0 = y as isize;
        }
        Self { size, pos }
    }
}

impl Grid {
    pub fn shortest_diagonal(&self) -> i64 {
        if self.pos.len() <= 1 {
            return self.pos.get(&self.size).copied().unwrap_or_default() as i64;
        }

        let (sx, sy) = self.size;
        self.shortest_path((0, 0), (sx, sy)).unwrap()
    }

    pub fn shortest_path(&self, start: (isize, isize), end: (isize, isize)) -> Option<i64> {
        let mut visited = HashSet::new();
        // Elements are (risk, pos)
        let mut queue = BinaryHeap::new();
        // let risk0 = self.pos.get(&start).copied()? as i64;
        // Starting position is never entered
        queue.push((Reverse(0), start));
        while let Some((Reverse(risk), pos)) = queue.pop() {
            if pos == end {
                return Some(risk);
            }
            if visited.contains(&pos) {
                continue;
            }

            visited.insert(pos);
            for dir in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let next = (pos.0 + dir.0, pos.1 + dir.1);
                if let Some(r) = self.pos.get(&next).copied() {
                    queue.push((Reverse(r as i64 + risk), next));
                }
            }
        }
        None
    }

    pub fn multiply(self, (xtimes, ytimes): (isize, isize)) -> Self {
        let mut pos = HashMap::new();
        let (w, h) = (self.size.0 + 1, self.size.1 + 1);

        for ((x, y), val) in self.pos {
            for nx in 0..xtimes {
                for ny in 0..ytimes {
                    let r: i8 = (val - 1 + nx as i8 + ny as i8) % 9 + 1;
                    pos.insert((x + nx * w, y + ny * h), r);
                }
            }
        }

        Self {
            size: (w * xtimes - 1, h * ytimes - 1),
            pos,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day15.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let file = File::open(args.input).unwrap();
    let buf = BufReader::new(file);
    let grid: Grid = parse::buffer::<_, Row, _>(buf).unwrap();

    let risk = grid.shortest_diagonal();
    println!("Found path of risk {risk}");

    let big_grid = grid.multiply((5, 5));
    let risk = big_grid.shortest_diagonal();
    println!("Found path of risk {risk} in big grid");
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE: &str = r###"
        1163751742
        1381373672
        2136511328
        3694931569
        7463417111
        1319128137
        1359912421
        3125421639
        1293138521
        2311944581
    "###;

    #[test]
    fn test_basic() {
        let grid = parse::buffer::<_, Row, Grid>(EXAMPLE.as_bytes()).unwrap();
        let risk = grid.shortest_diagonal();
        assert_eq!(risk, 40);
    }

    #[test]
    fn test_multiply() {
        let grid = parse::buffer::<_, Row, Grid>("8".as_bytes()).unwrap();
        let grid = grid.multiply((5, 5));
        assert_eq!(grid.pos.get(&(0, 0)).copied(), Some(8));
        assert_eq!(grid.pos.get(&(0, 1)).copied(), Some(9));
        assert_eq!(grid.pos.get(&(0, 2)).copied(), Some(1));
        assert_eq!(grid.pos.get(&(1, 1)).copied(), Some(1));

        let expected_str = "89123\n91234\n12345\n23456\n34567";
        let expected = parse::buffer::<_, Row, Grid>(expected_str.as_bytes()).unwrap();
        assert_eq!(grid, expected);
    }

    #[test]
    fn test_big_path() {
        let grid = parse::buffer::<_, Row, Grid>(EXAMPLE.as_bytes()).unwrap();
        let grid = grid.multiply((5, 5));
        let risk = grid.shortest_diagonal();
        assert_eq!(risk, 315);
    }
}
