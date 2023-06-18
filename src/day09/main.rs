use std::collections::HashSet;
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

    /// Returns an iterator over the neighbors of the given location
    pub fn neighbors(&self, x: isize, y: isize) -> impl Iterator<Item = (isize, isize, u8)> + '_ {
        let neighbor_ixs = [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)];

        neighbor_ixs
            .into_iter()
            .flat_map(|(nx, ny)| self.get(nx, ny).map(|n| (nx, ny, n)))
    }

    /// Returns a list of (x, y, value) tuples for all the minima in the grid.
    pub fn minima(&self) -> Vec<(usize, usize, u8)> {
        let mut points = Vec::new();
        for (x, row) in self.0.iter().enumerate() {
            for (y, &value) in row.0.iter().enumerate() {
                if self
                    .neighbors(x as isize, y as isize)
                    .all(|(_, _, n)| n > value)
                {
                    points.push((x, y, value));
                }
            }
        }

        points
    }

    /// Returns the sum of the risk levels of all the minima in the grid
    pub fn risk_sum(&self) -> i64 {
        self.minima().iter().map(|&(_, _, v)| v as i64 + 1).sum()
    }

    pub fn basin_sizes(&self) -> Vec<usize> {
        let minima = self.minima();
        let mut sizes: Vec<usize> = minima.iter().map(|_| 0).collect();

        for (&(mx, my, mv), size) in minima.iter().zip(sizes.iter_mut()) {
            let mut visited = HashSet::new();
            let mut queue = vec![(mx as isize, my as isize, mv)];
            while let Some((x, y, v)) = queue.pop() {
                if v == 9 || visited.contains(&(x, y)) {
                    continue;
                }
                visited.insert((x, y));

                *size += 1;

                let nbrs: Vec<_> = self.neighbors(x, y).collect();

                queue.extend(nbrs);
            }
        }

        sizes
    }

    pub fn basin_max_product(&self) -> i64 {
        let mut sizes = self.basin_sizes();
        sizes.sort_unstable();

        sizes.iter().rev().take(3).map(|&n| n as i64).product()
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

    println!("Part 2: {}", grid.basin_max_product());
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

    #[test]
    fn test_basins() {
        let grid: Grid = parse::buffer(EXAMPLE.as_bytes()).unwrap();
        let sizes = grid.basin_sizes();
        assert_eq!(sizes, vec![3, 9, 14, 9]);
        assert_eq!(grid.basin_max_product(), 1134);
    }
}
