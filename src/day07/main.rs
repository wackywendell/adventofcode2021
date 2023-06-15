use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use log::debug;

pub struct Crabs {
    pub locations: Vec<u16>,
}

impl FromStr for Crabs {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let locations = s
            .split(',')
            .map(str::parse)
            .collect::<Result<Vec<u16>, _>>()?;
        Ok(Crabs { locations })
    }
}

impl Crabs {
    pub fn shortest(&self) -> (u16, u64) {
        let mut ns = self.locations.clone();
        ns.sort();

        let mid = (ns.len() + 1) / 2;
        let mid = ns[mid];
        let diff_total = ns.iter().map(|&n| n.abs_diff(mid) as u64).sum();

        (mid, diff_total)
    }

    // fuel cost for distance d is d(d+1)/2
    // for a crab at position p with goal at x, the fuel cost is
    //     (|p-x|)(|p-x|+1)/2
    //     (|p-x|)^2 + |p-x|)/2
    //     ((p^2 - 2px + x^2) + |p-x|/2
    // So total is
    //     Sum_p (p^2 - 2px + x^2 + |p - x|)/2
    // Pulling out a constant C gives
    //     C + Sum_p (x^2 - 2px + |p-x|)/2
    // Setting derivative to 0
    //     Sum_p (x - p + 1/2 sgn(p-x)) = 0
    //     N x = Sum_p p
    //     x = <p> ± ½
    // So the optimal position is the average of the positions
    pub fn shortest_linear(&self) -> (u16, u64) {
        let sum: i64 = self.locations.iter().map(|&n| n as i64).sum();

        let n = self.locations.len() as i64;
        // avg rounded down
        let avg = (sum / n) as u16;

        let fuel_func = |x: u16| {
            self.locations
                .iter()
                .map(|&p| {
                    let d = (p as i64 - x as i64).abs();
                    d * (d + 1) / 2
                })
                .sum::<i64>() as u64
        };

        let locs: [u16; 3] = [avg - 1, avg, avg + 1];
        locs.iter()
            .map(|&x| (x, fuel_func(x)))
            .min_by_key(|&(_, fuel)| fuel)
            .unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day07.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let s = std::fs::read_to_string(args.input).unwrap();
    let crabs = Crabs::from_str(s.trim()).unwrap();

    let (mid, fuel) = crabs.shortest();
    println!("Shortest position {mid} requires {fuel:?}");

    // 99540639 too high
    let (mid, fuel) = crabs.shortest_linear();
    println!("Shortest position {mid} with linear ramp requires {fuel:?}");
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE: &str = r###"
        16,1,2,0,4,2,7,1,2,14
    "###;

    #[test]
    fn test_basic() {
        let crabs = Crabs::from_str(EXAMPLE.trim()).unwrap();
        let (mid, fuel) = crabs.shortest();

        assert_eq!((mid, fuel), (2, 37));
    }

    #[test]
    fn test_linear() {
        let crabs = Crabs::from_str(EXAMPLE.trim()).unwrap();
        let (mid, fuel) = crabs.shortest_linear();

        assert_eq!((mid, fuel), (5, 168));
    }
}
