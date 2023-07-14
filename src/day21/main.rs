use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use log::debug;

mod parser {
    use super::Game;

    use nom::bytes::complete::tag;
    use nom::character::complete::{char, digit1, one_of};
    use nom::combinator::{all_consuming, map, map_res, recognize};
    use nom::error::Error as NomError;
    use nom::multi::many0;
    use nom::sequence::{pair, preceded, tuple};

    pub type ErrorRef<'a> = nom::Err<NomError<&'a str>>;
    pub type Error = nom::Err<NomError<String>>;

    fn ws(input: &str) -> nom::IResult<&str, &str> {
        recognize(many0(one_of(" \n")))(input)
    }

    fn newline(input: &str) -> nom::IResult<&str, &str> {
        recognize(pair(char('\n'), many0(char(' '))))(input)
    }

    fn int(input: &str) -> nom::IResult<&str, i64> {
        map_res(digit1, str::parse::<i64>)(input)
    }

    pub fn game(input: &str) -> Result<Game, ErrorRef> {
        let line1 = preceded(tag("Player 1 starting position: "), int);
        let line2 = preceded(tag("Player 2 starting position: "), int);

        all_consuming(map(
            tuple((ws, line1, newline, line2, ws)),
            |(_, p1, _, p2, _)| Game::new(p1, p2),
        ))(input)
        .map(|(_, game)| game)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeterministicDie {
    sides: i64,
    next: i64,
}

impl DeterministicDie {
    pub fn new(sides: i64) -> Self {
        Self { sides, next: 1 }
    }

    pub fn roll(&mut self) -> i64 {
        let result = self.next;
        self.next = (self.next % self.sides) + 1;
        result
    }
}

impl Iterator for DeterministicDie {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.roll())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TripleRoll(DeterministicDie);

impl TripleRoll {
    pub fn new(sides: i64) -> Self {
        Self(DeterministicDie::new(sides))
    }
}

impl Iterator for TripleRoll {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0.roll() + self.0.roll() + self.0.roll())
    }
}

