use adventofcode2021::parse;
use bitvec::prelude as bits;
use std::fs::File;
use std::io::BufReader;
use std::iter::repeat;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use log::debug;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiagnosticReport<const N: usize>(Vec<Observation<N>>);

impl<const N: usize> DiagnosticReport<N> {
    pub fn power(&self) -> (u16, u16) {
        let mut summed: Vec<usize> = repeat(0).take(N).collect();

        for &obs in &self.0 {
            for (ix, b) in obs.bools().enumerate() {
                if b {
                    summed[ix] += 1
                }
            }
        }

        let mut gamma = 0u16;
        let mut epsilon = 0u16;
        for &cnt in &summed {
            gamma <<= 1;
            epsilon <<= 1;
            if cnt > self.0.len() / 2 {
                gamma |= 1;
            } else {
                epsilon |= 1;
            }
        }

        (gamma, epsilon)
    }

    fn popular_bit(observations: impl IntoIterator<Item = Observation<N>>, ix: usize) -> bool {
        let mut cnt = 0;
        let mut total = 0;
        for o in observations {
            total += 1;
            if *o.0.get(ix).unwrap() {
                cnt += 1;
            }
        }

        cnt >= total - cnt
    }

    pub fn life(&self) -> (u16, u16) {
        let mut oxygens = self.0.clone();
        let mut co2 = self.0.clone();

        for ix in 0..N {
            let bit = DiagnosticReport::popular_bit(oxygens.iter().copied(), ix);
            oxygens.retain(|&n| n.0.get(ix).as_deref().copied() == Some(bit));

            if co2.len() > 1 {
                let bit = !DiagnosticReport::popular_bit(co2.iter().copied(), ix);
                co2.retain(|&n| n.0.get(ix).as_deref().copied() == Some(bit));
            }
        }

        if oxygens.len() != 1 {
            panic!("Expected 1 oxygen {:?}", oxygens);
        }
        if co2.len() != 1 {
            panic!("Expected 1 co2 {:?}", co2);
        }

        (oxygens[0].into(), co2[0].into())
    }
}

impl<const N: usize> FromIterator<Observation<N>> for DiagnosticReport<N> {
    fn from_iter<T: IntoIterator<Item = Observation<N>>>(iter: T) -> Self {
        DiagnosticReport(Vec::from_iter(iter))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Observation<const N: usize>(bits::BitArray<u16, bits::Msb0>);

impl<const N: usize> Observation<N> {
    pub fn bools(&self) -> impl Iterator<Item = bool> + '_ {
        self.0.iter().take(N).map(|r| *r)
    }
}

impl<const N: usize> FromIterator<bool> for Observation<N> {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        if N > 16 {
            panic!("N={N} too large");
        }
        let mut arr: bits::BitArray<u16, bits::Msb0> = bits::BitArray::ZERO;
        for (ix, b) in iter.into_iter().enumerate() {
            if b {
                arr.set(ix, b)
            }
        }

        Observation(arr)
    }
}

impl<const N: usize> From<u16> for Observation<N> {
    fn from(value: u16) -> Self {
        Observation(From::from(value << (16 - N)))
    }
}

impl<const N: usize> From<Observation<N>> for u16 {
    fn from(value: Observation<N>) -> Self {
        if N > 16 {
            panic!("N={N} too large");
        }

        value.0.as_raw_slice()[0] >> (16 - N)
    }
}

impl<const N: usize> FromStr for Observation<N> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != N {
            return Err(anyhow::anyhow!("Length {} != {}", s.len(), N));
        }

        let mut obs = Observation(bits::BitArray::default());

        for (ix, c) in s.as_bytes().iter().enumerate() {
            let val = match c {
                b'0' => false,
                b'1' => true,
                _ => return Err(anyhow::anyhow!("Unexpected char '{c}'")),
            };
            obs.0.set(ix, val);
        }

        debug!("{s} -> {n} = {n:b}", n = u16::from(obs));

        // dbg!(s, u16::from(obs));

        Ok(obs)
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Main

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = "inputs/day03.txt")]
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    debug!("Using input {}", args.input.display());
    let file = File::open(args.input).unwrap();
    let buf = BufReader::new(file);

    let observations: Vec<Observation<12>> = parse::buffer(buf).unwrap();
    let diagnostics = DiagnosticReport::from_iter(observations.iter().copied());

    let (g, e) = diagnostics.power();
    let mul = (g as u32) * (e as u32);

    println!("Found power {g} * {e} = {mul}");

    let (ox, co) = diagnostics.life();
    let mul = (ox as u32) * (co as u32);
    println!("Found life {ox} * {co} = {mul}");
}

////////////////////////////////////////////////////////////////////////////////
/// Tests

#[cfg(test)]
mod tests {
    use adventofcode2021::parse;
    use test_log::test;

    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_observation() {
        let obs: Observation<1> = "1".parse().unwrap();
        let value: u16 = obs.into();
        assert_eq!(value, 0b1);

        let obs: Observation<2> = "11".parse().unwrap();
        assert_eq!(obs.bools().collect::<Vec<bool>>(), vec![true, true]);
        let value: u16 = obs.into();
        assert_eq!(value, 0b11);

        let obs: Observation<5> = "11001".parse().unwrap();
        let value: u16 = obs.into();
        assert_eq!(value, 0b11001);
        assert_eq!(obs, Observation::from(value));
        let expected = [true, true, false, false, true];
        assert_eq!(obs, Observation::from_iter(expected));

        let obs: Observation<5> = "11110".parse().unwrap();
        let value: u16 = obs.into();
        assert_eq!(value, 0b11110);

        let obs: Observation<16> = "1110100100010111".parse().unwrap();
        let value: u16 = obs.into();
        assert_eq!(value, 0b1110100100010111);
    }

    static EXAMPLE: &str = r###"
        00100
        11110
        10110
        10111
        10101
        01111
        00111
        11100
        10000
        11001
        00010
        01010
    "###;

    #[test]
    fn test_parse() {
        let observations: Vec<Observation<5>> = parse::buffer(EXAMPLE.as_bytes()).unwrap();

        let first = observations[0];
        let refs: Vec<bool> = first.bools().collect();
        assert_eq!(refs, vec![false, false, true, false, false]);

        let first = observations[1];
        let refs: Vec<bool> = first.bools().collect();
        assert_eq!(refs, vec![true, true, true, true, false]);
        let value: u16 = first.into();
        assert_eq!(value, 0b11110);
    }

    #[test]
    fn test_diagnostics() {
        let observations: Vec<Observation<5>> = parse::buffer(EXAMPLE.as_bytes()).unwrap();
        let diagnostics = DiagnosticReport::from_iter(observations.iter().copied());

        let (g, e) = diagnostics.power();
        assert_eq!((g, e), (22, 9));
    }

    #[test]
    fn test_life() {
        let observations: Vec<Observation<5>> = parse::buffer(EXAMPLE.as_bytes()).unwrap();
        let diagnostics = DiagnosticReport::from_iter(observations.iter().copied());

        let (ox, co) = diagnostics.life();
        assert_eq!((ox, co), (23, 10));
    }
}
