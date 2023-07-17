use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;
use std::path::PathBuf;
use std::str::FromStr;

use adventofcode2021::nom::simplify;
use clap::Parser;
use log::{debug, info};

mod parser {
    use std::collections::HashMap;

    pub use adventofcode2021::nom::Error;
    use adventofcode2021::nom::*;

    use nom::multi::many_m_n;
    use nom::sequence::terminated;

    use crate::Location;

    use super::Burrow;

    use super::Amphipod;

    pub fn amphipod(input: &str) -> IResult<Amphipod> {
        alt((
            map(char('A'), |_| Amphipod::A),
            map(char('B'), |_| Amphipod::B),
            map(char('C'), |_| Amphipod::C),
            map(char('D'), |_| Amphipod::D),
        ))(input)
    }

    pub fn location(input: &str) -> IResult<Option<Amphipod>> {
        alt((map(amphipod, Some), map(char('.'), |_| None)))(input)
    }

    fn room_row(input: &str) -> IResult<Vec<Option<Amphipod>>> {
        preceded(char('#'), many_m_n(4, 4, terminated(location, char('#'))))(input)
    }

    pub fn burrow(input: &str) -> IResult<Burrow> {
        let (rest, _) = many0(char('\n'))(input)?;

        let (rest, indent) = recognize(many0(char(' ')))(rest)?;

        let (rest, _) = terminated(tag("#############"), char('\n'))(rest)?;

        let (rest, hallways) = delimited(
            pair(tag(indent), char('#')),
            many_m_n(11, 11, location),
            tag("#\n"),
        )(rest)?;

        let (rest, rooms1) = delimited(pair(tag(indent), tag("##")), room_row, tag("##\n"))(rest)?;
        let (rest, more_rooms) = many0(delimited(
            pair(tag(indent), tag("  ")),
            room_row,
            char('\n'),
        ))(rest)?;
        let (rest, _) = tuple((tag(indent), tag("  #########"), ws))(rest)?;

        let room_depth = more_rooms.len() as i16 + 1;

        let mut amphipods = HashMap::new();
        for (amph, loc) in hallways.into_iter().zip(1..=11) {
            if let Some(amphipod) = amph {
                amphipods.insert(Location::Hallway(loc), amphipod);
            }
        }
        for (amph, loc) in rooms1.into_iter().zip(1..=4) {
            if let Some(amphipod) = amph {
                amphipods.insert(Location::Room(loc, 1), amphipod);
            }
        }
        for (row, depth) in more_rooms.into_iter().zip(2..) {
            for (amph, room) in row.into_iter().zip(1..=4) {
                if let Some(amphipod) = amph {
                    amphipods.insert(Location::Room(room, depth), amphipod);
                }
            }
        }

        Ok((
            rest,
            Burrow {
                amphipods,
                room_depth,
            },
        ))
    }

