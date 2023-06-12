use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use itertools::Itertools;
use log::debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BingoGame {
    instructions: Vec<u16>,
    boards: Vec<Board>,
    played: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    Continuing(u16),
    Won(u16, usize),
    Finished,
}

impl BingoGame {
    pub fn parse(buf: impl BufRead) -> anyhow::Result<Self> {
        let mut lines = buf.lines();
        let first = loop {
            let line = lines
                .next()
                .ok_or(anyhow::anyhow!("expected first line"))??;
            if !line.is_empty() {
                break line;
            }
        };

        let ns: Result<Vec<u16>, _> = first
            .trim()
            .split(',')
            .map(|ns| ns.parse::<u16>())
            .collect();
        let instructions = ns?;

        let chunks = lines.chunks(6);
        let boards_iter = chunks
            .into_iter()
            .map(|ls| {
                ls.skip(1).collect::<Result<Vec<String>, _>>().map(|ls| {
                    if !ls.is_empty() {
                        Some(Board::from_lines(&ls))
                    } else {
                        None
                    }
                })
            })
            .flat_map(|l| l.transpose());

        let boards_result: std::io::Result<anyhow::Result<Vec<Board>>> = boards_iter.collect();
        let boards: Vec<Board> = boards_result??;

        Ok(BingoGame {
            instructions,
            boards,
            played: 0,
        })
    }

    pub fn draw(&mut self) -> State {
        let value = match self.instructions.get(self.played) {
            Some(&ix) => ix,
            None => return State::Finished,
        };

        for (ix, board) in self.boards.iter_mut().enumerate() {
            debug!("Checking board {ix}, value {value}");
            board.draw(value);
            if board.won() {
                return State::Won(value, ix);
            }
        }

        self.played += 1;
        State::Continuing(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Board {
    values: [[u16; 5]; 5],
    crossed: [[bool; 5]; 5],
}

impl Board {
    pub fn from_lines(lines: &[impl AsRef<str>]) -> anyhow::Result<Self> {
        let mut values: [[u16; 5]; 5] = Default::default();

        assert_eq!(values.len(), 5);

        for (ix, s) in lines.iter().enumerate() {
            let s = s.as_ref();
            let s = s.trim();
            let ns: Result<Vec<u16>, ParseIntError> = s
                .split(' ')
                .filter(|&s| !s.is_empty())
                .map(u16::from_str)
                .collect();
            let ns = ns?;
            assert_eq!(ns.len(), 5);

            values[ix] = ns.as_slice().try_into()?;
        }

        Ok(Board {
            values,
            crossed: Default::default(),
        })
    }

    pub fn draw(&mut self, n: u16) {
        for ix1 in 0..5 {
            for ix2 in 0..5 {
                if self.values[ix1][ix2] == n {
                    self.crossed[ix1][ix2] = true;
                }
            }
        }
    }

    pub fn won(&self) -> bool {
        for ix1 in 0..5 {
            let mut row = true;
            let mut col = true;
            for ix2 in 0..5 {
                row &= self.crossed[ix1][ix2];
                col &= self.crossed[ix2][ix1];

                if !(row || col) {
                    break;
                }
            }

            if row || col {
                // debug!("{ix1}: {row}, {col}");
                // debug!("{:?}", self.values);
                // debug!("{:?}", self.crossed);
                return true;
            }
        }

        false
    }

    pub fn unmarked_sum(&self) -> u32 {
        let mut sum = 0u32;
        for ix1 in 0..5 {
            for ix2 in 0..5 {
                if self.crossed[ix1][ix2] {
                    continue;
                }
                sum += self.values[ix1][ix2] as u32;
            }
        }

        sum
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day04.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let file = File::open(args.input).unwrap();
    let buf = BufReader::new(file);

    let mut game = BingoGame::parse(buf).unwrap();

    loop {
        match game.draw() {
            State::Continuing(value) => println!("Drew {value}"),
            State::Won(value, ix) => {
                let sum = game.boards[ix].unmarked_sum();
                let mul = sum * (value as u32);
                println!("Drew {value}, and {ix} Won with sum {sum} (mul {mul})!");
                break;
            }
            State::Finished => {
                println!("No winners?!");
                break;
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE: &str = r###"
        7,4,9,5,11,17,23,2,0,14,21,24,10,16,13,6,15,25,12,22,18,20,8,19,3,26,1

        22 13 17 11  0
         8  2 23  4 24
        21  9 14 16  7
         6 10  3 18  5
         1 12 20 15 19
        
         3 15  0  2 22
         9 18 13 17  5
        19  8  7 25 23
        20 11 10 24  4
        14 21 16 12  6
        
        14 21 17 24  4
        10 16 15  9 19
        18  8 23 26 20
        22 11 13  6  5
         2  0 12  3  7
    "###;

    #[test]
    fn test_parse() {
        let game = BingoGame::parse(EXAMPLE.as_bytes()).unwrap();
        assert_eq!(&game.instructions[..3], vec![7, 4, 9]);
        assert_eq!(game.instructions.len(), 27);
        assert_eq!(&game.instructions[24..], vec![3, 26, 1]);
    }

    #[test]
    fn test_games() {
        let mut game = BingoGame::parse(EXAMPLE.as_bytes()).unwrap();

        assert_eq!(game.draw(), State::Continuing(7));
        assert_eq!(game.draw(), State::Continuing(4));
        assert_eq!(game.draw(), State::Continuing(9));
        assert_eq!(game.draw(), State::Continuing(5));
        assert_eq!(game.draw(), State::Continuing(11));
        assert_eq!(game.draw(), State::Continuing(17));
        assert_eq!(game.draw(), State::Continuing(23));
        assert_eq!(game.draw(), State::Continuing(2));
        assert_eq!(game.draw(), State::Continuing(0));
        assert_eq!(game.draw(), State::Continuing(14));
        assert_eq!(game.draw(), State::Continuing(21));
        assert_eq!(game.draw(), State::Won(24, 2));

        assert_eq!(game.boards[2].unmarked_sum(), 188);
    }
}
