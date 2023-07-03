use std::fmt::{self, Display};
use std::path::PathBuf;
use std::{collections::VecDeque, str::FromStr};

use anyhow::anyhow;
use clap::Parser;
use log::debug;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sequence {
    // Nibbles remaining
    nibbles: VecDeque<u8>,
    // Unprocessed bits from the last nibble
    bits: VecDeque<bool>,
}

fn bits64(bits: &[bool]) -> u64 {
    assert!(bits.len() <= 64);
    let mut n = 0u64;
    for &bit in bits {
        n <<= 1;
        n |= bit as u64;
    }

    n
}

impl Sequence {
    pub fn new<V: Into<VecDeque<u8>>>(nibbles: V) -> Self {
        Self {
            nibbles: nibbles.into(),
            bits: VecDeque::new(),
        }
    }

    pub fn from_hex_bytes<I: IntoIterator<Item = u8>>(iter: I) -> anyhow::Result<Self> {
        let mut nibbles = VecDeque::new();
        for (ix, nibble) in iter.into_iter().enumerate() {
            if !(b'0'..=b'F').contains(&nibble) {
                return Err(anyhow!("Unexpected nibble {nibble} at index {ix}"));
            }

            nibbles.push_back(nibble - b'0');
        }

        Ok(Self::new(nibbles))
    }

    fn move_nibble(&mut self) -> bool {
        let nibble = match self.nibbles.pop_front() {
            Some(n) => n,
            None => return false,
        };
        self.bits
            .extend((0..4).rev().map(|ix| (nibble >> ix) & 1 == 1));

        true
    }

    pub fn pop_bit(&mut self) -> anyhow::Result<bool> {
        if self.bits.is_empty() && !self.move_nibble() {
            return Err(anyhow!("No more bits"));
        }

        Ok(self.bits.pop_front().unwrap())
    }

    pub fn pop_bits(&mut self, n: usize) -> anyhow::Result<Vec<bool>> {
        while self.bits.len() < n {
            if !self.move_nibble() {
                break;
            };
        }

        if self.bits.len() < n {
            return Err(anyhow!(
                "Not enough bits: {bits:?} < {n}",
                bits = self.bits,
                n = n
            ));
        }

        let mut remainder = self.bits.split_off(n);
        std::mem::swap(&mut remainder, &mut self.bits);

        Ok(remainder.into())
    }

    pub fn pop_header(&mut self) -> anyhow::Result<(u8, u8)> {
        let bits = self.pop_bits(6)?;
        Ok((bits64(&bits[0..3]) as u8, bits64(&bits[3..6]) as u8))
    }

    pub fn parse_literal(&mut self) -> anyhow::Result<Literal> {
        let mut bits = Vec::with_capacity(64);
        loop {
            let cur = self.pop_bits(5)?;
            bits.extend(&cur[1..]);
            if !cur[0] {
                break;
            }
        }

        if bits.len() > 64 {
            return Err(anyhow!("Literal too long ({l}): {bits:?}", l = bits.len()));
        }

        Ok(Literal(bits64(&bits)))
    }

    pub fn remainder_zero(&self) -> bool {
        return self.bits.iter().all(|&b| !b) && self.nibbles.iter().all(|&n| n == 0);
    }

    pub fn bits_count(&self) -> usize {
        self.nibbles.len() * 4 + self.bits.len()
    }

    pub fn parse_packet(&mut self) -> anyhow::Result<Packet> {
        let (v, t) = self.pop_header()?;
        if t == 4 {
            return Ok(Packet {
                version: v,
                payload: Payload::Literal(self.parse_literal()?),
            });
        }

        // It's an operator
        let op = if self.pop_bit()? {
            // sub-packets
            let l = self.pop_bits(11)?;
            let n = bits64(&l) as usize;
            debug!("Operator (sub-packets): {v} {t} {n}", v = v, t = t, n = n);
            self.parse_operator_packetlength(t, n)?
        } else {
            let l = self.pop_bits(15)?;
            let n = bits64(&l) as usize;
            debug!("Operator (bits):        {v} {t} {n}", v = v, t = t, n = n);
            self.parse_operator_bitlength(t, n)?
        };
        Ok(Packet {
            version: v,
            payload: Payload::Operator(op),
        })
    }

