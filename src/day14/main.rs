use std::collections::HashMap;


use std::path::PathBuf;
use std::str::FromStr;

use anyhow::anyhow;
use clap::Parser;
use log::debug;



pub struct Formula {
    rules: HashMap<(char, char), char>,
    template: String,
}

impl FromStr for Formula {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut rules = HashMap::new();
        let mut lines = s.lines();

        let template = loop {
            let template = lines.next().ok_or(anyhow!("Expected template"))?.trim();
            if template.is_empty() {
                continue;
            }
            break template.to_string();
        };

        for line in lines.by_ref() {
            let l = line.trim();
            if l.is_empty() {
                continue;
            }
            let (s1, s2) = l.split_once(" -> ").ok_or(anyhow!("Expected arrow"))?;
            let (s1, s2) = (s1.trim(), s2.trim());
            let mut cs1 = s1.chars();
            let (c1, c2) = (
                cs1.next().ok_or_else(|| anyhow!("Expected char"))?,
                cs1.next().ok_or_else(|| anyhow!("Expected char"))?,
            );
            if let Some(c) = cs1.next() {
                return Err(anyhow!("Expected two characters, found {}", c));
            }
            let mut cs2 = s2.chars();
            let cmid = cs2.next().ok_or_else(|| anyhow!("Expected char"))?;
            if let Some(c) = cs2.next() {
                return Err(anyhow!("Expected one characters, found {}", c));
            }
            rules.insert((c1, c2), cmid);
        }

        Ok(Formula { rules, template })
    }
}

impl Formula {
    pub fn step(&mut self) {
        if self.template.chars().take(2).count() < 2 {
            return;
        }

        let mut new = String::new();
        let mut chars = self.template.chars();
        let mut last = chars.next().unwrap();
        new.push(last);
        for c in chars {
            if let Some(&cnew) = self.rules.get(&(last, c)) {
                new.push(cnew);
            }
            new.push(c);
            last = c;
        }
        self.template = new;
    }

    pub fn score(&self) -> i64 {
        if self.template.len() < 2 {
            return 0;
        }
        let mut counts = HashMap::new();
        for c in self.template.chars() {
            *counts.entry(c).or_insert(0i64) += 1;
        }

        let &mn = counts.values().min().unwrap();
        let &mx = counts.values().max().unwrap();

        mx - mn
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day14.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let input = std::fs::read_to_string(&args.input).unwrap();
    let mut formula = Formula::from_str(&input).unwrap();

    for _ in 0..10 {
        formula.step();
    }

    let length = formula.template.chars().count();
    let score = formula.score();
    println!("Found {length} template, score {score}");
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE: &str = r###"
        NNCB

        CH -> B
        HH -> N
        CB -> H
        NH -> C
        HB -> C
        HC -> B
        HN -> C
        NN -> C
        BH -> H
        NC -> B
        NB -> B
        BN -> B
        BB -> N
        BC -> B
        CC -> N
        CN -> C
    "###;

    #[test]
    fn test_basic() {
        let mut formula = Formula::from_str(EXAMPLE).unwrap();
        assert_eq!(16, formula.rules.len());
        assert_eq!(4, formula.template.chars().count());

        let expected = vec![
            "NCNBCHB",
            "NBCCNBBBCBHCB",
            "NBBBCNCCNBBNBNBBCHBHHBCHB",
            "NBBNBNBBCCNBCNCCNBBNBBNBBBNBBNBBCBHCBHHNHCBBCBHCB",
        ];

        for (i, e) in expected.iter().enumerate() {
            formula.step();
            assert_eq!(e, &formula.template, "Failed at step {}", i + 1);
        }

        formula = Formula::from_str(EXAMPLE).unwrap();
        for _ in 0..10 {
            formula.step();
        }
        let score = formula.score();
        assert_eq!(score, 1588);
    }
}
