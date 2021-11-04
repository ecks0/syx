use measurements::{Frequency, Power};
use std::{str::FromStr, time::Duration};
use crate::{Error, Result};
use crate::CardId;

fn start_of_unit(s: &str) -> Option<usize> {
    for (i, c) in s.chars().enumerate() {
        match c {
            '0'..='9' | '.' => continue,
            _ => return Some(i),
        }
    }
    None
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct BoolStr(bool);

impl FromStr for BoolStr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "0" | "false" => Ok(Self(false)),
            "1" | "true" => Ok(Self(true)),
            _ => Err(Error::ParseValue("Expected 0, 1, false, or true".into())),
        }
    }
}

impl From<BoolStr> for bool {
    fn from(b: BoolStr) -> Self { b.0 }
}

impl FromStr for CardId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.contains(':') {
            Ok(Self::PciId(s.into()))
        } else {
            let id = s.parse::<u64>()
                .map_err(|_| Error::ParseValue("Expected id integer or pci id string".into()))?;
            Ok(Self::Id(id))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct CardIds(Vec<CardId>);

impl FromStr for CardIds {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut indices = vec![];
        let mut pci_ids = vec![];
        for ss in s.split(',') {
            if ss.contains(':') {
                pci_ids.push(ss.to_string());
            } else {
                indices.push(ss.to_string());
            }
        }
        let mut ids = vec![];
        for id in Vec::from(Indices::from_str(&indices.join(","))?) {
            ids.push(CardId::Id(id));
        }
        for id in pci_ids {
            ids.push(CardId::PciId(id));
        }
        Ok(Self(ids))
    }
}

impl From<CardIds> for Vec<CardId> {
    fn from(c: CardIds) -> Self { c.0 }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct DurationStr(Duration);

impl FromStr for DurationStr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if let Some(pos) = start_of_unit(s) {
            match s[..pos].parse::<u64>() {
                Ok(v) => match &s[pos..] {
                    "n" | "ns" => Ok(Self(Duration::from_nanos(v))),
                    "u" | "us" => Ok(Self(Duration::from_micros(v))),
                    "m" | "ms" => Ok(Self(Duration::from_millis(v))),
                    "s" => Ok(Self(Duration::from_secs(v))),
                    _ => Err(Error::ParseValue("Unrecognized duration unit".into())),
                },
                Err(_) => Err(Error::ParseValue("Expected duration value, ex. 2000, 2000ms, 2s".into())),
            }
        } else {
            match s.parse::<u64>() {
                Ok(v) => Ok(Self(Duration::from_millis(v))),
                Err(_) => Err(Error::ParseValue("Expected duration value, ex. 3000, 3000ms, 3s".into())),
            }
        }
    }
}

impl From<DurationStr> for Duration {
    fn from(d: DurationStr) -> Self { d.0 }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct FrequencyStr(Frequency);

impl FromStr for FrequencyStr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let f = match start_of_unit(s) {
            Some(pos) => match s[..pos].parse::<f64>() {
                Ok(v) => match s[pos..].to_lowercase().as_str() {
                    "h" | "hz" => Frequency::from_hertz(v),
                    "k" | "khz" => Frequency::from_kilohertz(v),
                    "m" | "mhz" => Frequency::from_megahertz(v),
                    "g" | "ghz" => Frequency::from_gigahertz(v),
                    "t" | "thz" => Frequency::from_terahertz(v),
                    _ => return Err(Error::ParseValue("Unrecognized frequency unit".into())),
                },
                Err(_) => return Err(Error::ParseValue("Expected frequency value with optional unit".into())),
            },
            None => match s.parse::<f64>() {
                Ok(v) => Frequency::from_megahertz(v),
                Err(_) => return Err(Error::ParseValue("Expected frequency value with optional unit".into())),
            }
        };
        Ok(Self(f))
    }
}

impl From<FrequencyStr> for Frequency {
    fn from(f: FrequencyStr) -> Self { f.0 }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct Indices(Vec<u64>);

impl FromStr for Indices {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut ids = vec![];
        let s = s.trim_end_matches(',');
        for item in s.split(',') {
            let s: Vec<&str> = item.split('-').collect();
            match &s[..] {
                [id] => ids.push(id.parse::<u64>()
                    .map_err(|_| Error::ParseValue("Index is not an integer".into()))?),
                [start, end] =>
                    std::ops::Range {
                        start: start.parse::<u64>()
                            .map_err(|_| Error::ParseValue("Start of range is not an integer".into()))?,
                        end: 1 + end.parse::<u64>()
                            .map_err(|_| Error::ParseValue("End of range is not an integer".into()))?,
                    }
                    .for_each(|i| ids.push(i)),
                _ => return Err(Error::ParseValue("Expected comma-delimited list of integers and/or integer ranges".into())),
            }
        }
        ids.sort_unstable();
        ids.dedup();
        Ok(Self(ids))
    }
}

impl From<Indices> for Vec<u64> {
    fn from(i: Indices) -> Self { i.0 }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct PowerStr(Power);

impl FromStr for PowerStr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if let Some(pos) = start_of_unit(s) {
            match s[..pos].parse::<f64>() {
                Ok(v) => match &s[pos..] {
                    "u" | "uw" => Ok(Self(Power::from_microwatts(v))),
                    "m" | "mw" => Ok(Self(Power::from_milliwatts(v))),
                    "w" => Ok(Self(Power::from_watts(v))),
                    "k" | "kw" => Ok(Self(Power::from_kilowatts(v))),
                    _ => Err(Error::ParseValue("Unrecognized power unit".into())),
                },
                Err(_) => Err(Error::ParseValue("Expected power value".into())),
            }
        } else {
            match s.parse::<f64>() {
                Ok(v) => Ok(Self(Power::from_watts(v))),
                Err(_) => Err(Error::ParseValue("Expected power value".into())),
            }
        }
    }
}

impl From<PowerStr> for Power {
    fn from(p: PowerStr) -> Self { p.0 }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct Toggles(Vec<(u64, bool)>);

impl FromStr for Toggles {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut toggles = vec![];
        for (i, c) in s.chars().enumerate() {
            toggles.push(
                (
                    i as u64,
                    match c {
                        '_' | '-' => continue,
                        '0' => false,
                        '1' => true,
                        _ => return Err(Error::ParseValue("Expected sequence of 0, 1, or -".into())),
                    },
                )
            );
        }
        Ok(Self(toggles))
    }
}

impl From<Toggles> for Vec<(u64, bool)> {
    fn from(t: Toggles) -> Self { t.0 }
}
