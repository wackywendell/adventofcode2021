use std::collections::VecDeque;
use std::iter::repeat;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use log::debug;

const REFRESH: u8 = 7;
const INITIAL: u8 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FishSchool {
    fish: VecDeque<u64>,
}

impl FromStr for FishSchool {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ns = s
            .split(',')
            .map(str::parse)
            .collect::<Result<Vec<u8>, _>>()?;

        Ok(FishSchool::from_iter(ns))
    }
}

impl FromIterator<u8> for FishSchool {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        let mut fish = VecDeque::from_iter(repeat(0).take((REFRESH + INITIAL) as usize));
        for i in iter {
            fish[i as usize] += 1;
        }

        FishSchool { fish }
    }
}

impl FishSchool {
    pub fn step(&mut self) {
        let birthing = self.fish.pop_front().unwrap();
        // refresh
        self.fish[REFRESH as usize - 1] += birthing;
        // babies
        self.fish.push_back(birthing);
    }

    pub fn total(&self) -> u64 {
        self.fish.iter().sum()
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day06.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let s = std::fs::read_to_string(args.input).unwrap();
    let mut school: FishSchool = s.parse().unwrap();

    for _ in 0..80 {
        school.step();
    }

    println!("Total (80 days):  {}", school.total());

    for _ in 80..256 {
        school.step();
    }
    println!("Total (256 days): {}", school.total());
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE: &str = r###"
        3,4,3,1,2
    "###;

    #[test]
    fn test_basic() {
        let mut school: FishSchool = EXAMPLE.trim().parse().unwrap();
        for _ in 0..18 {
            school.step();
            println!("{:?}", school.fish);
        }
        assert_eq!(school.total(), 26);
        for _ in 18..80 {
            school.step();
        }
        assert_eq!(school.total(), 5934);

        for _ in 80..256 {
            school.step();
        }
        assert_eq!(school.total(), 26984457539);
    }
}
