use adventofcode2021::parse;
use clap::Parser;
use log::debug;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub fn find_increases(depths: &[i64]) -> isize {
    let mut count = 0;
    let mut prev = depths.first().copied().unwrap_or_default();

    for &n in &depths[1..] {
        if prev < n {
            count += 1;
        }
        prev = n;
    }

    count
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day01.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let file = File::open(args.input).unwrap();
    let buf = BufReader::new(file);
    let ns: Vec<i64> = parse::buffer(buf).unwrap();

    let count = find_increases(&ns);

    println!("Found {count} increases");
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE: &str = r###"
        199
        200
        208
        210
        200
        207
        240
        269
        260
        263    
    "###;

    #[test]
    fn test_thing() {
        let ns: Vec<i64> = parse::buffer(EXAMPLE.as_bytes()).unwrap();

        let count = find_increases(&ns);

        assert_eq!(count, 7);
    }
}
