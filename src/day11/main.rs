use std::collections::VecDeque;
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
pub struct Cavern(Vec<Row>);

impl Cavern {
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
        let neighbor_ixs = [
            (x - 1, y - 1),
            (x - 1, y),
            (x - 1, y + 1),
            (x, y - 1),
            (x, y + 1),
            (x + 1, y - 1),
            (x + 1, y),
            (x + 1, y + 1),
        ];

        neighbor_ixs
            .into_iter()
            .flat_map(|(nx, ny)| self.get(nx, ny).map(|n| (nx, ny, n)))
    }

    pub fn step(&mut self) -> usize {
        // let mut new_grid = self.clone();

        // Increase them all by one, make queue of flashes
        let mut queue = VecDeque::new();
        for (x, row) in self.0.iter_mut().enumerate() {
            for (y, value) in row.0.iter_mut().enumerate() {
                *value += 1;
                if *value > 9 {
                    queue.push_back((x, y));
                }
            }
        }

        let mut flashes = 0;
        while let Some((x, y)) = queue.pop_front() {
            let value = self.0[x].0[y];
            match value {
                // This one already flashed
                0 => continue,
                v if v > 9 => (),
                v => panic!("Unexpected value {v}"),
            }

            // It flashes now
            self.0[x].0[y] = 0;
            flashes += 1;

            let neighbors: Vec<_> = self.neighbors(x as isize, y as isize).collect();

            for (nx, ny, n) in neighbors {
                if n == 0 {
                    // This neighbor already flashed and reset, don't increase
                    continue;
                }

                let loc = &mut self.0[nx as usize].0[ny as usize];
                assert_eq!(*loc, n);
                *loc += 1;
                if *loc > 9 {
                    // This neighbor is now going to flash, add to queue
                    queue.push_back((nx as usize, ny as usize));
                }
            }
        }

        flashes
    }

    pub fn steps(&mut self, n: usize) -> usize {
        let mut flashes = 0;
        for _ in 0..n {
            flashes += self.step();
        }

        flashes
    }

    /// Step forward until all octopi are synchronized. Returns the number of steps taken.
    pub fn synchronize(&mut self) -> usize {
        let octopi_count = self.0.iter().map(|r| r.0.len()).sum::<usize>();
        for step in 1.. {
            let flashes = self.step();
            if flashes == octopi_count {
                return step;
            }
        }

        unreachable!()
    }
}

impl FromIterator<Row> for Cavern {
    fn from_iter<T: IntoIterator<Item = Row>>(iter: T) -> Self {
        Cavern(iter.into_iter().collect())
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day11.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let file = File::open(args.input).unwrap();
    let buf = BufReader::new(file);
    let mut octopi: Cavern = parse::buffer(buf).unwrap();

    let mut steps = 100;
    let flashes = octopi.steps(steps);
    println!("Flashed {flashes} times");

    steps += octopi.synchronize();
    println!("Synchronized after {steps} steps");
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE_SMALL: &str = r###"
        11111
        19991
        19191
        19991
        11111
    "###;

    const EXAMPLE_SMALL_1: &str = r###"
        34543
        40004
        50005
        40004
        34543
    "###;

    const EXAMPLE_SMALL_2: &str = r###"
        45654
        51115
        61116
        51115
        45654
    "###;

    #[test]
    fn test_basic() {
        let mut octopi: Cavern = parse::buffer(EXAMPLE_SMALL.as_bytes()).unwrap();
        assert_eq!(octopi.0.len(), 5);

        let flashed = octopi.step();
        assert_eq!(flashed, 9);
        let expected: Cavern = parse::buffer(EXAMPLE_SMALL_1.as_bytes()).unwrap();
        assert_eq!(octopi, expected);

        let flashed = octopi.step();
        assert_eq!(flashed, 0);
        let expected: Cavern = parse::buffer(EXAMPLE_SMALL_2.as_bytes()).unwrap();
        assert_eq!(octopi, expected);
    }

    const EXAMPLE: &str = r###"
        5483143223
        2745854711
        5264556173
        6141336146
        6357385478
        4167524645
        2176841721
        6882881134
        4846848554
        5283751526
    "###;

    const EXAMPLE_STEP_10: &str = r###"
        0481112976
        0031112009
        0041112504
        0081111406
        0099111306
        0093511233
        0442361130
        5532252350
        0532250600
        0032240000
    "###;

    const EXAMPLE_STEP_20: &str = r###"
        3936556452
        5686556806
        4496555690
        4448655580
        4456865570
        5680086577
        7000009896
        0000000344
        6000000364
        4600009543
    "###;

    #[test]
    fn test_flashing() {
        let mut octopi: Cavern = parse::buffer(EXAMPLE.as_bytes()).unwrap();
        assert_eq!(octopi.0.len(), 10);

        let mut flashed = octopi.steps(10);
        assert_eq!(flashed, 204);
        let expected: Cavern = parse::buffer(EXAMPLE_STEP_10.as_bytes()).unwrap();
        assert_eq!(octopi, expected);

        flashed += octopi.steps(10);
        let expected: Cavern = parse::buffer(EXAMPLE_STEP_20.as_bytes()).unwrap();
        assert_eq!(octopi, expected);

        // Go to 100
        flashed += octopi.steps(80);
        assert_eq!(flashed, 1656);

        let steps = 100 + octopi.synchronize();
        assert_eq!(steps, 195);
    }
}
