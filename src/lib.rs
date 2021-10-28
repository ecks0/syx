use measurements::{Frequency, Power};
use serde::{Deserialize, Deserializer, de::Error as _};
use zysfs::types as sysfs;
use std::{
    str::FromStr,
    time::Duration
};

pub mod cli;
pub mod format;
pub mod policy;

pub use format::Format;

pub use measurements;
#[cfg(feature = "nvml")]
pub use nvml_facade;
pub use zysfs;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Clap(#[from] clap::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    ParseValue(String),

    #[error("--{flag}: {message}")]
    ParseFlag {
        flag: String,
        message: String,
    },
}

impl Error {
    fn parse_flag(flag: &str, message: String) -> Self {
        Self::ParseFlag {
            flag: flag.to_string(),
            message,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

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

impl BoolStr {
    pub fn into_bool(self) -> bool { self.0 }
}

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

impl<'de> Deserialize<'de> for BoolStr {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Self::from_str(s).map_err(D::Error::custom)
    }
}

impl From<BoolStr> for bool {
    fn from(b: BoolStr) -> Self { b.0 }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct Indices(Vec<u64>);

impl Indices {
    pub fn into_vec(self) -> Vec<u64> { self.0 }
}

impl FromStr for Indices {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut ids = vec![];
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

impl<'de> Deserialize<'de> for Indices {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Self::from_str(s).map_err(D::Error::custom)
    }
}

impl From<Indices> for Vec<u64> {
    fn from(i: Indices) -> Self { i.0 }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct Toggles(Vec<(u64, bool)>);

impl Toggles {
    pub fn into_vec(self) -> Vec<(u64, bool)> { self.0 }
}

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

impl<'de> Deserialize<'de> for Toggles {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Self::from_str(s).map_err(D::Error::custom)
    }
}

impl From<Toggles> for Vec<(u64, bool)> {
    fn from(t: Toggles) -> Self { t.0 }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct FrequencyStr(Frequency);

impl FrequencyStr {
    pub fn into_frequency(self) -> Frequency { self.0 }
}

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

impl<'de> Deserialize<'de> for FrequencyStr {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Self::from_str(s).map_err(D::Error::custom)
    }
}

impl From<FrequencyStr> for Frequency {
    fn from(f: FrequencyStr) -> Self { f.0 }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct PowerStr(Power);

impl PowerStr {
    pub fn into_power(self) -> Power { self.0 }
}

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

impl<'de> Deserialize<'de> for PowerStr {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Self::from_str(s).map_err(D::Error::custom)
    }
}

impl From<PowerStr> for Power {
    fn from(p: PowerStr) -> Self { p.0 }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct DurationStr(Duration);

impl DurationStr {
    pub fn into_duration(self) -> Duration { self.0 }
}

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

impl<'de> Deserialize<'de> for DurationStr {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Self::from_str(s).map_err(D::Error::custom)
    }
}

impl From<DurationStr> for Duration {
    fn from(d: DurationStr) -> Self { d.0 }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum CardId {
    Id(u64),
    PciId(String),
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

impl<'de> Deserialize<'de> for CardId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Self::from_str(s).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct CardIds(Vec<CardId>);

impl CardIds {
    pub fn into_vec(self) -> Vec<CardId> { self.0 }
}

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
        for id in Indices::from_str(&indices.join(","))?.into_vec() {
            ids.push(CardId::Id(id));
        }
        for id in pci_ids {
            ids.push(CardId::PciId(id));
        }
        Ok(Self(ids))
    }
}

impl<'de> Deserialize<'de> for CardIds {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Self::from_str(s).map_err(D::Error::custom)
    }
}

impl From<CardIds> for Vec<CardId> {
    fn from(c: CardIds) -> Self { c.0 }
}

fn de_indices<'de, D>(deserializer: D) -> std::result::Result<Option<Vec<u64>>, D::Error>
where
    D: Deserializer<'de>,
{
    let i: Indices = Deserialize::deserialize(deserializer)?;
    Ok(Some(i.into_vec()))
}

fn de_bool<'de, D>(deserializer: D) -> std::result::Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let b: BoolStr = Deserialize::deserialize(deserializer)?;
    Ok(Some(b.into_bool()))
}

fn de_toggles<'de, D>(deserializer: D) -> std::result::Result<Option<Vec<(u64, bool)>>, D::Error>
where
    D: Deserializer<'de>,
{
    let t: Toggles = Deserialize::deserialize(deserializer)?;
    Ok(Some(t.into_vec()))
}

fn de_frequency<'de, D>(deserializer: D) -> std::result::Result<Option<Frequency>, D::Error>
where
    D: Deserializer<'de>,
{
    let f: FrequencyStr = Deserialize::deserialize(deserializer)?;
    Ok(Some(f.into_frequency()))
}

fn de_card_ids<'de, D>(deserializer: D) -> std::result::Result<Option<Vec<CardId>>, D::Error>
where
    D: Deserializer<'de>,
{
    let c: CardIds = Deserialize::deserialize(deserializer)?;
    Ok(Some(c.into_vec()))
}

fn de_power<'de, D>(deserializer: D) -> std::result::Result<Option<Power>, D::Error>
where
    D: Deserializer<'de>,
{
    let p: PowerStr = Deserialize::deserialize(deserializer)?;
    Ok(Some(p.into_power()))
}

fn de_duration<'de, D>(deserializer: D) -> std::result::Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    let d: DurationStr = Deserialize::deserialize(deserializer)?;
    Ok(Some(d.into_duration()))
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, PartialOrd)]
pub struct Knobs {

