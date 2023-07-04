use std::fs::File;
use std::io::BufReader;
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::anyhow;
use clap::Parser;
use log::debug;

use adventofcode2021::parse;

pub struct Targeting {
    xs: RangeInclusive<i64>,
    ys: RangeInclusive<i64>,
}

impl Targeting {
    pub fn max_y(&self) -> i64 {
        // It will come back down with the same y-velocity as it left, with a
        // perfect mirroring of y-coordinates. So it will have a point at y=0,
        // with the next step taking a differential of its initial velocity + 1.
        // So to be in the expected range, initial_velocity + 1 must be within the range;
        // maximized, that means initial_velocity = range.start - 1.

        let initial_velocity = self.ys.start().abs() - 1;
        // It will go a height of vy + (vy-1) + (vy-2) + ... + 1 + 0, or vy * (vy + 1) / 2.

        let height = initial_velocity * (initial_velocity + 1) / 2;

        dbg!(initial_velocity, height);

        height
    }
}

impl FromStr for Targeting {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        let prefix = "target area: x=";
        if !trimmed.starts_with("target area: x=") {
            return Err(anyhow!("Invalid targeting string: {s}"));
        }

        let (p1, p2) = trimmed
            .trim_start_matches(prefix)
            .split_once(", y=")
            .ok_or_else(|| anyhow!("Invalid targeting string, xy not found: {s}"))?;

        let (xs1, xs2) = p1
            .split_once("..")
            .ok_or_else(|| anyhow!("Invalid targeting string, x range not found: {p1}"))?;
        let x1: i64 = xs1.parse()?;
        let x2: i64 = xs2.parse()?;

        let (ys1, ys2) = p2
            .split_once("..")
            .ok_or_else(|| anyhow!("Invalid targeting string, y range not found: {p2}"))?;
        let y1: i64 = ys1.parse()?;
        let y2: i64 = ys2.parse()?;

        Ok(Self {
            xs: x1..=x2,
            ys: y1..=y2,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day17.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let s = std::fs::read_to_string(&args.input).unwrap();
    let target = Targeting::from_str(&s).unwrap();
    let height = target.max_y();
    println!("Found height {height}");
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE: &str = r###"target area: x=20..30, y=-10..-5"###;

    #[test]
    fn test_basic() {
        let target = Targeting::from_str(EXAMPLE).unwrap();
        assert_eq!(target.xs, 20..=30);
        assert_eq!(target.ys, -10..=-5);

        assert_eq!(target.max_y(), 45);
    }
}
