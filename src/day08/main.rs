use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use log::debug;

use adventofcode2021::parse;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connections {
    patterns: Vec<String>,
    outputs: Vec<String>,
}

impl Connections {
    pub fn simples(&self) -> usize {
        self.outputs
            .iter()
            .filter(|s| [2usize, 3, 4, 7].contains(&s.chars().count()))
            .count()
    }
}

impl FromStr for Connections {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let (pattern_str, output_str) = s.split_once(" | ").ok_or(anyhow::anyhow!("expected |"))?;

        let patterns = pattern_str
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let outputs = output_str
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        Ok(Connections { patterns, outputs })
    }
}

// Segments used for each digit
// e.g. SEGMENTS[3] = "acdeg" - the number 3 uses segments a, c, d, e, and g
const SEGMENTS: [&str; 10] = [
    "abcefg", "cf", "acdeg", "acdfg", "bcdf", "abdfg", "abdefg", "acf", "abcdefg",
    "abcdfg",
    // "abcefg", "cf", "acdeg", "acdfg", "bcdf", "abdfg", "abdefg", "acf", "abcdefg", "abcdfg",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Possibilities {
    // Known pattern, possible numeric matches
    patterns: HashMap<Vec<char>, HashSet<u8>>,

    // Segments to possible input wires
    rewiring: HashMap<char, HashSet<char>>,

    // Outputs for this connection set
    outputs: Vec<Vec<char>>,
}

impl Possibilities {
    pub fn new(connections: &Connections) -> Self {
        let mut patterns = HashMap::new();

        for pattern in &connections.patterns {
            let mut pattern = pattern.chars().collect::<Vec<char>>();
            pattern.sort();

            let ns: HashSet<u8> = (0..10u8)
                .filter(|&n| SEGMENTS[n as usize].chars().count() == pattern.len())
                .collect();
            patterns.insert(pattern, ns);
        }

        let outputs = connections
            .outputs
            .iter()
            .map(|s| {
                let mut cs = s.chars().collect::<Vec<char>>();
                cs.sort();
                cs
            })
            .collect::<Vec<Vec<char>>>();

        let rewiring: HashMap<char, HashSet<char>> = "abcdefg"
            .chars()
            .map(|c| (c, HashSet::from_iter("abcdefg".chars())))
            .collect();
        Self {
            patterns,
            outputs,
            rewiring,
        }
    }

    fn pattern_reduce(&mut self) -> bool {
        let mut changed = false;
        for (pattern, possible_digits) in &mut self.patterns {
            let possible_segments: HashSet<char> = possible_digits
                .iter()
                .flat_map(|&d| SEGMENTS[d as usize].chars())
                .collect();

            // Segments that could possibly be missing from the pattern
            let possible_missing: HashSet<char> = "abcdefg"
                .chars()
                .filter(|&c| {
                    possible_digits
                        .iter()
                        .any(|&d| !SEGMENTS[d as usize].contains(c))
                })
                .collect();

            debug!(
                "Looking at {} -> {:?}, segments {:?}",
                pattern.iter().collect::<String>(),
                possible_digits,
                possible_segments
            );
            for (&segment, wires) in &mut self.rewiring {
                let wire_copy = wires.clone();
                let l = wires.len();
                if pattern.contains(&segment) {
                    // e.g. 'f' above.
                    // This segment is used by the digit, so one of the wires intended for the current digit
                    // must map to this segment; segment 'f' could only be matched by 'a', 'b', or 'c' digit
                    // So segment 'b'
                    wires.retain(|&w| possible_segments.contains(&w));
                    continue;
                }

                if possible_digits.len() == 1 {
                    // This digit is known, and this segment is not lit up during this digit.
                    // Thus, it can't be any of the wires for this digit.
                    wires.retain(|&w| !possible_segments.contains(&w));
                    continue;
                }

                // This segment is not used by the pattern, so it can only be attached to a wire that is not used
                // by all the possible digits
                wires.retain(|&w| possible_missing.contains(&w));

                changed |= wires.len() != l;

                if wires.len() != l {
                    debug!("  segment {segment}: {wire_copy:?} -> {wires:?}",);
                }
            }
        }

        changed
    }

    fn wire_reduce(&mut self) -> bool {
        let mut changed = false;
        // If any wire is known, then its not a possible match for any other segment
        let known_wires: HashSet<char> = self
            .rewiring
            .values()
            .filter_map(|v| {
                if v.len() == 1 {
                    Some(*v.iter().next().unwrap())
                } else {
                    None
                }
            })
            .collect();

        for wires in self.rewiring.values_mut() {
            let l = wires.len();
            if l == 1 {
                continue;
            }
            wires.retain(|&w| !known_wires.contains(&w));
            changed |= wires.len() != l;
        }

        changed
    }

