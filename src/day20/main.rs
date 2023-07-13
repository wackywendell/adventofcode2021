use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use clap::Parser;
use log::debug;

use adventofcode2021::parse;

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
    let file = File::open(args.input).unwrap();
    let buf = BufReader::new(file);
    let nums: Vec<i64> = parse::buffer(buf).unwrap();

    println!("Found {length} lines: {nums:?}", length = nums.len());
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE: &str = r###"
        2
        3
        5
        7
        9
    "###;

    #[test]
    fn test_basic() {
        let nums: Vec<i64> = parse::buffer(EXAMPLE.as_bytes()).unwrap();
        assert_eq!(nums, vec![2, 3, 5, 7, 9]);
    }
}
