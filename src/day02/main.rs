use std::fs::File;
use std::io::BufReader;
use std::ops::Add;
use std::path::PathBuf;
use std::str::FromStr;

use adventofcode2021::parse;
use anyhow::anyhow;
use clap::Parser;
use log::debug;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Command {
    depth: i64,
    forward: i64,
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (d, n) = s.split_once(' ').ok_or(anyhow!("No space in '{s}'"))?;
        let n: i64 = str::parse(n)?;

        let (depth, forward) = match d {
            "forward" => (0, n),
            "down" => (n, 0),
            "up" => (-n, 0),
            _ => return Err(anyhow!("Unexpected direction {d}")),
        };

        Ok(Command { depth, forward })
    }
}

impl Add<Command> for Command {
    type Output = Command;

    fn add(self, rhs: Command) -> Self::Output {
        Command {
            depth: self.depth + rhs.depth,
            forward: self.forward + rhs.forward,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day02.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let file = File::open(args.input).unwrap();
    let buf = BufReader::new(file);

    let directions: Vec<Command> = parse::buffer(buf).unwrap();
    let sum: Command = directions
        .iter()
        .copied()
        .reduce(Command::add)
        .unwrap_or_default();

    let mul = sum.depth * sum.forward;

    println!("Found {mul}");
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    static EXAMPLE: &str = r###"
        forward 5
        down 5
        forward 8
        up 3
        down 8
        forward 2
    "###;

    #[test]
    fn test_parse() {
        let directions: Vec<Command> = parse::buffer(EXAMPLE.as_bytes()).unwrap();
        let sum: Command = directions
            .iter()
            .copied()
            .reduce(Command::add)
            .unwrap_or_default();

        assert_eq!(
            sum,
            Command {
                depth: 10,
                forward: 15
            }
        )
    }
}