    // For any pattern that could only be one digit, remove that digit from all other patterns
    fn pattern_singles_reduce(&mut self) -> bool {
        // Digits that have a pattern with no other possibilities
        let mut loners: HashSet<u8> = HashSet::new();

        let mut counts: HashMap<u8, usize> = HashMap::new();
        for digits in self.patterns.values() {
            if digits.len() == 1 {
                loners.extend(digits);
            }
            for &d in digits {
                *counts.entry(d).or_insert(0) += 1;
            }
        }

        // Digits that have only one possible pattern
        let singles: HashSet<u8> = counts
            .iter()
            .flat_map(|(&d, &cnt)| if cnt == 1 { Some(d) } else { None })
            .collect();

        let mut changed = false;
        for digits in self.patterns.values_mut() {
            let l = digits.len();
            if l == 1 {
                continue;
            }

            // Loners are already taken
            digits.retain(|&d| !loners.contains(&d));

            let possible_single = singles.intersection(digits).next().copied();
            if let Some(d) = possible_single {
                digits.clear();
                digits.insert(d);
            }

            changed |= digits.len() != l;
        }

        changed
    }

    // For any wire that has only one possible segment, that segment must be that wire
    fn wire_singles_reduce(&mut self) -> bool {
        let mut counts = HashMap::new();
        for wires in self.rewiring.values() {
            for &w in wires {
                *counts.entry(w).or_insert(0) += 1;
            }
        }

        let mut changed = false;
        for (&w, &count) in &counts {
            if count == 1 {
                for wires in self.rewiring.values_mut() {
                    if wires.contains(&w) && wires.len() > 1 {
                        changed = true;
                        wires.clear();
                        wires.insert(w);
                        break;
                    }
                }
            }
        }

        changed
    }

    fn solve_known_wire_patterns(&mut self) -> bool {
        let mut changed = false;
        'outer: for (pattern, digits) in &mut self.patterns {
            if digits.len() == 1 {
                continue;
            }

            let mut wires: Vec<char> = Vec::new();
            for c in pattern {
                let wire_possibilities = self.rewiring.get(c).unwrap();
                if wire_possibilities.len() != 1 {
                    continue 'outer;
                }
                wires.extend(wire_possibilities);
            }

            // So we know exactly what digit this is, so we know exactly what digit this should be.
            wires.sort();
            let wire_str = wires.iter().collect::<String>();

            let d = SEGMENTS
                .iter()
                .enumerate()
                .flat_map(|(d, &s)| if s == wire_str { Some(d as u8) } else { None })
                .next()
                .unwrap();

            digits.clear();
            digits.insert(d);
            changed = true;
        }

