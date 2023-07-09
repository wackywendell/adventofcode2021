use std::collections::{HashMap, HashSet, VecDeque};

use std::ops::Sub;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use log::debug;

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
pub struct Vector(i64, i64, i64);

impl Vector {
    pub fn manhattan(self) -> i64 {
        let Vector(x, y, z) = self;
        x.abs() + y.abs() + z.abs()
    }

    pub fn rotate_x(self) -> Self {
        let Vector(x, y, z) = self;
        let (x, y, z) = (
            RX[0][0] * x + RX[0][1] * y + RX[0][2] * z,
            RX[1][0] * x + RX[1][1] * y + RX[1][2] * z,
            RX[2][0] * x + RX[2][1] * y + RX[2][2] * z,
        );
        Vector(x, y, z)
    }
    pub fn rotate_y(self) -> Self {
        let Vector(x, y, z) = self;
        let (x, y, z) = (
            RY[0][0] * x + RY[0][1] * y + RY[0][2] * z,
            RY[1][0] * x + RY[1][1] * y + RY[1][2] * z,
            RY[2][0] * x + RY[2][1] * y + RY[2][2] * z,
        );
        Vector(x, y, z)
    }
    pub fn rotate_z(self) -> Self {
        let Vector(x, y, z) = self;
        let (x, y, z) = (
            RZ[0][0] * x + RZ[0][1] * y + RZ[0][2] * z,
            RZ[1][0] * x + RZ[1][1] * y + RZ[1][2] * z,
            RZ[2][0] * x + RZ[2][1] * y + RZ[2][2] * z,
        );
        Vector(x, y, z)
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

    pub fn rotation(self, n: usize) -> Vector {
        let Vector(x, y, z) = self;
        match n % 24 {
            0 => Vector(-z, -y, -x),
            1 => Vector(-z, -x, y),
            2 => Vector(-z, x, -y),
            3 => Vector(-z, y, x),
            4 => Vector(-y, -z, x),
            5 => Vector(-y, -x, -z),
            6 => Vector(-y, x, z),
            7 => Vector(-y, z, -x),
            8 => Vector(-x, -z, -y),
            9 => Vector(-x, -y, z),
            10 => Vector(-x, y, -z),
            11 => Vector(-x, z, y),
            12 => Vector(x, -z, y),
            13 => Vector(x, -y, -z),
            14 => Vector(x, y, z),
            15 => Vector(x, z, -y),
            16 => Vector(y, -z, -x),
            17 => Vector(y, -x, z),
            18 => Vector(y, x, -z),
            19 => Vector(y, z, x),
            20 => Vector(z, -y, x),
            21 => Vector(z, -x, -y),
            22 => Vector(z, x, y),
            23 => Vector(z, y, -x),
            _ => unreachable!(),
        }
    }

    pub fn rotations(self) -> [Vector; 24] {
        let Vector(x, y, z) = self;
        [
            Vector(-z, -y, -x),
            Vector(-z, -x, y),
            Vector(-z, x, -y),
            Vector(-z, y, x),
            Vector(-y, -z, x),
            Vector(-y, -x, -z),
            Vector(-y, x, z),
            Vector(-y, z, -x),
            Vector(-x, -z, -y),
            Vector(-x, -y, z),
            Vector(-x, y, -z),
            Vector(-x, z, y),
            Vector(x, -z, y),
            Vector(x, -y, -z),
            Vector(x, y, z),
            Vector(x, z, -y),
            Vector(y, -z, -x),
            Vector(y, -x, z),
            Vector(y, x, -z),
            Vector(y, z, x),
            Vector(z, -y, x),
            Vector(z, -x, -y),
            Vector(z, x, y),
            Vector(z, y, -x),
        ]
    }
}

impl Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Self::Output {
        let Vector(x1, y1, z1) = self;
        let Vector(x2, y2, z2) = rhs;
        Vector(x1 - x2, y1 - y2, z1 - z2)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Overlap {
    rot: usize,
    diff: Vector,
    pairs: HashSet<(usize, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Region {
    pub id: u64,
    pub positions: Vec<Vector>,
}

impl Region {
    fn dist_hash(p1: Vector, p2: Vector) -> Vector {
        let Vector(x, y, z) = p1 - p2;

        let mut ns = vec![x.abs(), y.abs(), z.abs()];
        ns.sort();

        Vector(ns[0], ns[1], ns[2])
    }

    pub fn dists_renormed(&self) -> HashMap<Vector, Vec<(usize, usize)>> {
        let mut hashes: HashMap<Vector, Vec<(usize, usize)>> = HashMap::new();
        for (ix2, &p2) in self.positions.iter().enumerate() {
            for (ix1, &p1) in self.positions[..ix2].iter().enumerate() {
                hashes
                    .entry(Self::dist_hash(p1, p2))
                    .or_default()
                    .push((ix1, ix2));
            }
        }

        hashes
    }

    pub fn dists(&self) -> HashMap<Vector, Vec<(usize, usize)>> {
        let mut dists: HashMap<Vector, Vec<(usize, usize)>> = HashMap::new();
        for (ix2, &p2) in self.positions.iter().enumerate() {
            for (ix1, &p1) in self.positions[..ix2].iter().enumerate() {
                dists.entry(p2 - p1).or_default().push((ix1, ix2));
            }
        }

        dists
    }

    // Finds the maximum overlap between self and rhs based on rotations and translations of rhs.
    // If no overlap of >=2 2 points is found, returns None.
    pub fn overlap(&self, rhs: &Region) -> Option<Overlap> {
        // (rotation: usize, diff: Vector) -> HashSet<(index1, index2)>, where
        // index1 and index2 are equivalent points and diff is the distance
        // between the two pairs
        let mut overlaps: HashMap<(usize, Vector), HashSet<(usize, usize)>> = HashMap::new();
        let dists1 = self.dists();
        let dists2 = rhs.dists();
        for n in 0..24 {
            // rotate rhs by n to match with self
            for (&d2, v2) in dists2.iter() {
                let d1 = d2.rotation(n);
                if let Some(v1) = dists1.get(&d1) {
                    for &p1 in v1.iter() {
                        let v1a = self.positions[p1.0];
                        let v1b = self.positions[p1.1];
                        assert_eq!(v1b - v1a, d1);
                        for &p2 in v2.iter() {
                            let v2a = rhs.positions[p2.0].rotation(n);
                            let v2b = rhs.positions[p2.1].rotation(n);
                            assert_eq!(v2b - v2a, d1);

                            let diff = v2a - v1a;

                            debug!(
                                "d1: {d1}, d2: {d2}, diff: {diff}, diff2: {diff2}",
                                diff2 = v2b - v1b
                            );
                            assert_eq!(diff, v2b - v1b);
                            assert_eq!(
                                (rhs.positions[p2.0].rotation(n) - diff),
                                self.positions[p1.0]
                            );
                            assert_eq!(
                                (rhs.positions[p2.1].rotation(n) - diff),
                                self.positions[p1.1]
                            );
                            overlaps.entry((n, diff)).or_default().insert((p1.0, p2.0));
                            overlaps.entry((n, diff)).or_default().insert((p1.1, p2.1));
                        }
                    }
                }
            }
        }
        // diff = rot(rhs, n) - self
        // self = rot(rhs, n) - diff
        let ((rot, diff), pairs) = overlaps
            .into_iter()
            .max_by_key(|(_rot, pairs)| pairs.len())?;

        let skip_ixs = pairs.iter().map(|&(_, ix2)| ix2).collect::<HashSet<_>>();

        let positions: Vec<Vector> = rhs
            .positions
            .iter()
            .enumerate()
            .filter_map(|(ix, pos)| {
                if skip_ixs.contains(&ix) {
                    None
                } else {
                    Some(pos.rotation(rot) - diff)
                }
            })
            .collect();

        debug!(
            "Found {} points, skipping {}; {} -> {}",
            pairs.len(),
            skip_ixs.len(),
            rhs.positions.len(),
            positions.len()
        );

        let _region = Region {
            positions,
            id: rhs.id,
        };

        Some(Overlap { rot, diff, pairs })
    }

    pub fn apply(&mut self, overlap: &Overlap) {
        for pos in self.positions.iter_mut() {
            *pos = pos.rotation(overlap.rot) - overlap.diff;
        }
    }
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

pub fn parse_position_line(input: &str) -> IResult<&str, Vector> {
    let (remainder, (x, y, z)) = tuple((
        parse_int,
        preceded(char(','), parse_int),
        preceded(char(','), parse_int),
    ))(input)?;

    let pos = Vector(x, y, z);
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

pub struct Regions(Vec<Region>);

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

impl Regions {
    pub fn reduce(&self, min_overlap: usize) -> Combined {
        let first = &self.0[0];
        let mut diffs: HashMap<u64, Vector> = HashMap::from_iter(vec![(first.id, Vector(0, 0, 0))]);
        let mut unmerged: HashSet<&Region> = self.0.iter().skip(1).collect();

        // Scanners properly rotated and translated, to be checked against those not yet merged in
        let mut left_sides = VecDeque::from(vec![first.clone()]);

        let mut known_points: HashSet<Vector> = HashSet::from_iter(first.positions.iter().copied());

        while let Some(next) = left_sides.pop_back() {
            let mut merged = HashSet::new();
            for &rhs in &unmerged {
                let Some(overlap) = next.overlap(rhs) else {
                    debug!("Skipping {} -> {} (no overlap)", next.id, rhs.id);
                    continue;
                };
                if overlap.pairs.len() < min_overlap {
                    debug!(
                        "Can't merge in {} -> {} (only {} overlap)",
                        next.id,
                        rhs.id,
                        overlap.pairs.len()
                    );
                }

                debug!(
                    "Merging {} -> {} (overlap {})",
                    next.id,
                    rhs.id,
                    overlap.pairs.len()
                );
                merged.insert(rhs);

                let mut new_left = rhs.clone();
                new_left.apply(&overlap);
                known_points.extend(new_left.positions.iter().copied());
                diffs.insert(new_left.id, overlap.diff);
                left_sides.push_back(new_left);
            }
            unmerged = unmerged.difference(&merged).copied().collect();
        }

        if !unmerged.is_empty() {
            debug!("Unmerged regions: {:?}", unmerged);
            return Combined::default();
        }

        Combined {
            positions: known_points,
            scanners: diffs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Combined {
    pub positions: HashSet<Vector>,
    pub scanners: HashMap<u64, Vector>,
}

impl Combined {
    pub fn max_distance(&self) -> i64 {
        let mut max = 0;
        for (&i1, &v1) in self.scanners.iter() {
            for (&i2, &v2) in self.scanners.iter() {
                if i2 <= i1 {
                    continue;
                }

                let d = (v2 - v1).manhattan();
                max = max.max(d);
            }
        }

        max
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
    let p = Vector(1, 2, 3);

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
    let s = std::fs::read_to_string(args.input).unwrap();
    let regions = s.parse::<Regions>().unwrap();
    let all = regions.reduce(12);

    println!(
        "Found {} points, max distance {}",
        all.positions.len(),
        all.max_distance()
    );
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

    fn example_regions() -> Regions {
        Regions::from_str(EXAMPLE.trim()).unwrap()
    }

    #[test]
    fn test_overlap14() {
        let regions = example_regions();
        let r1 = &regions.0[1];
        assert_eq!(r1.id, 1);
        let r4 = &regions.0[4];
        assert_eq!(r4.id, 4);

        let overlap = r1.overlap(r4).unwrap();
        assert_eq!(overlap.pairs.len(), 12);

        let mut moved = r4.clone();
        moved.apply(&overlap);
        let mut all_points = moved.positions.iter().cloned().collect::<HashSet<_>>();
        all_points.extend(r1.positions.iter().cloned());
        assert_eq!(
            all_points.len(),
            r1.positions.len() + r4.positions.len() - 12
        );
    }

    #[test]
    fn test_overlaps() {
        let regions = example_regions();
        let r0 = &regions.0[0];
        assert_eq!(r0.id, 0);
        let r1 = &regions.0[1];
        assert_eq!(r1.id, 1);

        let overlap = r0.overlap(r1).unwrap();

        assert_eq!(overlap.pairs.len(), 12);
    }

    #[test]
    fn test_reduce() {
        let regions = example_regions();
        let reduced = regions.reduce(12);
        assert_eq!(reduced.positions.len(), 79);
        assert_eq!(reduced.max_distance(), 3621);
    }
}
