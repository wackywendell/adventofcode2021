use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use log::debug;

use adventofcode2021::parse;
use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1};
use nom::combinator::{complete, opt};
use nom::error::Error;
use nom::multi::{many0, many1, separated_list1};
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::{Finish, IResult};
use parse_display::{Display, FromStr};

type Matrix = [[i64; 3]; 3];

const RX: Matrix = [[1, 0, 0], [0, 0, -1], [0, 1, 0]];
const RY: Matrix = [[0, 0, 1], [0, 1, 0], [-1, 0, 0]];
const RZ: Matrix = [[0, -1, 0], [1, 0, 0], [0, 0, 1]];

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, FromStr, Display)]
#[display("({0},{1},{2})")]
pub struct Position(i64, i64, i64);

impl Position {
    pub fn rotate_x(self) -> Self {
        let Position(x, y, z) = self;
        let (x, y, z) = (
            RX[0][0] * x + RX[0][1] * y + RX[0][2] * z,
            RX[1][0] * x + RX[1][1] * y + RX[1][2] * z,
            RX[2][0] * x + RX[2][1] * y + RX[2][2] * z,
        );
        Position(x, y, z)
    }
    pub fn rotate_y(self) -> Self {
        let Position(x, y, z) = self;
        let (x, y, z) = (
            RY[0][0] * x + RY[0][1] * y + RY[0][2] * z,
            RY[1][0] * x + RY[1][1] * y + RY[1][2] * z,
            RY[2][0] * x + RY[2][1] * y + RY[2][2] * z,
        );
        Position(x, y, z)
    }
    pub fn rotate_z(self) -> Self {
        let Position(x, y, z) = self;
        let (x, y, z) = (
            RZ[0][0] * x + RZ[0][1] * y + RZ[0][2] * z,
            RZ[1][0] * x + RZ[1][1] * y + RZ[1][2] * z,
            RZ[2][0] * x + RZ[2][1] * y + RZ[2][2] * z,
        );
        Position(x, y, z)
    }

    pub fn rotate(self, nx: u8, ny: u8, nz: u8) -> Self {
        let mut p = self;
        for _ in 0..(nx % 4) {
            p = p.rotate_x();
        }
        for _ in 0..(ny % 4) {
            p = p.rotate_y();
        }
        for _ in 0..(nz % 4) {
            p = p.rotate_z();
        }
        p
    }

    pub fn rotations(self) -> [Position; 24] {
        let Position(x, y, z) = self;
        [
            Position(-z, -y, -x),
            Position(-z, -x, y),
            Position(-z, x, -y),
            Position(-z, y, x),
            Position(-y, -z, x),
            Position(-y, -x, -z),
            Position(-y, x, z),
            Position(-y, z, -x),
            Position(-x, -z, -y),
            Position(-x, -y, z),
            Position(-x, y, -z),
            Position(-x, z, y),
            Position(x, -z, y),
            Position(x, -y, -z),
            Position(x, y, z),
            Position(x, z, -y),
            Position(y, -z, -x),
            Position(y, -x, z),
            Position(y, x, -z),
            Position(y, z, x),
            Position(z, -y, x),
            Position(z, -x, -y),
            Position(z, x, y),
            Position(z, y, -x),
        ]
    }
}

pub struct Region {
    pub id: u64,
    pub positions: Vec<Position>,
}

pub fn parse_scanner_line(input: &str) -> IResult<&str, u64> {
    let mut digitizer = delimited(tag("--- scanner "), digit1, tag(" ---"));
    let (remaining, digits) = digitizer(input)?;
    let id = digits.parse::<u64>().unwrap();
    Ok((remaining, id))
}

pub fn parse_int(input: &str) -> IResult<&str, i64> {
    let (remainder, (neg, digits)) = tuple((opt(char('-')), digit1))(input)?;
    let n = digits.parse::<i64>().unwrap();
    if neg.is_none() {
        Ok((remainder, n))
    } else {
        Ok((remainder, -n))
    }
}

pub fn parse_position_line(input: &str) -> IResult<&str, Position> {
    let (remainder, (x, y, z)) = tuple((
        parse_int,
        preceded(char(','), parse_int),
        preceded(char(','), parse_int),
    ))(input)?;
    // let (remainder, _) = opt(char('\n'))(remainder)?;

    let pos = Position(x, y, z);
    Ok((remainder, pos))
}

pub fn parse_region(input: &str) -> IResult<&str, Region> {
    let (remainder, id) = delimited(many0(char(' ')), parse_scanner_line, char('\n'))(input)?;
    let (remainder, positions) = separated_list1(
        char('\n'), // Separated by newline + spaces
        preceded(many0(char(' ')), parse_position_line),
    )(remainder)?;
    Ok((remainder, Region { id, positions }))
}

pub fn parse_regions(input: &str) -> IResult<&str, Vec<Region>> {
    (separated_list1(many1(pair(char('\n'), many0(char(' ')))), parse_region))(input)
}

struct Regions(Vec<Region>);

