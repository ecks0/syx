use measurements::Frequency;
use crate::{Error, Result};

fn parse_bool(flag: &'static str, s: &str) -> Result<bool> {
    match s {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(Error::parse(flag, "expected 0 or 1")),
    }
}

fn parse_frequency(flag: &'static str, s: &str) -> Result<Frequency> {
    let mut pos = None;
    for (i, c) in s.chars().enumerate() {
        match c {
            '0'..='9' | '.' => continue,
            _ => {
                pos = Some(i);
                break;
            },
        }
    }
    if let Some(i) = pos {
        match s[..i].parse::<u64>() {
            Ok(v) => match &s[i..] {
                "hz" => Ok(Frequency::from_hertz(v as f64)),
                "khz" => Ok(Frequency::from_kilohertz(v as f64)),
                "mhz" => Ok(Frequency::from_megahertz(v as f64)),
                "ghz" => Ok(Frequency::from_gigahertz(v as f64)),
                "thz" => Ok(Frequency::from_terahertz(v as f64)),
                _ => Err(Error::parse(flag, "unrecognized hertz magnitude")),
            },
            Err(_) => Err(Error::parse(flag, "expected hertz value, ex. 1200mhz, 1.2ghz")),
        }
    } else {
        match s.parse::<u64>() {
            Ok(v) => Ok(Frequency::from_hertz(v as f64)),
            Err(_) => Err(Error::parse(flag, "expected hertz value, ex. 1200mhz, 1.2ghz")),
        }
    }
}

fn parse_indices(flag: &'static str, s: &str) -> Result<Option<Vec<u64>>> {
    let mut ids = vec![];
    for item in s.split(',') {
        let s: Vec<&str> = item.split('-').collect();
        match &s[..] {
            [id] => ids.push(id.parse::<u64>().map_err(|_| Error::parse(flag, "index is not an integer"))?),
            [start, end] =>
                std::ops::Range {
                    start: start.parse::<u64>().map_err(|_| Error::parse(flag, "start of range is not an integer"))?,
                    end: 1 + end.parse::<u64>().map_err(|_| Error::parse(flag, "end of range is not an integer"))?,
                }.for_each(|i| ids.push(i)),
            _ => return Err(Error::parse(flag, "expected sequence of indices, ex. 0,1,3-5,10")),
        }
    }
    Ok(if ids.is_empty() { None } else { Some(ids) })
}

fn parse_toggles(flag: &'static str, s: &str) -> Result<Option<Vec<(u64, bool)>>> {
    let mut toggles = vec![];
    for (i, c) in s.chars().enumerate() {
        toggles.push(
            (
                i as u64,
                match c {
                    '-' => continue,
                    '0' => false,
                    '1' => true,
                    _ => return Err(Error::parse(flag, "expected sequence of 0, 1, or -")),
                },
            )
        );
    }
    Ok(if toggles.is_empty() { None } else { Some(toggles) })
}

fn parse_u64(flag: &'static str, s: &str) -> Result<u64> {
    match s.parse::<u64>() {
        Ok(v) => Ok(v),
        Err(_) => Err(Error::parse(flag, "expected integer value")),
    }
}

pub fn cpus(s: Option<&str>) -> Result<Option<Vec<u64>>> {
    Ok(match s {
        Some(s) => parse_indices("-c/--cpus", s)?,
        None => None,
    })
}

pub fn cpu_on(s: Option<&str>) -> Result<Option<bool>> {
    Ok(match s {
        Some(s) => Some(parse_bool("-o/--cpu-on", s)?),
        None => None,
    })
}

pub fn cpu_on_each(s: Option<&str>) -> Result<Option<Vec<(u64, bool)>>> {
    Ok(match s {
        Some(s) => parse_toggles("-O/--cpu-on-each", s)?,
        None => None,
    })
}

pub fn cpufreq_gov(s: Option<&str>) -> Option<String> {
    s.map(|s| s.to_string())
}

pub fn cpufreq_min(s: Option<&str>) -> Result<Option<Frequency>> {
    Ok(match s {
        Some(s) => Some(parse_frequency("-n/--cpufreq-min", s)?),
        None => None,
    })
}

pub fn cpufreq_max(s: Option<&str>) -> Result<Option<Frequency>> {
    Ok(match s {
        Some(s) => Some(parse_frequency("-x/--cpufreq-max", s)?),
        None => None,
    })
}

pub fn pstate_epb(s: Option<&str>) -> Result<Option<u64>> {
    Ok(match s {
        None => None,
        Some(s) => {
            let epb = parse_u64("--pstate-epb", s)?;
            if epb > 15 {
                return Err(Error::parse("--pstate-epb", "expected integer between 0 and 15, inclusive"));
            } else {
                Some(epb)
            }
        },
    })
}

pub fn pstate_epp(s: Option<&str>) -> Option<String> {
    s.map(|s| s.to_string())
}

pub fn drm_i915(s: Option<&str>) -> Result<Option<Vec<u64>>> {
    Ok(match s {
        Some(s) => parse_indices("--drm-i915", s)?,
        None => None,
    })
}

pub fn drm_i915_min(s: Option<&str>) -> Result<Option<Frequency>> {
    Ok(match s {
        Some(s) => Some(parse_frequency("--i915-freq-min", s)?),
        None => None,
    })
}

pub fn drm_i915_max(s: Option<&str>) -> Result<Option<Frequency>> {
    Ok(match s {
        Some(s) => Some(parse_frequency("--i915-freq-max", s)?),
        None => None,
    })
}

pub fn drm_i915_boost(s: Option<&str>) -> Result<Option<Frequency>> {
    Ok(match s {
        Some(s) => Some(parse_frequency("--i915-freq-boost", s)?),
        None => None,
    })
}
