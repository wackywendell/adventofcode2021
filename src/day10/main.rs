use std::path::PathBuf;

use clap::Parser;
use log::debug;

pub fn pair(c: char) -> Option<char> {
    match c {
        '(' => Some(')'),
        '<' => Some('>'),
        '{' => Some('}'),
        '[' => Some(']'),
        ')' => Some('('),
        '>' => Some('<'),
        '}' => Some('{'),
        ']' => Some('['),
        _ => None,
    }
}

pub fn mismatches(s: &str) -> (Vec<char>, Vec<char>) {
    let mut closers = Vec::new();
    let mut stack = Vec::new();
    for c in s.chars() {
        match c {
            '(' | '<' | '{' | '[' => stack.push(c),
            ')' | '>' | '}' | ']' => {
                let expected = pair(c).unwrap();
                let popped = stack.pop();
                match popped {
                    None => {
                        closers.push(c);
                    }
                    Some(p) if p == expected => {
                        // It matches, all is well
                    }
                    Some(p) => {
                        // It doesn't match; put it in the list of closers,
                        // and put the popped one back on
                        stack.push(p);
                        closers.push(c);
                    }
                }
            }
            _ => {}
        }
    }

    (stack, closers)
}

pub fn score_pairs(s: &str) -> (Vec<i64>, Vec<i64>) {
    let mut closers_scores = Vec::new();
    let mut openers_scores = Vec::new();

    for line in s.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }

        let (unclosed, closers) = mismatches(t);

        // debug!(
        //     "Line: {t}\n    {unclosed:?}, {closers:?}",
        //     unclosed = _unclosed.iter().collect::<String>(),
        //     closers = closers.iter().collect::<String>(),
        // );

        let s = match closers.first() {
            None => 0,
            Some(')') => 3,
            Some(']') => 57,
            Some('}') => 1197,
            Some('>') => 25137,
            Some(c) => panic!("Unexpected closer {c}"),
        };

        if s > 0 {
            closers_scores.push(s);
            continue;
        }

        let mut score = 0i64;
        for &c in unclosed.iter().rev() {
            let cur = match c {
                '(' => 1,
                '[' => 2,
                '{' => 3,
                '<' => 4,
                _ => panic!("Unexpected opener {c}"),
            };

            score = score * 5 + cur;
        }
        openers_scores.push(score);
    }

    (closers_scores, openers_scores)
}

pub fn score_pair(s: &str) -> (i64, i64) {
    let (closers_scores, mut openers_scores) = score_pairs(s);
    let closers_score: i64 = closers_scores.iter().sum();
    openers_scores.sort();
    let openers_score: i64 = openers_scores[openers_scores.len() / 2];

    (closers_score, openers_score)
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day10.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let s = std::fs::read_to_string(&args.input).unwrap();

    let (closers_score, openers_score) = score_pair(&s);

    println!("Found scores {closers_score}, {openers_score}");
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE: &str = r###"
        [({(<(())[]>[[{[]{<()<>>
        [(()[<>])]({[<{<<[]>>(
        {([(<{}[<>[]}>{[]{[(<()>
        (((({<>}<{<{<>}{[]{[]{}
        [[<[([]))<([[{}[[()]]]
        [{[{({}]{}}([{[{{{}}([]
        {<[[]]>}<{[{[{[]{()[[[]
        [<(<(<(<{}))><([]([]()
        <{([([[(<>()){}]>(<<{{
        <{([{{}}[<[[[<>{}]]]>[]]
    "###;

    #[test]
    fn test_basic() {
        let (closers_scores, openers_scores) = score_pairs(EXAMPLE);
        assert_eq!(closers_scores, vec![1197, 3, 57, 3, 25137]);
        assert_eq!(openers_scores, vec![288957, 5566, 1480781, 995444, 294]);

        let (s1, s2) = score_pair(EXAMPLE);

        assert_eq!(s1, 26397);
        assert_eq!(s2, 288957);
    }
}