impl FromStr for Regions {
    // the error must be owned as well
    type Err = Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match complete(parse_regions)(s).finish() {
            Ok((_remaining, regions)) => Ok(Regions(regions)),
            Err(Error { input, code }) => Err(Error {
                input: input.to_string(),
                code,
            }),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day19.txt")]
    input: PathBuf,
}

// Used to generate rotations above
#[allow(dead_code)]
fn print_rotations() {
    let p = Position(1, 2, 3);

    let mut positions = Vec::new();

    for nx in 0..4 {
        for ny in 0..4 {
            for nz in 0..4 {
                positions.push(p.rotate(nx, ny, nz));
            }
        }
    }

    positions.sort();
    positions.dedup();

    println!("Found {} positions", positions.len());

    for p in positions {
        println!("{}", p);
    }
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let file = File::open(args.input).unwrap();
    let buf = BufReader::new(file);
    let nums: Vec<i64> = parse::buffer(buf).unwrap();

    println!("Found {length} lines: {nums:?}", length = nums.len());
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    const EXAMPLE: &str = r###"
    --- scanner 0 ---
    404,-588,-901
    528,-643,409
    -838,591,734
    390,-675,-793
    -537,-823,-458
    -485,-357,347
    -345,-311,381
    -661,-816,-575
    -876,649,763
    -618,-824,-621
    553,345,-567
    474,580,667
    -447,-329,318
    -584,868,-557
    544,-627,-890
    564,392,-477
    455,729,728
    -892,524,684
    -689,845,-530
    423,-701,434
    7,-33,-71
    630,319,-379
    443,580,662
    -789,900,-551
    459,-707,401
    
    --- scanner 1 ---
    686,422,578
    605,423,415
    515,917,-361
    -336,658,858
    95,138,22
    -476,619,847
    -340,-569,-846
    567,-361,727
    -460,603,-452
    669,-402,600
    729,430,532
    -500,-761,534
    -322,571,750
    -466,-666,-811
    -429,-592,574
    -355,545,-477
    703,-491,-529
    -328,-685,520
    413,935,-424
    -391,539,-444
    586,-435,557
    -364,-763,-893
    807,-499,-711
    755,-354,-619
    553,889,-390
    
    --- scanner 2 ---
    649,640,665
    682,-795,504
    -784,533,-524
    -644,584,-595
    -588,-843,648
    -30,6,44
    -674,560,763
    500,723,-460
    609,671,-379
    -555,-800,653
    -675,-892,-343
    697,-426,-610
    578,704,681
    493,664,-388
    -671,-858,530
    -667,343,800
    571,-461,-707
    -138,-166,112
    -889,563,-600
    646,-828,498
    640,759,510
    -630,509,768
    -681,-892,-333
    673,-379,-804
    -742,-814,-386
    577,-820,562
    
    --- scanner 3 ---
    -589,542,597
    605,-692,669
    -500,565,-823
    -660,373,557
    -458,-679,-417
    -488,449,543
    -626,468,-788
    338,-750,-386
    528,-832,-391
    562,-778,733
    -938,-730,414
    543,643,-506
    -524,371,-870
    407,773,750
    -104,29,83
    378,-903,-323
    -778,-728,485
    426,699,580
    -438,-605,-362
    -469,-447,-387
    509,732,623
    647,635,-688
    -868,-804,481
    614,-800,639
    595,780,-596
    
    --- scanner 4 ---
    727,592,562
    -293,-554,779
    441,611,-461
    -714,465,-776
    -743,427,-804
    -660,-479,-426
    832,-632,460
    927,-485,-438
    408,393,-506
    466,436,-512
    110,16,151
    -258,-428,682
    -393,719,612
    -211,-452,876
    808,-476,-593
    -575,615,604
    -485,667,467
    -680,325,-822
    -627,-443,-432
    872,-547,-609
    833,512,582
    807,604,487
    839,-516,451
    891,-625,532
    -652,-548,-490
    30,-46,-14
    "###;

    #[test]
    fn test_basic() {
        let lines = EXAMPLE.trim().lines().collect::<Vec<_>>();
        parse_scanner_line(lines[0].trim()).unwrap();
        parse_position_line(lines[1].trim()).unwrap();
        let (r, _) = preceded(many0(char(' ')), parse_position_line)(lines[1]).unwrap();
        assert_eq!(r, "");
        let (r, _) = preceded(many0(char(' ')), parse_position_line)(lines[2]).unwrap();
        assert_eq!(r, "");
        let (rem, region) = parse_region(EXAMPLE.trim()).unwrap();
        assert_eq!(region.positions.len(), 25);

        assert!(rem.trim_start().starts_with("--- scanner 1 ---"));

        let (rem, regions) = parse_regions(EXAMPLE.trim()).unwrap();
        assert_eq!(rem, "");
        assert_eq!(regions.len(), 5);

        let regions = Regions::from_str(EXAMPLE.trim()).unwrap();
        assert_eq!(regions.0.len(), 5);
    }
}