    fn parse_operator_bitlength(&mut self, typ: u8, n: usize) -> anyhow::Result<Operator> {
        let mut components = Vec::new();
        assert!(self.bits_count() >= n);
        let remainder = self.bits_count() - n;
        while self.bits_count() > remainder {
            components.push(self.parse_packet()?);
        }

        Ok(Operator { typ, components })
    }

    fn parse_operator_packetlength(&mut self, typ: u8, n: usize) -> anyhow::Result<Operator> {
        let mut components = Vec::new();
        for _ in 0..n {
            components.push(self.parse_packet()?);
        }

        Ok(Operator { typ, components })
    }
}

impl FromStr for Sequence {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: Result<VecDeque<u8>, anyhow::Error> = s
            .trim()
            .chars()
            .map(|s| {
                s.to_digit(16)
                    .map(|n| n as u8)
                    .ok_or_else(|| anyhow!("Invalid digit: {s}"))
            })
            .collect();

        Ok(Self::new(bytes?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Literal(u64);

impl Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Packet {
    pub version: u8,
    pub payload: Payload,
}

impl Packet {
    pub fn version_sum(&self) -> u64 {
        self.version as u64
            + match &self.payload {
                Payload::Literal(_) => 0,
                Payload::Operator(o) => o.components.iter().map(|c| c.version_sum()).sum(),
            }
    }

    pub fn evaluate(&self) -> i64 {
        let (t, c) = match self.payload {
            Payload::Literal(Literal(n)) => return n as i64,
            Payload::Operator(Operator {
                typ: t,
                components: ref c,
            }) => (t, c),
        };

        let mut inner_values = c.iter().map(|c| c.evaluate());
        let (l, r) = match t {
            0 => return inner_values.sum(),
            1 => return inner_values.product(),
            2 => return inner_values.min().unwrap_or(0),
            3 => return inner_values.max().unwrap_or(0),
            5..=7 => (inner_values.next().unwrap(), inner_values.next().unwrap()),
            _ => panic!("Invalid operator type: {}", t),
        };

        let found = match t {
            5 => l > r,
            6 => l < r,
            7 => l == r,
            _ => unreachable!(),
        };

        found as i64
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "P{}:{}", self.version, self.payload)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Payload {
    Literal(Literal),
    Operator(Operator),
}

impl Display for Payload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Payload::Literal(l) => write!(f, "L{}", l),
            Payload::Operator(o) => write!(f, "O{}", o),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Operator {
    typ: u8,
    components: Vec<Packet>,
}

impl Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:[", self.typ)?;
        for (ix, c) in self.components.iter().enumerate() {
            if ix > 0 {
                write!(f, ",")?;
            }
            write!(f, "{}", c)?;
        }

        write!(f, "]")
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day16.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let s = std::fs::read_to_string(args.input).unwrap();
    let mut seq = s.trim().parse::<Sequence>().unwrap();
    let packet = seq.parse_packet().unwrap();

    let vs = packet.version_sum();
    let value = packet.evaluate();
    println!("Found version sum {vs}, value {value}");
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_basic() {
        let example = r"D2FE28";
        let mut seq: Sequence = example.parse().unwrap();
        assert_eq!(seq.nibbles, vec![0xD, 0x2, 0xF, 0xE, 0x2, 0x8]);

        assert_eq!(seq.pop_bits(3).unwrap(), vec![true, true, false]);
        assert_eq!(seq.pop_bits(3).unwrap(), vec![true, false, false]);
        assert_eq!(
            seq.pop_bits(5).unwrap(),
            vec![true, false, true, true, true]
        );
        assert_eq!(
            seq.pop_bits(5).unwrap(),
            vec![true, true, true, true, false]
        );
        assert_eq!(
            seq.pop_bits(5).unwrap(),
            vec![false, false, true, false, true]
        );

        seq = example.parse().unwrap();
        let (v, t) = seq.pop_header().unwrap();
        assert_eq!((v, t), (6, 4));
        let lit = seq.parse_literal().unwrap();
        assert_eq!(lit, Literal(2021));
        assert!(seq.remainder_zero());
    }

    #[test]
    fn test_packets() {
        let example2 = r"38006F45291200";
        let mut seq: Sequence = example2.parse().unwrap();

        let pkt = seq.parse_packet().unwrap();
        assert!(seq.remainder_zero());
        assert_eq!(
            pkt,
            Packet {
                version: 1,
                payload: Payload::Operator(Operator {
                    typ: 6,
                    components: vec![
                        Packet {
                            version: 6,
                            payload: Payload::Literal(Literal(10))
                        },
                        Packet {
                            version: 2,
                            payload: Payload::Literal(Literal(20))
                        }
                    ]
                })
            }
        );

        let packet_str = format!("{}", pkt);
        assert_eq!(packet_str, "P1:O6:[P6:L10,P2:L20]");

        let example3 = r"EE00D40C823060";
        let mut seq: Sequence = example3.parse().unwrap();

        let pkt = seq.parse_packet().unwrap();
        assert!(seq.remainder_zero());
        assert_eq!(
            pkt,
            Packet {
                version: 7,
                payload: Payload::Operator(Operator {
                    typ: 3,
                    components: vec![
                        Packet {
                            version: 2,
                            payload: Payload::Literal(Literal(1))
                        },
                        Packet {
                            version: 4,
                            payload: Payload::Literal(Literal(2))
                        },
                        Packet {
                            version: 1,
                            payload: Payload::Literal(Literal(3))
                        }
                    ]
                })
            }
        );

        let packet_str = format!("{}", pkt);
        assert_eq!(packet_str, "P7:O3:[P2:L1,P4:L2,P1:L3]");

        let example4 = r"8A004A801A8002F478";
        let mut seq: Sequence = example4.parse().unwrap();

        let pkt = seq.parse_packet().unwrap();
        assert!(seq.remainder_zero());
        let packet_str = format!("{}", pkt);
        assert_eq!(packet_str, "P4:O2:[P1:O2:[P5:O2:[P6:L15]]]");
        assert_eq!(pkt.version_sum(), 16);

        let example4 = r"620080001611562C8802118E34";
        let mut seq: Sequence = example4.parse().unwrap();
        let pkt = seq.parse_packet().unwrap();
        assert!(seq.remainder_zero());
        assert_eq!(pkt.version_sum(), 12);

        let example4 = r"C0015000016115A2E0802F182340";
        let mut seq: Sequence = example4.parse().unwrap();
        let pkt = seq.parse_packet().unwrap();
        assert!(seq.remainder_zero());
        assert_eq!(pkt.version_sum(), 23);

        let example4 = r"A0016C880162017C3686B18A3D4780";
        let mut seq: Sequence = example4.parse().unwrap();
        let pkt = seq.parse_packet().unwrap();
        assert!(seq.remainder_zero());
        assert_eq!(pkt.version_sum(), 31);
    }

    #[test]
    fn test_evaluate() {
        let examples: Vec<(&str, i64)> = vec![
            ("C200B40A82", 3),
            ("04005AC33890", 54),
            ("880086C3E88112", 7),
            ("CE00C43D881120", 9),
            ("D8005AC2A8F0", 1),
            ("F600BC2D8F", 0),
            ("9C005AC2F8F0", 0),
            ("9C0141080250320F1802104A08", 1),
        ];

        for (n, &(s, expected)) in examples.iter().enumerate() {
            let mut seq: Sequence = s.parse().unwrap();
            let pkt = seq.parse_packet().unwrap();
            assert!(seq.remainder_zero());
            assert_eq!(pkt.evaluate(), expected, "Failed example {n}: {s}");
        }
    }
}