        changed
    }

    // Determine which pattern is 3, and use that to determine segments b, e, and f
    fn solve_three(&mut self) -> bool {
        let five_pats: Vec<_> = self
            .patterns
            .keys()
            .flat_map(|p| if p.len() == 5 { Some(p.clone()) } else { None })
            .collect();

        for p in &five_pats {
            if self.patterns.get(p).unwrap() == &HashSet::from_iter(vec![3]) {
                // Already know which one is 3
                return false;
            }
        }

        if five_pats.len() != 3 {
            debug!("Not enough 5-patterns");
            return false;
        }

        let not_ins = five_pats
            .iter()
            .map(|p| {
                "abcdefg"
                    .chars()
                    .filter(|&c| !p.contains(&c))
                    .collect::<HashSet<char>>()
            })
            .collect::<Vec<HashSet<char>>>();

        // index of digit 3
        let tix = if not_ins[1].intersection(&not_ins[2]).count() == 0 {
            0
        } else if not_ins[0].intersection(&not_ins[2]).count() == 0 {
            1
        } else {
            assert_eq!(not_ins[0].intersection(&not_ins[1]).count(), 0);
            2
        };

        let three_pats = self.patterns.get_mut(&five_pats[tix]).unwrap();
        assert!(three_pats.len() > 1);
        assert!(three_pats.contains(&3));
        three_pats.clear();
        three_pats.insert(3);

        true
    }

    pub fn simplify(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;

            changed |= self.pattern_reduce();
            changed |= self.pattern_singles_reduce();
            changed |= self.wire_reduce();
            changed |= self.wire_singles_reduce();
            changed |= self.solve_known_wire_patterns();

            if changed {
                continue;
            }

            // It turns out - this isn't needed, the above cover all cases
            changed |= self.solve_three();
        }
    }

    pub fn all_known(&self) -> bool {
        self.patterns.values().all(|ds| ds.len() == 1)
    }

    pub fn lookup(&self, pattern: &str) -> Option<u8> {
        let mut pattern = pattern.chars().collect::<Vec<char>>();
        pattern.sort();
        let pattern = pattern;

        let digits = self.patterns.get(&pattern)?;
        if digits.len() != 1 {
            return None;
        }
        digits.iter().next().copied()
    }

    pub fn solve_outputs(&self) -> Option<u64> {
        let mut looked_up: u64 = 0;
        for output in &self.outputs {
            let digits = self.patterns.get(output)?;
            if digits.len() != 1 {
                return None;
            }
            let d = digits.iter().next().copied()?;
            looked_up *= 10;
            looked_up += d as u64;
        }
        Some(looked_up)
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day08.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let file = File::open(args.input).unwrap();
    let buf = BufReader::new(file);
    let connections: Vec<Connections> = parse::buffer(buf).unwrap();

    let count: usize = connections.iter().map(|c| c.simples()).sum();
    println!("Found {count} simples");

    let mut total: u64 = 0;
    for connections in &connections {
        let mut possibilites = Possibilities::new(connections);
        possibilites.simplify();
        if !possibilites.all_known() {
            panic!("Unsolved!");
        }

        let looked_up = possibilites.solve_outputs().unwrap();
        total += looked_up;
    }

    println!("Output sum: {total}");
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE: &str = r###"
        be cfbegad cbdgef fgaecd cgeb fdcge agebfd fecdb fabcd edb | fdgacbe cefdb cefbgd gcbe
        edbfga begcd cbg gc gcadebf fbgde acbgfd abcde gfcbed gfec | fcgedb cgb dgebacf gc
        fgaebd cg bdaec gdafb agbcfd gdcbef bgcad gfac gcb cdgabef | cg cg fdcagb cbg
        fbegcd cbd adcefb dageb afcb bc aefdc ecdab fgdeca fcdbega | efabcd cedba gadfec cb
        aecbfdg fbg gf bafeg dbefa fcge gcbea fcaegb dgceab fcbdga | gecf egdcabf bgf bfgea
        fgeab ca afcebg bdacfeg cfaedg gcfdb baec bfadeg bafgc acf | gebdcfa ecba ca fadegcb
        dbcfg fgd bdegcaf fgec aegbdf ecdfab fbedc dacgb gdcebf gf | cefg dcbef fcge gbcadfe
        bdfegc cbegaf gecbf dfcage bdacg ed bedf ced adcbefg gebcd | ed bcgafe cdgba cbgef
        egadfb cdbfeg cegd fecab cgb gbdefca cg fgcdab egfdb bfceg | gbdfcae bgc cg cgb
        gcafb gcf dcaebfg ecagb gf abcdeg gaef cafbge fdbac fegbdc | fgae cfgab fg bagce
    "###;

    const EXAMPLE_OUTPUTS: [u64; 10] = [8394, 9781, 1197, 9361, 4873, 8418, 4548, 1625, 8717, 4315];

    #[test]
    fn test_basic() {
        let connections: Vec<Connections> = parse::buffer(EXAMPLE.as_bytes()).unwrap();
        let count: usize = connections.iter().map(|c| c.simples()).sum();
        assert_eq!(count, 26);
    }

    #[test]
    fn test_simplify() {
        let connections: Vec<Connections> = parse::buffer(EXAMPLE.as_bytes()).unwrap();
        let mut possibilities = Possibilities::new(&connections[0]);
        possibilities.simplify();

        println!("{:?}", possibilities);
        assert!(possibilities.all_known());
        assert_eq!(possibilities.lookup("acedgfb"), Some(8));
    }

    #[test]
    fn test_full_example() {
        let connections: Vec<Connections> = parse::buffer(EXAMPLE.as_bytes()).unwrap();
        assert_eq!(connections.len(), EXAMPLE_OUTPUTS.len());

        let mut output_sum = 0;
        for (c, &out) in connections.iter().zip(EXAMPLE_OUTPUTS.iter()) {
            let mut possibilities = Possibilities::new(c);
            possibilities.simplify();

            // println!("{:?}", possibilities);
            assert!(possibilities.all_known());
            let output = possibilities.solve_outputs().unwrap();

            assert_eq!(output, out);
            output_sum += output;
        }

        assert_eq!(output_sum, 61229);
    }
}
