use bitvec::prelude as bits;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use log::debug;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiagnosticReport {
    observations: usize,
    summed: Vec<usize>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Observation<const N: usize>(bits::BitArray<[u16; 1], bits::Msb0>);

impl<'a, const N: usize> IntoIterator for &'a Observation<N> {
    type Item = bitvec::ptr::BitRef<'a, bitvec::ptr::Const, u16, bits::Msb0>;

    type IntoIter = std::iter::Take<bitvec::slice::Iter<'a, u16, bits::Msb0>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().take(N)
    }
}

impl<const N: usize> FromStr for Observation<N> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != N {
            return Err(anyhow::anyhow!("Length {} != {}", s.len(), N));
        }

        let mut obs = Observation(bits::BitArray::default());

        for (ix, c) in s.as_bytes().iter().rev().enumerate() {
            obs.0.set(
                ix,
                match c {
                    b'0' => false,
                    b'1' => true,
                    _ => return Err(anyhow::anyhow!("Unexpected char '{c}'")),
                },
            );
        }

        Ok(obs)
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day03.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let file = File::open(args.input).unwrap();
    let buf = BufReader::new(file);

    let line_count = buf.lines().count();
    println!("Found {line_count} lines");
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use adventofcode2021::parse;
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    static EXAMPLE: &str = r###"
        00100
        11110
        10110
        10111
        10101
        01111
        00111
        11100
        10000
        11001
        00010
        01010
    "###;

    #[test]
    fn test_parse() {
        let observations: Vec<Observation<5>> = parse::buffer(EXAMPLE.as_bytes()).unwrap();

        let first = observations[0];
        let refs: Vec<bool> = first.into_iter().map(|r| *r).collect();
    }
}
