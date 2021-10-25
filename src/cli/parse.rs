use measurements::{Frequency, Power};
use std::time::Duration;
use crate::cli::{Error, CardId, Result};

fn parse_bool(flag: &'static str, s: &str) -> Result<bool> {
    match s {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(Error::parse(flag, "expected 0 or 1")),
    }
}

fn start_of_unit(s: &str) -> Option<usize> {
    for (i, c) in s.chars().enumerate() {
        match c {
            '0'..='9' | '.' => continue,
            _ => return Some(i),
        }
    }
    None
}

fn parse_frequency(flag: &'static str, s: &str) -> Result<Frequency> {
    if let Some(pos) = start_of_unit(s) {
        match s[..pos].parse::<f64>() {
            Ok(v) => match &s[pos..] {
                "h" | "hz" => Ok(Frequency::from_hertz(v)),
                "k" | "khz" => Ok(Frequency::from_kilohertz(v)),
                "m" | "mhz" => Ok(Frequency::from_megahertz(v)),
                "g" | "ghz" => Ok(Frequency::from_gigahertz(v)),
                "t" | "thz" => Ok(Frequency::from_terahertz(v)),
                _ => Err(Error::parse(flag, "unrecognized frequency unit")),
            },
            Err(_) => Err(Error::parse(flag, "expected hertz value, ex. 1200, 1200mhz, 1.2ghz")),
        }
    } else {
        match s.parse::<f64>() {
            Ok(v) => Ok(Frequency::from_megahertz(v)),
            Err(_) => Err(Error::parse(flag, "expected hertz value, ex. 1300, 1300mhz, 1.3ghz")),
        }
    }
}

#[cfg(feature = "nvml")]
fn parse_frequency_min_max(flag: &'static str, s: &str) -> Result<(Frequency, Frequency)> {
    let s: Vec<&str> = s.split(',').collect();
    match &s[..] {
        [freq] => {
            let freq = parse_frequency(flag, freq)?;
            Ok((freq, freq))
        }
        [min, max] => Ok((
            parse_frequency(flag, min)?,
            parse_frequency(flag, max)?,
        )),
        _ => Err(Error::parse(flag, "Expected frequency min/max, ex. 800,1.2ghz)")),
    }
}

fn parse_power(flag: &'static str, s: &str) -> Result<Power> {
    if let Some(pos) = start_of_unit(s) {
        match s[..pos].parse::<f64>() {
            Ok(v) => match &s[pos..] {
                "u" | "uw" => Ok(Power::from_microwatts(v)),
                "m" | "mw" => Ok(Power::from_milliwatts(v)),
                "w" => Ok(Power::from_watts(v)),
                "k" | "kw" => Ok(Power::from_kilowatts(v)),
                _ => Err(Error::parse(flag, "unrecognized power unit")),
            },
            Err(_) => Err(Error::parse(flag, "expected power value")),
        }
    } else {
        match s.parse::<f64>() {
            Ok(v) => Ok(Power::from_watts(v)),
            Err(_) => Err(Error::parse(flag, "expected power value")),
        }
    }
}

fn parse_duration(flag: &'static str, s: &str) -> Result<Duration> {
    if let Some(pos) = start_of_unit(s) {
        match s[..pos].parse::<u64>() {
            Ok(v) => match &s[pos..] {
                "n" | "ns" => Ok(Duration::from_nanos(v)),
                "u" | "us" => Ok(Duration::from_micros(v)),
                "m" | "ms" => Ok(Duration::from_millis(v)),
                "s" => Ok(Duration::from_secs(v)),
                _ => Err(Error::parse(flag, "unrecognized duration unit")),
            },
            Err(_) => Err(Error::parse(flag, "expected duration value, ex. 2000, 2000ms, 2s")),
        }
    } else {
        match s.parse::<u64>() {
            Ok(v) => Ok(Duration::from_millis(v)),
            Err(_) => Err(Error::parse(flag, "expected duration value, ex. 3000, 3000ms, 3s")),
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

fn parse_card_ids(flag: &'static str, s: &str) -> Result<Vec<CardId>> {
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
    for id in parse_indices(flag, &indices.join(","))? {
        ids.push(CardId::Index(id));
    }
    for id in pci_ids {
        ids.push(CardId::PciId(id));
    }
    Ok(ids)
}

fn parse_toggles(flag: &'static str, s: &str) -> Result<Vec<(u64, bool)>> {
    let mut toggles = vec![];
    for (i, c) in s.chars().enumerate() {
        toggles.push(
            (
                i as u64,
                match c {
                    '-' | '_' => continue,
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
        .map_err(|_| Error::parse(flag, "expected 64-bit unsigned integer value"))
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

pub fn rapl_package(s: &str) -> Result<u64> {
    parse_u64("-p/--rapl-package", s)
}

pub fn rapl_zone(s: &str) -> Result<u64> {
    parse_u64("-z/--rapl-zone", s)
}

pub fn rapl_constraint(s: &str) -> Result<u64> {
    parse_u64("-C/--rapl-constraint", s)
}

pub fn rapl_limit(s: &str) -> Result<Power> {
    parse_power("-p/--rapl-package", s)
}

pub fn rapl_window(s: &str) -> Result<Duration> {
    parse_duration("-w/--rapl-window", s)
}

pub fn drm_i915(s: &str) -> Result<Vec<CardId>> {
    parse_card_ids("--drm-i915", s)
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

#[cfg(feature = "nvml")]
pub fn nvml(s: &str) -> Result<Vec<CardId>> {
    parse_card_ids("--nvml", s)
}

#[cfg(feature = "nvml")]
pub fn nvml_gpu_freq(s: &str) -> Result<(Frequency, Frequency)> {
    parse_frequency_min_max("--nvml-gpu-freq", s)
}

#[cfg(feature = "nvml")]
pub fn nvml_power_limit(s: &str) -> Result<Power> {
    parse_power("--nvml-power-limit", s)
}
