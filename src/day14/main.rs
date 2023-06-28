use std::collections::HashMap;

use std::path::PathBuf;
use std::str::FromStr;

use anyhow::anyhow;
use clap::Parser;
use log::debug;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct FormulaCounts {
    rules: HashMap<(char, char), char>,
    // (character, character) -> count
    template: HashMap<(char, char), usize>,
    begin: char,
    end: char,
}

impl From<Formula> for FormulaCounts {
    fn from(value: Formula) -> Self {
        assert!(value.template.len() >= 2);

        let mut chars = value.template.chars();
        let begin = chars.next().unwrap();
        let mut last = begin;

        let mut template = HashMap::new();
        for c in chars {
            *template.entry((last, c)).or_insert(0usize) += 1;
            last = c;
        }

        FormulaCounts {
            rules: value.rules,
            template,
            begin,
            end: last,
        }
    }
}

impl FormulaCounts {
    pub fn step(&mut self) {
        let mut new = HashMap::new();
        for (&(c1, c2), &count) in self.template.iter() {
            if let Some(&mid) = self.rules.get(&(c1, c2)) {
                *new.entry((c1, mid)).or_insert(0usize) += count;
                *new.entry((mid, c2)).or_insert(0usize) += count;
            } else {
                *new.entry((c1, c2)).or_insert(0usize) += count;
            }
        }
        self.template = new;
    }

    pub fn score(&self) -> i64 {
        let mut counts = HashMap::new();
        counts.insert(self.begin, 1i64);
        *counts.entry(self.end).or_insert(1) += 1;
        for (&(c1, c2), &count) in self.template.iter() {
            *counts.entry(c1).or_insert(0i64) += count as i64;
            *counts.entry(c2).or_insert(0i64) += count as i64;
        }

        // Counts are the number of pairs each letter is in (plus one for begin and end),
        // so divide by two to get the actual letter count
        let mn = counts.values().min().unwrap() / 2;
        let mx = counts.values().max().unwrap() / 2;

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

    let initial = Formula::from_str(&input).unwrap();
    let mut formula = initial.clone();

    for _ in 0..10 {
        formula.step();
    }

    let length = formula.template.chars().count();
    let score = formula.score();
    println!("Found {length} template, score {score}");

    let mut counts = FormulaCounts::from(initial);
    for _ in 0..40 {
        counts.step();
    }
    let score = counts.score();
    println!("Found score {score}");
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

    #[test]
    fn test_long() {
        let mut formula = Formula::from_str(EXAMPLE).unwrap();
        let mut counts = FormulaCounts::from(formula.clone());
        assert_eq!(formula.score(), counts.score());

        for _ in 0..10 {
            formula.step();
            counts.step();

            let temp_counts = FormulaCounts::from(formula.clone());
            assert_eq!(counts, temp_counts);
            assert_eq!(formula.score(), counts.score());
        }

        for _ in 10..40 {
            counts.step();
        }
        assert_eq!(counts.score(), 2188189693529);
    }
}