    #[serde(deserialize_with = "de_indices")]
    pub cpu: Option<Vec<u64>>,

    #[serde(deserialize_with = "de_bool")]
    pub cpu_on: Option<bool>,

    #[serde(deserialize_with = "de_toggles")]
    pub cpus_on: Option<Vec<(u64, bool)>>,

    pub cpufreq_gov: Option<String>,

    #[serde(deserialize_with = "de_frequency")]
    pub cpufreq_min: Option<Frequency>,

    #[serde(deserialize_with = "de_frequency")]
    pub cpufreq_max: Option<Frequency>,

    #[serde(deserialize_with = "de_card_ids")]
    pub drm_i915: Option<Vec<CardId>>,

    #[serde(deserialize_with = "de_frequency")]
    pub drm_i915_min: Option<Frequency>,

    #[serde(deserialize_with = "de_frequency")]
    pub drm_i915_max: Option<Frequency>,

    #[serde(deserialize_with = "de_frequency")]
    pub drm_i915_boost: Option<Frequency>,

    #[cfg(feature = "nvml")]
    #[serde(deserialize_with = "de_card_ids")]
    pub nvml: Option<Vec<CardId>>,

    #[cfg(feature = "nvml")]
    #[serde(deserialize_with = "de_frequency")]
    pub nvml_gpu_min: Option<Frequency>,

    #[cfg(feature = "nvml")]
    #[serde(deserialize_with = "de_frequency")]
    pub nvml_gpu_max: Option<Frequency>,

    #[cfg(feature = "nvml")]
    #[serde(deserialize_with = "de_bool")]
    pub nvml_gpu_reset: Option<bool>,

    #[cfg(feature = "nvml")]
    #[serde(deserialize_with = "de_power")]
    pub nvml_power_limit: Option<Power>,

    pub pstate_epb: Option<u64>,

    pub pstate_epp: Option<String>,

    pub rapl_package: Option<u64>,

    pub rapl_zone: Option<u64>,

    #[serde(deserialize_with = "de_power")]
    pub rapl_c0_limit: Option<Power>,

    #[serde(deserialize_with = "de_power")]
    pub rapl_c1_limit: Option<Power>,

    #[serde(deserialize_with = "de_duration")]
    pub rapl_c0_window: Option<Duration>,

    #[serde(deserialize_with = "de_duration")]
    pub rapl_c1_window: Option<Duration>,
}

impl Knobs {
    pub fn has_cpu_values(&self) -> bool {
        self.cpu_on.is_some() ||
        self.cpus_on.is_some()
    }

    pub fn has_cpu_or_related_values(&self) -> bool {
        self.has_cpu_values() ||
        self.has_cpufreq_values() ||
        self.has_pstate_values()
    }

    pub fn has_cpufreq_values(&self) -> bool {
        self.cpufreq_gov.is_some() ||
        self.cpufreq_min.is_some() ||
        self.cpufreq_max.is_some()
    }

    pub fn has_drm_i915_values(&self) -> bool {
        self.drm_i915_min.is_some() ||
        self.drm_i915_max.is_some() ||
        self.drm_i915_boost.is_some()
    }

    pub fn has_drm_values(&self) -> bool {
        self.has_drm_i915_values()
    }

    #[cfg(feature = "nvml")]
    pub fn has_nvml_values(&self) -> bool {
        self.nvml_gpu_min.is_some() ||
        self.nvml_gpu_max.is_some() ||
        self.nvml_gpu_reset.is_some() ||
        self.nvml_power_limit.is_some()
    }

    pub fn has_pstate_values(&self) -> bool {
        self.pstate_epb.is_some() ||
        self.pstate_epp.is_some()
    }

    pub fn has_rapl_values(&self) -> bool {
        self.rapl_c0_limit.is_some() ||
        self.rapl_c1_limit.is_some() ||
        self.rapl_c0_window.is_some() ||
        self.rapl_c1_window.is_some()
    }
}

impl Knobs {

    async fn apply(&self) {
        use sysfs::tokio::Write as _;

        let cpufreq: Option<sysfs::cpufreq::Cpufreq> = self.into();
        let intel_pstate: Option<sysfs::intel_pstate::IntelPstate> = self.into();
        let cpu: Option<sysfs::cpu::Cpu> = self.into();
        let intel_rapl: Option<sysfs::intel_rapl::IntelRapl> = self.into();
        let drm: Option<sysfs::drm::Drm> = self.into();
        #[cfg(feature = "nvml")]
        let nvml: Option<policy::NvmlPolicies> = self.into();

        if cpufreq.is_some() || intel_pstate.is_some() {
            let onlined = policy::set_all_cpus_online().await;
            if let Some(cpufreq) = cpufreq { cpufreq.write().await; }
            if let Some(intel_pstate) = intel_pstate { intel_pstate.write().await; }
            policy::set_cpus_offline(onlined).await;
        }
        if let Some(cpu) = cpu { cpu.write().await; }
        if let Some(intel_rapl) = intel_rapl { intel_rapl.write().await; }
        if let Some(drm) = drm { drm.write().await; }
        #[cfg(feature = "nvml")]
        if let Some(nvml) = nvml { nvml.write(); }
    }
}