pub const DIRAC_ROLLS: [(i64, usize); 7] = [
    (3, 1), // 1-1-1
    (4, 3), // 1-1-2, 3x
    (5, 6), // 1-2-2, 3x, 1-1-3, 3x
    (6, 7), // 1-2-3, 6x, 2-2-2, 1x
    (7, 6), // 1-3-3, 3x, 2-2-3, 3x
    (8, 3), // 2-3-3, 3x
    (9, 1), // 3-3-3
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Game {
    position1: i64,
    position2: i64,
    score1: i64,
    score2: i64,
}

impl Game {
    pub fn new(position1: i64, position2: i64) -> Self {
        Self {
            position1,
            position2,
            score1: 0,
            score2: 0,
        }
    }

    pub fn scores(&self) -> (i64, i64) {
        (self.score1, self.score2)
    }

    // Run a practice round. Returns number of rounds and score of loser.
    pub fn practice(&mut self) -> (usize, i64) {
        let die = TripleRoll::new(100);

        for (n, roll) in die.enumerate() {
            if n % 2 == 0 {
                self.position1 = ((self.position1 + roll - 1) % 10) + 1;
                self.score1 += self.position1;
                if self.score1 >= 1000 {
                    return ((n + 1) * 3, self.score2);
                }
            } else {
                self.position2 = ((self.position2 + roll - 1) % 10) + 1;
                self.score2 += self.position2;
                if self.score2 >= 1000 {
                    return ((n + 1) * 3, self.score1);
                }
            }
        }

        unreachable!("Die should never run out of rolls")
    }

    // Determines all the possible number of ways a win could happen.
    //
    // Returns:
    // - a map of (number of turns) -> (number of ways to win, number of ways to lose)
    pub fn win_states(start: i64, max_score: i64) -> HashMap<usize, (usize, usize)> {
        // state is (score, rolls, position).
        let first = (0i64, 0usize, start);
        // (score, # of rolls, position) -> # of ways to get there
        let mut states = HashMap::new();
        states.insert(first, 1usize);

        // Previously visited states we haven't checked yet. We use Reverse so
        // the min is popped out of the queue.
        let mut queue = BinaryHeap::new();
        queue.push(Reverse(first));

        // map of number of turns -> # of ways to win
        let mut completed: HashMap<usize, (usize, usize)> = HashMap::new();

        while let Some(Reverse((score, rolls, position))) = queue.pop() {
            while queue.peek() == Some(&Reverse((score, rolls, position))) {
                // Deduplicate
                queue.pop();
            }

            let ways = states[&(score, rolls, position)];

            if score >= max_score {
                // We add here, because there are multiple end positions that
                // can lead to the same score in the same number of rolls
                let (wins, _losses) = completed.entry(rolls).or_insert((0, 0));
                *wins += ways;
                continue;
            }

            let (_wins, losses) = completed.entry(rolls).or_insert((0, 0));
            *losses += ways;

            for &(roll, roll_ways) in &DIRAC_ROLLS {
                let next_position = ((position + roll - 1) % 10) + 1;
                let next_score = score + next_position;

                let next = (next_score, rolls + 3, next_position);
                let next_ways = states.entry(next).or_insert(0);

                debug!(
                    "{score} {rolls} {position} -> {next_score} {next_rolls} {next_position} ({next_ways} += {ways}*{roll_ways})",
                    next_rolls = rolls + 3,
                );

                *next_ways += ways * roll_ways;
                queue.push(Reverse(next));
            }
        }

        completed
    }

    pub fn win_universes(&self, max_score: i64) -> (usize, usize) {
        let states1 = Self::win_states(self.position1, max_score);
        let states2 = Self::win_states(self.position2, max_score);

        let mut wins1 = 0;
        for (&turns1, &(ways1, _)) in &states1 {
            if ways1 == 0 {
                continue;
            }

            // use turns2 - 3, because if player 1 wins, player 2 must have
            // taken 1 less turn (3 less rolls) than player 1, since player 1 goes first
            let (_, ways2) = states2.get(&(turns1 - 3)).copied().unwrap_or_default();
            debug!("turns and ways: {} {} {}", turns1, ways1, ways2);
            wins1 += ways1 * ways2;
        }

        let mut wins2 = 0;
        for (&turns2, &(ways2, _)) in &states2 {
            if ways2 == 0 {
                continue;
            }
            let (_, ways1) = states1.get(&turns2).copied().unwrap_or_default();
            debug!("turns and ways 2: {} {} {}", turns2, ways1, ways2);
            wins2 += ways1 * ways2;
        }

        (wins1, wins2)
    }
}

impl FromStr for Game {
    type Err = parser::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::game(s).map_err(|e| e.to_owned())
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day21.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let s = std::fs::read_to_string(&args.input).unwrap();
    let game = Game::from_str(&s).unwrap();
    let mut practice_game = Game::from_str(&s).unwrap();

    let (rounds, score) = practice_game.practice();
    let multiple = (rounds as i64) * score;
    println!("Practice game: {rounds} rounds, score {score}, multiple {multiple}");

    let (wins1, wins2) = game.win_universes(21);
    println!("Most wins: {}", wins1.max(wins2));
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE: &str = r###"
        Player 1 starting position: 4
        Player 2 starting position: 8
    "###;

    #[test]
    fn test_basic() {
        let game = Game::from_str(EXAMPLE).unwrap();
        assert_eq!(game.position1, 4);
        assert_eq!(game.position2, 8);
    }

    #[test]
    fn test_practice() {
        let mut game = Game::from_str(EXAMPLE).unwrap();
        let (rounds, score) = game.practice();

        assert_eq!(rounds, 993);
        assert_eq!(score, 745);
    }

    #[test]
    fn test_dirac() {
        let mut ways = HashMap::new();
        for d1 in 1..=3i64 {
            for d2 in 1..=3i64 {
                for d3 in 1..=3i64 {
                    let sum = d1 + d2 + d3;
                    let entry = ways.entry(sum).or_insert(0);
                    *entry += 1usize;
                }
            }
        }
        let static_ways: HashMap<i64, usize> = HashMap::from_iter(DIRAC_ROLLS.iter().copied());
        assert_eq!(ways, static_ways);
    }

    #[test]
    fn test_play() {
        let game = Game::from_str(EXAMPLE).unwrap();
        let (wins1, wins2) = game.win_universes(21);

        assert_eq!(wins1, 444356092776315);
        assert_eq!(wins2, 341960390180808);
    }
}