    pub fn only_burrow(input: &str) -> IResult<Burrow> {
        all_consuming(terminated(burrow, ws))(input)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Location {
    // (which room (1-4), depth in (1, 2))
    Room(i8, i16),
    // (which location - 1-11)
    Hallway(i16),
}

impl Location {
    fn to_hallway(self) -> (i16, i16) {
        match self {
            // 1 -> 3, 2 -> 5, 3 -> 7, 4 -> 9
            Self::Room(room, depth) => (2 * room as i16 + 1, depth),
            Self::Hallway(hallway) => (hallway, 0),
        }
    }

    pub fn distance(self, other: Self) -> i64 {
        let (h1, d1) = self.to_hallway();
        let (h2, d2) = other.to_hallway();
        ((h1 - h2).abs() + d1 + d2) as i64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Amphipod {
    A,
    B,
    C,
    D,
}

impl Amphipod {
    pub fn energy(&self) -> i64 {
        match self {
            Self::A => 1,
            Self::B => 10,
            Self::C => 100,
            Self::D => 1000,
        }
    }

    pub fn char(self) -> char {
        match self {
            Self::A => 'A',
            Self::B => 'B',
            Self::C => 'C',
            Self::D => 'D',
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Burrow {
    pub amphipods: HashMap<Location, Amphipod>,
    pub room_depth: i16,
}

impl Hash for Burrow {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut sorted: Vec<_> = self.amphipods.iter().collect();
        sorted.sort();
        sorted.hash(state);
    }
}

impl Burrow {
    pub fn room_spaces(&self) -> impl Iterator<Item = Location> + '_ {
        (1..=4).flat_map(|room| {
            (1..=2)
                .rev()
                .map(move |depth| Location::Room(room, depth))
                .find(|loc| !self.amphipods.contains_key(loc))
        })
    }

    pub fn hallway_spaces(&self) -> impl Iterator<Item = Location> + '_ {
        let dests = [1, 2, 4, 6, 8, 10, 11];

        dests
            .into_iter()
            .rev()
            .map(Location::Hallway)
            .filter(move |loc| !self.amphipods.contains_key(loc))
    }

    fn room_no(amph: Amphipod) -> i8 {
        match amph {
            Amphipod::A => 1,
            Amphipod::B => 2,
            Amphipod::C => 3,
            Amphipod::D => 4,
        }
    }

    // Returns true if the amphipod at the given location is snug in its room
    //
    // If there's no Amphipod there, then returns false
    pub fn snug(&self, loc: Location) -> bool {
        let (r, d) = match loc {
            Location::Room(r, d) => (r, d),
            Location::Hallway(_) => return false,
        };

        let amph = match self.amphipods.get(&loc) {
            Some(amph) => *amph,
            None => return false,
        };

        let room_no = Self::room_no(amph);
        if r != room_no {
            return false;
        }

        // It's in its room
        for depth in d + 1..=self.room_depth {
            if self.amphipods.get(&Location::Room(r, depth)) != Some(&amph) {
                return false;
            }
        }

        true
    }

    // Returns a list of (distance, possible destination) for a given amphipod
    // at a given location
    pub fn movements(&self, loc: Location, amph: Amphipod) -> Vec<(i16, Location)> {
        if let Location::Room(n, d) = loc {
            for dabove in 1..d {
                // There is another amphipod above this one; we're stuck in
                if self.amphipods.contains_key(&Location::Room(n, dabove)) {
                    return vec![];
                }
            }
        }

        let mut result = Vec::with_capacity(10);

        let room_no = match amph {
            Amphipod::A => 1,
            Amphipod::B => 2,
            Amphipod::C => 3,
            Amphipod::D => 4,
        };

        // Find an open spot in the destination room, if any
        let mut spot = None;
        for depth in (1..=self.room_depth).rev() {
            let loc = Location::Room(room_no, depth);
            match self.amphipods.get(&loc) {
                None => {
                    spot = Some(loc);
                    break;
                }
                Some(&other) if amph == other => {
                    // There's another amphipod of this sort at this depth, so we could be next
                    continue;
                }
                Some(_) => break,
            }
        }

        let (h1, d1) = loc.to_hallway();
        if let Some(spot) = spot {
            let (h2, d2) = spot.to_hallway();
            if h1 == h2 {
                // We're already in the room we want to be in,

                if d1 == 2
                    || (d1 == 1 && self.amphipods.get(&Location::Room(room_no, 2)) == Some(&amph))
                {
                    // we can allow a zero-step move if we're already in the
                    // room at the bottom or with our partner
                    //
                    // result.push((0, loc));
                    return result;
                }
            } else {
                // See if the hallway is clear from the current location to the destination
                let mut rng = if h1 < h2 { h1 + 1..h2 } else { h2 + 1..h1 };
                if rng.all(|h| !self.amphipods.contains_key(&Location::Hallway(h))) {
                    // Hallway is clear, count this as valid
                    let dist = d1 + (h1 - h2).abs() + d2;
                    result.push((dist, spot));
                }
            }
        }

        if let Location::Hallway(_) = loc {
            // Cannot move from a hallway to a hallway
            return result;
        }

        const FORBIDDEN: [i16; 4] = [3, 5, 7, 9];

        for h in h1 + 1..=11 {
            if FORBIDDEN.contains(&h) {
                // Cannot stop in front of a room
                continue;
            }
            if self.amphipods.contains_key(&Location::Hallway(h)) {
                // Cannot pass another amphipod
                break;
            }
            let dist = d1 + (h - h1);
            result.push((dist, Location::Hallway(h)));
        }

        for h in (1..h1).rev() {
            if FORBIDDEN.contains(&h) {
                // Cannot stop in front of a room
                continue;
            }
            if self.amphipods.contains_key(&Location::Hallway(h)) {
                // Cannot pass another amphipod
                break;
            }
            let dist = d1 + (h1 - h);
            result.push((dist, Location::Hallway(h)));
        }

        result
    }

    // Returns a list of possible (Amphipod, distance, possible destination)
    // movements
    pub fn possibilities(&self) -> Vec<(Amphipod, i16, Burrow)> {
        let mut result = Vec::with_capacity(100);

        for (&loc, &amph) in &self.amphipods {
            for (dist, dest) in self.movements(loc, amph) {
                let mut new = self.clone();
                new.amphipods.remove(&loc);
                new.amphipods.insert(dest, amph);
                result.push((amph, dist, new));
            }
        }

        result
    }

    pub fn min_cost(&self) -> i64 {
        let mut cost = 0i64;
        for (&loc, &amph) in &self.amphipods {
            if self.snug(loc) {
                continue;
            }

            let r = Burrow::room_no(amph);
            // We go for the less-deep destination, it's an approximation
            cost += loc.distance(Location::Room(r, 1)) * amph.energy();
        }
        cost
    }
}

impl FromStr for Burrow {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        simplify(s, parser::only_burrow(s))
    }
}

impl Display for Burrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // #############
        // #...........#
        // ###B#C#B#D###
        //   #A#D#C#A#
        //   #########";

        writeln!(f, "#############")?;

        write!(f, "#")?;
        for i in 1..=11 {
            let loc = Location::Hallway(i);
            let c = match self.amphipods.get(&loc) {
                None => '.',
                Some(a) => a.char(),
            };
            write!(f, "{}", c)?;
        }
        writeln!(f, "#")?;

        for d in 1..=self.room_depth {
            if d == 1 {
                write!(f, "###")?;
            } else {
                write!(f, "  #")?;
            }
            for r in 1..=4 {
                let loc = Location::Room(r, d);
                let c = match self.amphipods.get(&loc) {
                    None => '.',
                    Some(a) => a.char(),
                };
                write!(f, "{}#", c)?;
            }
            if d == 1 {
                writeln!(f, "##")?;
            } else {
                writeln!(f)?;
            }
        }

        write!(f, "  #########")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Possibility {
    energy: i64,
    expected_cost: i64,
    burrow: Burrow,
}

impl PartialOrd for Possibility {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Possibility {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp = self.expected_cost.cmp(&other.expected_cost);
        if cmp.is_ne() {
            // Take the reverse to sort smaller expected costs first
            return cmp.reverse();
        }
        let cmp = self.energy.cmp(&other.energy);
        if cmp.is_ne() {
            // The one with the more energy already is better
            return cmp;
        }

        // Finally, compare the burrows, just to have something
        let mut locs1: Vec<_> = self.burrow.amphipods.iter().collect();
        locs1.sort();
        let mut locs2: Vec<_> = other.burrow.amphipods.iter().collect();
        locs2.sort();
        locs1.cmp(&locs2)
    }
}

impl Possibility {
    pub fn complete(&self) -> bool {
        self.energy == self.expected_cost
    }
}

pub struct Solver {
    queue: BinaryHeap<Possibility>,
    seen: HashSet<Burrow>,
}

impl Solver {
    pub fn new(burrow: Burrow) -> Self {
        let mut queue = BinaryHeap::new();

        let mut seen = HashSet::new();
        seen.insert(burrow.clone());

        let expected_cost = burrow.min_cost();
        queue.push(Possibility {
            energy: 0,
            expected_cost,
            burrow,
        });

        Solver { queue, seen }
    }

    // Take a step forward in the solver. Returns true if there are more steps
    pub fn step(&mut self) -> bool {
        let current = match self.queue.pop() {
            None => return false,
            Some(p) => p,
        };

        if current.complete() {
            info!("Pushing {}, {}", current.energy, current.expected_cost);
            self.queue.push(current);
            return false;
        }

        let possibilities = current.burrow.possibilities();
        for (amph, dist, burrow) in possibilities {
            if self.seen.contains(&burrow) {
                continue;
            }
            self.seen.insert(burrow.clone());

            let energy = current.energy + (dist as i64 * amph.energy());
            let expected_cost = energy + burrow.min_cost();
            self.queue.push(Possibility {
                energy,
                expected_cost,
                burrow,
            });
        }

        true
    }

    pub fn solve(&mut self) -> Option<i64> {
        while self.step() {}

        self.queue.peek().map(|p| p.energy)
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day23.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let s = std::fs::read_to_string(args.input).unwrap();
    let burrow = Burrow::from_str(&s).unwrap();
    let mut solver = Solver::new(burrow);
    let e = solver.solve().unwrap();

    println!("Found {e}");
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use log::info;
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE: &str = r"
        #############
        #...........#
        ###B#C#B#D###
          #A#D#C#A#
          #########";

    #[test]
    fn test_basic() {
        let burrow: Burrow = EXAMPLE.parse().unwrap();
        println!("{}", burrow);
        assert_eq!(burrow.amphipods.len(), 8);
    }

    const PARTIAL_EXAMPLE: &str = r"
        #############
        #C....C...A.#
        ###.#B#.#D###
          #A#B#.#D#
          #########";
    //   123456789012

    #[test]
    fn test_movement() {
        let burrow: Burrow = PARTIAL_EXAMPLE.parse().unwrap();

        let loc = Location::Hallway(10);
        let amph = Amphipod::A;
        assert_eq!(burrow.amphipods.get(&loc), Some(&amph));
        let movements = burrow.movements(loc, amph);
        // println!("{:?}", movements);
        // A@10 cannot move; it can't get to its room, and it can't move to a hallway
        assert_eq!(movements.len(), 0);

        let loc = Location::Room(1, 2);
        let amph = Amphipod::A;
        assert_eq!(burrow.amphipods.get(&loc), Some(&amph));
        let movements = burrow.movements(loc, amph);
        println!("{:?}", movements);
        // It's already snug, so it can't move
        assert_eq!(movements.len(), 0);
        // If it wasn't snug
        // let expected = HashSet::from([
        //     (0i16, Location::Room(1, 2)),
        //     (3, Location::Hallway(4)),
        //     (3, Location::Hallway(2)),
        // ]);
        // assert_eq!(HashSet::from_iter(movements.iter().copied()), expected);

        let loc = Location::Hallway(6);
        let amph = Amphipod::C;
        assert_eq!(burrow.amphipods.get(&loc), Some(&amph));
        let movements = burrow.movements(loc, amph);
        println!("{:?}", movements);
        let expected = HashSet::from([(3i16, Location::Room(3, 2))]);
        assert_eq!(HashSet::from_iter(movements.iter().copied()), expected);
    }

    #[test]
    fn test_solver_steps() {
        let burrow: Burrow = EXAMPLE.parse().unwrap();

        let mut solver = Solver::new(burrow);

        for i in 0..=3922 {
            let p = solver.queue.peek().unwrap();
            let c = p.expected_cost;
            let e = p.energy;
            let min = p.burrow.min_cost();
            assert_eq!(p.energy + min, c);
            info!("Step {:2}:{:5}+{:5} ->{:5}", i, e, min, c);
            info!("{}", p.burrow);

            let stepped = solver.step();
            if !stepped {
                assert_eq!(min, 0);
                assert_eq!(e, c);
                assert_eq!(e, 12521);
                break;
            }
        }
    }

    #[test]
    fn test_solver() {
        let burrow: Burrow = EXAMPLE.parse().unwrap();
        let mut solver = Solver::new(burrow);
        assert_eq!(solver.solve(), Some(12521));
    }

    const EXAMPLE2: &str = r#"
        #############
        #...........#
        ###B#C#B#D###
          #D#C#B#A#
          #D#B#A#C#
          #A#D#C#A#
          #########"#;

    #[test]
    #[ignore]
    fn test_solver2() {
        let burrow: Burrow = EXAMPLE2.parse().unwrap();
        assert_eq!(burrow.amphipods.len(), 16);
        println!("{}", burrow);
        let mut solver = Solver::new(burrow);
        info!(
            "Made solver, initial cost {}",
            solver.queue.peek().unwrap().expected_cost
        );

        for i in 0.. {
            let p = solver.queue.peek().unwrap();
            let c = p.expected_cost;
            let e = p.energy;
            let min = p.burrow.min_cost();
            assert_eq!(p.energy + min, c);
            let log_level = if i % 1000 == 0 {
                log::Level::Info
            } else {
                log::Level::Debug
            };
            log::log!(
                log_level,
                "Step {:2}:{:5}+{:5} ->{:5} ({})\n{}",
                i,
                e,
                min,
                c,
                solver.queue.len(),
                p.burrow
            );

            let stepped = solver.step();
            if !stepped {
                assert_eq!(min, 0);
                assert_eq!(e, c);
                assert_eq!(e, 44169);
                break;
            }
        }

        // assert_eq!(solver.solve(), Some(44169));
    }
}
