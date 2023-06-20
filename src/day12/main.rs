use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Display;
use std::fs::File;
use std::hash::Hash;
use std::io::BufReader;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::anyhow;
use clap::Parser;
use log::debug;

use adventofcode2021::parse;

#[derive(Debug, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Cave {
    Start,
    Named(char, char),
    End,
}

impl Cave {
    pub fn is_big(self) -> bool {
        match self {
            Cave::Start | Cave::End => false,
            Cave::Named(first, second) => {
                first.is_ascii_uppercase() && (second.is_ascii_uppercase() || second == ' ')
            }
        }
    }
}

impl Display for Cave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Cave::Start => write!(f, "start"),
            Cave::End => write!(f, "end"),
            Cave::Named(first, ' ') => write!(f, "{first}"),
            Cave::Named(first, second) => write!(f, "{first}{second}"),
        }
    }
}

impl FromStr for Cave {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "start" => Cave::Start,
            "end" => Cave::End,
            _ => {
                let mut chars = s.chars();
                let first = chars.next().ok_or(anyhow!("Need a first character"))?;
                let second = chars.next().unwrap_or(' ');
                if let Some(c) = chars.next() {
                    return Err(anyhow!("Too many characters: {c}"));
                }
                Cave::Named(first, second)
            }
        })
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
struct Pair(Cave, Cave);

impl FromStr for Pair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (first, second) = s.split_once('-').ok_or(anyhow::anyhow!("Invalid pair"))?;
        let first = Cave::from_str(first)?;
        let second = Cave::from_str(second)?;
        Ok(Pair(first, second))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caves {
    connections: HashMap<Cave, HashSet<Cave>>,
}

impl Caves {
    pub fn paths(&self) -> HashSet<Vec<Cave>> {
        let mut paths: HashSet<Vec<Cave>> = HashSet::new();
        let mut queue: VecDeque<Vec<Cave>> = VecDeque::new();
        queue.push_back(vec![Cave::Start]);
        while let Some(path) = queue.pop_front() {
            let &cur = path.last().unwrap();
            if cur == Cave::End {
                paths.insert(path);
                continue;
            }

            let neighbors = self.connections.get(&cur).unwrap();

            for &neighbor in neighbors {
                if !neighbor.is_big() && path.contains(&neighbor) {
                    // Can't return to small caves
                    continue;
                }

                let mut new_path = path.clone();
                new_path.push(neighbor);
                queue.push_back(new_path);
            }
        }

        paths
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CavePaths(HashSet<Vec<Cave>>);

impl Display for CavePaths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut paths: Vec<Vec<Cave>> = self.0.iter().cloned().collect();
        paths.sort();

        for (ix, mut path) in paths.into_iter().enumerate() {
            if ix > 0 {
                writeln!(f)?;
            }
            let last = path.pop();
            for cave in path {
                write!(f, "{}-", cave)?;
            }
            if let Some(cave) = last {
                write!(f, "{}", cave)?;
            }
        }

        Ok(())
    }
}

impl FromIterator<Pair> for Caves {
    fn from_iter<T: IntoIterator<Item = Pair>>(iter: T) -> Self {
        let mut connections: HashMap<Cave, HashSet<Cave>> = HashMap::new();
        for pair in iter {
            connections.entry(pair.0).or_default().insert(pair.1);
            connections.entry(pair.1).or_default().insert(pair.0);
        }
        Caves { connections }
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day12.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let file = File::open(args.input).unwrap();
    let buf = BufReader::new(file);
    let caves: Caves = parse::buffer(buf).unwrap();

    let paths = caves.paths();

    println!("Found {} paths", paths.len());
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE_SMALL: &str = r###"
        start-A
        start-b
        A-c
        A-b
        b-d
        A-end
        b-end
    "###;

    #[test]
    fn test_basic() {
        let caves: Caves = parse::buffer(EXAMPLE_SMALL.as_bytes()).unwrap();
        assert_eq!(caves.connections.len(), 6);

        let paths = caves.paths();
        assert_eq!(paths.len(), 10);
    }

    const EXAMPLE_MEDIUM: &str = r###"
        dc-end
        HN-start
        start-kj
        dc-start
        dc-HN
        LN-dc
        HN-end
        kj-sa
        kj-HN
        kj-dc
    "###;

    const EXAMPLE_BIG: &str = r###"
        fs-end
        he-DX
        fs-he
        start-DX
        pj-DX
        end-zg
        zg-sl
        zg-pj
        pj-he
        RW-he
        fs-DX
        pj-RW
        zg-RW
        start-pj
        he-WI
        zg-he
        pj-fs
        start-RW
    "###;

    #[test]
    fn test_paths() {
        let caves: Caves = parse::buffer(EXAMPLE_MEDIUM.as_bytes()).unwrap();
        assert_eq!(caves.connections.len(), 7);

        let paths = caves.paths();
        assert_eq!(paths.len(), 19);
        let caves: Caves = parse::buffer(EXAMPLE_BIG.as_bytes()).unwrap();
        assert_eq!(caves.connections.len(), 10);

        let paths = caves.paths();
        assert_eq!(paths.len(), 226);
    }
}
