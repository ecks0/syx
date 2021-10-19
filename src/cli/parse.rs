use measurements::{Frequency, Power};
use crate::cli::{Error, Result};

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
    if let Some(pos) = pos {
        match s[..pos].parse::<f64>() {
            Ok(v) => match &s[pos..] {
                "h" | "hz" => Ok(Frequency::from_hertz(v)),
                "k" | "khz" => Ok(Frequency::from_kilohertz(v)),
                "m" | "mhz" => Ok(Frequency::from_megahertz(v)),
                "g" | "ghz" => Ok(Frequency::from_gigahertz(v)),
                "t" | "thz" => Ok(Frequency::from_terahertz(v)),
                _ => Err(Error::parse(flag, "unrecognized hertz magnitude")),
            },
            Err(_) => Err(Error::parse(flag, "expected hertz value, ex. 1200mhz, 1.2ghz")),
        }
    } else {
        match s.parse::<u64>() {
            Ok(v) => Ok(Frequency::from_megahertz(v as f64)),
            Err(_) => Err(Error::parse(flag, "expected hertz value, ex. 1300mhz, 1.3ghz")),
        }
    }
}

fn parse_power(flag: &'static str, s: &str) -> Result<Power> {
    let mut pos = None;
    for (i, c) in s.chars().enumerate() {
        match c {
            '0'..='9' | '.' => continue,
            _ => {
                pos = Some(i);
                break;
            }
        }
    }
    if let Some(pos) = pos {
        match s[..pos].parse::<f64>() {
            Ok(v) => match &s[pos..] {
                "m" | "mw" => Ok(Power::from_milliwatts(v)),
                "w" => Ok(Power::from_watts(v)),
                "k" | "kw" => Ok(Power::from_kilowatts(v)),
                _ => Err(Error::parse(flag, "unrecognized watts magnitude")),
            },
            Err(_) => Err(Error::parse(flag, "expected watts value, ex. 260w, 260000mw")),
        }
    } else {
        match s.parse::<u32>() {
            Ok(v) => Ok(Power::from_watts(v as f64)),
            Err(_) => Err(Error::parse(flag, "expected watts value, ex. 270w, 270000mw")),
        }
    }
}

fn parse_indices(flag: &'static str, s: &str) -> Result<Vec<u64>> {
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
    ids.sort_unstable();
    ids.dedup();
    Ok(ids)
}

fn parse_toggles(flag: &'static str, s: &str) -> Result<Vec<(u64, bool)>> {
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
    Ok(toggles)
}

fn parse_u64(flag: &'static str, s: &str) -> Result<u64> {
    s.parse::<u64>()
        .map_err(|_| Error::parse(flag, "expected 64-bit integer value"))
}

pub fn cpu(s: &str) -> Result<Vec<u64>> {
    parse_indices("-c/--cpu", s)
}

pub fn cpu_on(s: &str) -> Result<bool> {
    parse_bool("-o/--cpu-on", s)
}

pub fn cpu_on_each(s: &str) -> Result<Vec<(u64, bool)>> {
    parse_toggles("-O/--cpu-on-each", s)
}

pub fn cpufreq_gov(s: &str) -> Result<String> {
    Ok(s.to_string())
}

pub fn cpufreq_min(s: &str) -> Result<Frequency> {
    parse_frequency("-n/--cpufreq-min", s)
}

pub fn cpufreq_max(s: &str) -> Result<Frequency> {
    parse_frequency("-x/--cpufreq-max", s)
}

pub fn pstate_epb(s: &str) -> Result<u64> {
    let epb = parse_u64("--pstate-epb", s)?;
    if epb > 15 {
        Err(Error::parse("--pstate-epb", "expected integer between 0 and 15, inclusive"))
    } else {
        Ok(epb)
    }
}

pub fn pstate_epp(s: &str) -> Result<String> {
    Ok(s.to_string())
}

pub fn drm_i915(s: &str) -> Result<Vec<u64>> {
    parse_indices("--drm-i915", s)
}

pub fn drm_i915_min(s: &str) -> Result<Frequency> {
    parse_frequency("--i915-freq-min", s)
}

pub fn drm_i915_max(s: &str) -> Result<Frequency> {
    parse_frequency("--i915-freq-max", s)
}

pub fn drm_i915_boost(s: &str) -> Result<Frequency> {
    parse_frequency("--i915-freq-boost", s)
}

pub fn nvml(s: &str) -> Result<Vec<u32>> {
    Ok(parse_indices("--nvml", s)?.into_iter().map(|i| i as u32).collect())
}

pub fn nvml_gpu_clock(s: &str) -> Result<(Frequency, Frequency)> {
    let s: Vec<&str> = s.split(',').collect();
    match &s[..] {
        [freq] => {
            let freq = parse_frequency("--nvml-gpu-clock", freq)?;
            Ok((freq, freq))
        }
        [min, max] => Ok((
            parse_frequency("--nvml-gpu-clock", min)?,
            parse_frequency("--nvml-gpu-clock", max)?,
        )),
        _ => Err(Error::parse("--nvml-gpu-clock", "Example: 1.4ghz (constant) or 1.2ghz,1.6ghz (min,max)")),
    }
}

pub fn nvml_power_limit(s: &str) -> Result<Power> {
    parse_power("--nvml-power-limit", s)
}
