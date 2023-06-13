use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use log::debug;

use adventofcode2021::parse;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Line {
    pub start: (i64, i64),
    pub end: (i64, i64),
}

impl Line {
    pub fn horizontal(&self) -> bool {
        self.start.1 == self.end.1
    }

    pub fn vertical(&self) -> bool {
        self.start.0 == self.end.0
    }

    pub fn diagonal(&self) -> bool {
        (self.start.1 - self.end.1).abs() == (self.start.0 - self.end.0).abs()
    }

    pub fn points(&self) -> HashSet<(i64, i64)> {
        let (x1, x2) = (self.start.0, self.end.0);
        let (y1, y2) = (self.start.1, self.end.1);

        let sign1 = (x2 - x1).signum();
        let sign2 = (y2 - y1).signum();

        let magnitude1 = (x2 - x1).abs();
        let magnitude2 = (y2 - y1).abs();
        let magnitude = match (magnitude1, magnitude2) {
            (0, m) => m,
            (m, 0) => m,
            (m1, m2) if m1 == m2 => m1,
            _ => panic!("Not a line: {magnitude1}, {magnitude2}"),
        };

        let mut points = HashSet::new();
        for dx in 0..=magnitude {
            let x = x1 + dx * sign1;
            let y = y1 + dx * sign2;
            points.insert((x, y));
        }

        points
    }
}

impl FromStr for Line {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (start, end) = s.split_once(" -> ").ok_or(anyhow::anyhow!("No arrow"))?;
        let (s1, s2) = start.split_once(',').ok_or(anyhow::anyhow!("No comma"))?;
        let (e1, e2) = end.split_once(',').ok_or(anyhow::anyhow!("No comma"))?;

        let start = (s1.parse::<i64>()?, s2.parse::<i64>()?);
        let end = (e1.parse::<i64>()?, e2.parse::<i64>()?);

        Ok(Line { start, end })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lines {
    lines: Vec<Line>,
}

impl Lines {
    pub fn parse(buf: impl BufRead) -> anyhow::Result<Self> {
        let mut lines: Vec<Line> = parse::buffer(buf)?;

        for line in &mut lines {
            if line.start.0 > line.end.0 {
                std::mem::swap(&mut line.start, &mut line.end);
            }
        }

        lines.sort_by_key(|l| (l.start.0, l.end.0, l.start.1, l.end.1));

        Ok(Lines { lines })
    }

    pub fn all_points(&self) -> HashMap<(i64, i64), usize> {
        let mut points = HashMap::new();

        for line in &self.lines {
            for point in line.points() {
                *points.entry(point).or_default() += 1;
            }
        }

        points
    }

    pub fn overlap_count(&self) -> usize {
        self.all_points().values().map(|n| n - 1).sum()
    }

    pub fn overlaps(&self) -> usize {
        self.all_points().values().filter(|&&n| n > 1).count()
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day05.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let file = File::open(args.input).unwrap();
    let buf = BufReader::new(file);
    let lines = Lines::parse(buf).unwrap();
    let mut hvlines = lines.clone();
    hvlines.lines.retain(|l| l.horizontal() || l.vertical());

    let hv_overlaps = hvlines.overlaps();
    let overlaps = lines.overlaps();
    // 3389 is too low
    // 5432 is too high
    println!("Found {hv_overlaps} h/v overlaps, {overlaps} total");
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE: &str = r###"
        0,9 -> 5,9
        8,0 -> 0,8
        9,4 -> 3,4
        2,2 -> 2,1
        7,0 -> 7,4
        6,4 -> 2,0
        0,9 -> 2,9
        3,4 -> 1,4
        0,0 -> 8,8
        5,5 -> 8,2
    "###;

    #[test]
    fn test_basic() {
        let lines = Lines::parse(EXAMPLE.as_bytes()).unwrap();
        let mut hvlines = lines; //.clone();
        hvlines.lines.retain(|l| l.horizontal() || l.vertical());

        let all_points = hvlines.all_points();
        let x1 = all_points.keys().map(|&(x, _)| x).min().unwrap();
        let x2 = all_points.keys().map(|&(x, _)| x).max().unwrap();
        let y1 = all_points.keys().map(|&(_, y)| y).min().unwrap();
        let y2 = all_points.keys().map(|&(_, y)| y).max().unwrap();

        for y in y1..=y2 {
            let row: String = (x1..=x2)
                .map(|x| {
                    all_points
                        .get(&(x, y))
                        .map(|n| n.to_string().chars().last().unwrap())
                        .unwrap_or('.')
                })
                .collect();
            debug!("{row}");
        }

        assert_eq!(hvlines.all_points().len(), 21);
        assert_eq!(hvlines.overlaps(), 5);
    }

    #[test]
    fn test_diagonals() {
        let lines = Lines::parse(EXAMPLE.as_bytes()).unwrap();
        assert_eq!(lines.all_points().len(), 39);
        assert_eq!(lines.overlaps(), 12);
    }
}
