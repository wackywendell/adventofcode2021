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

pub fn score_mismatches(s: &str) -> i64 {
    let mut score = 0;

    for line in s.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }

        let (_unclosed, closers) = mismatches(t);

        // debug!(
        //     "Line: {t}\n    {unclosed:?}, {closers:?}",
        //     unclosed = _unclosed.iter().collect::<String>(),
        //     closers = closers.iter().collect::<String>(),
        // );

        let s = match closers.first() {
            None => continue,
            Some(')') => 3,
            Some(']') => 57,
            Some('}') => 1197,
            Some('>') => 25137,
            Some(c) => panic!("Unexpected closer {c}"),
        };

        score += s;
    }

    score
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

    let score = score_mismatches(&s);

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
        let score = score_mismatches(EXAMPLE);
        assert_eq!(score, 26397);
    }
}
