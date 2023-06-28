use std::collections::HashSet;
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Context};
use clap::Parser;
use log::debug;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fold {
    Horizontal(i64),
    Vertical(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instructions {
    points: HashSet<(i64, i64)>,
    folds: Vec<Fold>,
}

impl FromStr for Instructions {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut points = HashSet::new();
        let mut folds = Vec::new();
        let trimmed = s.trim();
        let mut lines = trimmed.lines();
        for line in lines.by_ref() {
            let l = line.trim();
            if l.is_empty() {
                break;
            }
            let (s1, s2) = l.split_once(',').ok_or(anyhow!("Expected comma"))?;
            let x: i64 = s1.trim().parse().context("parsing x")?;
            let y: i64 = s2.trim().parse().context("parsing y")?;
            points.insert((x, y));
        }

        for line in lines {
            let l = line.trim();
            if l.is_empty() {
                continue;
            }

            let stripped = l
                .strip_prefix("fold along ")
                .ok_or(anyhow!("Expected fold"))?;
            let (s1, s2) = stripped.split_once('=').ok_or(anyhow!("Expected space"))?;
            let loc: i64 = s2.trim().parse().context("parsing fold")?;
            let fold = match s1 {
                "x" => Fold::Vertical(loc),
                "y" => Fold::Horizontal(loc),
                c => return Err(anyhow!("Expected x or y, found '{c}'")),
            };

            folds.push(fold);
        }

        // Store them backwards so we can pop them off the back
        folds.reverse();

        Ok(Self { points, folds })
    }
}

impl Instructions {
    pub fn fold(&mut self, fold: Fold) {
        let mut new_points = HashSet::new();
        match fold {
            Fold::Horizontal(y) => {
                for &(x2, y2) in &self.points {
                    if y2 > y {
                        new_points.insert((x2, 2 * y - y2));
                    }
                }
                self.points.retain(|&(_, y2)| y2 < y);
            }
            Fold::Vertical(x) => {
                for &(x2, y2) in &self.points {
                    if x2 > x {
                        new_points.insert((2 * x - x2, y2));
                    }
                }
                self.points.retain(|&(x2, _)| x2 < x);
            }
        }
        self.points.extend(new_points);
    }

    pub fn step(&mut self) -> bool {
        if let Some(fold) = self.folds.pop() {
            self.fold(fold);
            true
        } else {
            false
        }
    }

    pub fn fold_all(&mut self) {
        while self.step() {}
    }

    pub fn point_count(&self) -> usize {
        self.points.len()
    }
}

impl Display for Instructions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mx = self
            .points
            .iter()
            .copied()
            .map(|(x, _)| x)
            .max()
            .unwrap_or_default();
        let my = self
            .points
            .iter()
            .copied()
            .map(|(_, y)| y)
            .max()
            .unwrap_or_default();

        let pts: HashSet<(i64, i64)> = self.points.iter().copied().collect();

        for y in 0..=my {
            for x in 0..=mx {
                if pts.contains(&(x, y)) {
                    write!(f, "#")?;
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f)?;
        }

        // todo: folds
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day13.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let s = std::fs::read_to_string(&args.input).unwrap();
    let mut instructions = s.parse::<Instructions>().unwrap();
    let pcount = instructions.point_count();
    instructions.step();
    let pcount1 = instructions.point_count();

    instructions.fold_all();
    let pcount_end = instructions.point_count();
    println!("Found {pcount} -> {pcount1} -> {pcount_end} points");

    println!("{}", instructions);
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE: &str = r###"
        6,10
        0,14
        9,10
        0,3
        10,4
        4,11
        6,0
        6,12
        4,1
        0,13
        10,12
        3,4
        3,0
        8,4
        1,10
        2,14
        8,10
        9,0
        
        fold along y=7
        fold along x=5
    "###;

    #[test]
    fn test_parse() {
        let instructions: Instructions = EXAMPLE.parse().unwrap();
        println!("{}", instructions);
    }

    #[test]
    fn test_fold() {
        let mut instructions: Instructions = EXAMPLE.parse().unwrap();
        assert_eq!(instructions.point_count(), 18);
        debug!("{}", instructions);
        instructions.step();
        debug!("{}", instructions);
        assert_eq!(instructions.point_count(), 17);

        instructions.fold_all();
        debug!("{}", instructions);
        assert_eq!(instructions.point_count(), 16);
        let expected_raw = r###"
            #####
            #...#
            #...#
            #...#
            #####
        "###;

        // let expected: Instructions = expected_str.parse().unwrap();
        let expected: String = expected_raw
            .trim()
            .lines()
            .map(|s| format!("{}\n", s.trim_start()))
            .collect();
        assert_eq!(format!("{}", instructions), expected);
    }
}
