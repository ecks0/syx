use async_trait::async_trait;
use comfy_table as ct;
use measurements::{Energy, Frequency, Power};
#[cfg(feature = "nvml")] use nvml_facade as nvml;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use zysfs::types as sysfs;
use std::fmt::Display;
use crate::{Error, Result};

pub const DOT: &str = "\u{2022}";

pub fn dot() -> String { DOT.to_string() }

#[derive(Debug)]
pub struct Table(ct::Table);

impl Table {
    pub fn new(header: &[&str]) -> Self {
        let mut tab = ct::Table::new();
        tab.load_preset(ct::presets::NOTHING);
        tab.set_header(header);
        tab.add_row(
            header
                .iter()
                .map(|h| "-".repeat(h.len()))
                .collect::<Vec<String>>()
        );
        Self(tab)
    }

    pub fn row<S: Display>(&mut self, row: &[S]) {
        self.0.add_row(row);
    }
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.0)
    }
}

fn format_uw(uw: u64) -> String {
    match uw {
        0 => "0 W".to_string(),
        _ => {
            let scale = 10u64.pow(
                match uw {
                    v if v > 10u64.pow(18) => 15,
                    v if v > 10u64.pow(15) => 12,
                    v if v > 10u64.pow(12) => 9,
                    v if v > 10u64.pow(9) => 6,
                    v if v > 10u64.pow(6) => 3,
                    _ => 0,
                }
            );
            let uw = (uw/scale) * scale;
            Power::from_microwatts(uw as f64).to_string()
        }
    }
}

fn format_uj(uj: u64) -> String {
    match uj {
        0 => "0 J".to_string(),
        _ => {
            let j = uj as f64 * 10f64.powf(-6.);
            format!("{:.3}", Energy::from_joules(j))
        },
    }
}

fn format_hz(hz: u64) -> String {
    match hz {
        0 => "0 Hz".to_string(),
        _ => {
            let f = Frequency::from_hertz(hz as f64);
            if hz < 10u64.pow(9) {
                format!("{:.0}", f)
            } else {
                format!("{:.1}", f)
            }
        },
    }
}

#[cfg(feature = "nvml")]
fn format_bytes(b: u64) -> String {
    if b < 1000 { format!("{} B", b) }
    else if b < 1000u64.pow(2) { format!("{:.1} kB", b as f64/1000f64) }
    else if b < 1000u64.pow(3) { format!("{:.1} MB", b as f64/(1000u64.pow(2) as f64)) }
    else if b < 1000u64.pow(4) { format!("{:.1} GB", b as f64/(1000u64.pow(3) as f64)) }
    else if b < 1000u64.pow(5) { format!("{:.1} TB", b as f64/(1000u64.pow(4) as f64)) }
    else if b < 1000u64.pow(6) { format!("{:.1} PB", b as f64/(1000u64.pow(5) as f64)) }
    else { format!("{:.1} TB", b as f64/(1000u64.pow(4) as f64)) }
}

fn nl(mut s: String) -> String { s.push('\n'); s }

#[async_trait]
pub trait Format {
    type Err;

    async fn format_values<W: AsyncWrite + Send + Unpin>(&self, w: &mut W) -> std::result::Result<(), Self::Err>;
}

#[async_trait]
impl Format for (sysfs::cpu::Cpu, sysfs::cpufreq::Cpufreq) {
    type Err = Error;

    async fn format_values<W: AsyncWrite + Send + Unpin>(&self, w: &mut W) -> Result<()> {

        fn khz(khz: u64) -> String { format_hz(khz * 10u64.pow(3)) }

        fn cpu_cpufreq(cpu: &sysfs::cpu::Cpu, cpufreq: &sysfs::cpufreq::Cpufreq ) -> Option<String> {
            let cpu_pols = cpu.policies.as_ref()?;
            let cpufreq_pol_default = sysfs::cpufreq::Policy::default();
            let mut tab = Table::new(&["CPU", "Online", "Governor", "Cur", "Min", "Max", "CPU min", "CPU max"]);
            for cpu_pol in cpu_pols {
                let id = if let Some(id) = cpu_pol.id { id } else { continue; };
                let cpufreq_pol = cpufreq.policies
                    .as_ref()
                    .and_then(|p| p
                        .iter()
                        .find(|p| Some(id) == p.id))
                    .unwrap_or(&cpufreq_pol_default);
                tab.row(&[
                    id.to_string(),
                    cpu_pol.cpu_online.map(|v| v.to_string()).unwrap_or_else(dot),
                    cpufreq_pol.scaling_governor.clone().unwrap_or_else(dot),
                    cpufreq_pol.scaling_cur_freq.map(khz).unwrap_or_else(dot),
                    cpufreq_pol.scaling_min_freq.map(khz).unwrap_or_else(dot),
                    cpufreq_pol.scaling_max_freq.map(khz).unwrap_or_else(dot),
                    cpufreq_pol.cpuinfo_min_freq.map(khz).unwrap_or_else(dot),
                    cpufreq_pol.cpuinfo_max_freq.map(khz).unwrap_or_else(dot),
                ]);
            }
            Some(nl(tab.to_string()))
        }

        fn governors(cpufreq: &sysfs::cpufreq::Cpufreq) -> Option<String> {
            let policies = cpufreq.policies.as_deref()?;
            let mut govs: Vec<String> = policies
                .iter()
                .filter_map(|p| p.scaling_available_governors.as_deref().map(|g| g.join(" ")))
                .collect();
            if govs.is_empty() { return None; }
            govs.sort_unstable();
            govs.dedup();
            let mut tab = Table::new(&["CPU", "Available governors"]);
            if govs.len() == 1 {
                tab.row(&["all", &govs[0]]);
            } else {
                for p in policies {
                    tab.row(&[
                        p.id.map(|v| v.to_string()).unwrap_or_else(dot),
                        p.scaling_available_governors.as_ref().map(|v| v.join(" ")).unwrap_or_else(dot),
                    ])
                }
            }
            Some(nl(tab.to_string()))
        }

        let (cpu, cpufreq) = self;
        if let Some(s) = cpu_cpufreq(cpu, cpufreq) { w.write_all(s.as_bytes()).await?; }
        if let Some(s) = governors(cpufreq) { w.write_all(s.as_bytes()).await?; }
        Ok(())
    }
}

#[async_trait]
impl Format for sysfs::drm::Drm {
    type Err = Error;

    async fn format_values<W: AsyncWrite + Send + Unpin>(&self, w: &mut W) -> Result<()> {

        fn mhz(mhz: u64) -> String { format_hz(mhz * 10u64.pow(6)) }

        #[allow(clippy::ptr_arg)]
        fn i915(cards: &Vec<sysfs::drm::Card>) -> Option<String> {
            let cards: Vec<&sysfs::drm::Card> = cards
                .iter()
                .filter(|c| c.driver.as_ref().map(|d| "i915" == d).unwrap_or(false))
                .collect();
            if cards.is_empty() { return None; }
            let mut tab = Table::new(&["Card", "Driver", "Actual", "Req'd", "Min", "Max", "Boost", "GPU min", "GPU max"]);
            for card in cards {
                if let Some(sysfs::drm::DriverPolicy::I915(policy)) = card.driver_policy.as_ref() {
                    tab.row(&[
                        card.id.map(|v| v.to_string()).unwrap_or_else(dot),
                        card.driver.clone().unwrap_or_else(dot),
                        policy.act_freq_mhz.map(mhz).unwrap_or_else(dot),
                        policy.cur_freq_mhz.map(mhz).unwrap_or_else(dot),
                        policy.min_freq_mhz.map(mhz).unwrap_or_else(dot),
                        policy.max_freq_mhz.map(mhz).unwrap_or_else(dot),
                        policy.boost_freq_mhz.map(mhz).unwrap_or_else(dot),
                        policy.rpn_freq_mhz.map(mhz).unwrap_or_else(dot),
                        policy.rp0_freq_mhz.map(mhz).unwrap_or_else(dot),
                    ]);
                }
            }
            Some(nl(tab.to_string()))
        }

        if let Some(s) = self.cards.as_ref().and_then(i915) {
            w.write_all(s.as_bytes()).await?;
        }
        Ok(())
    }
}

#[cfg(feature = "nvml")]
#[async_trait]
impl Format for nvml_facade::Nvml {
    type Err = Error;

    async fn format_values<W: AsyncWrite + Send + Unpin>(&self, w: &mut W) -> Result<()> {

        fn mhz(mhz: u32) -> String { format_hz(mhz as u64 * 10u64.pow(6)) }

        fn mw(mw: u32) -> String { format_uw(mw as u64 * 10u64.pow(3)) }

        fn pad(s: &str, left: bool) -> String {
            const WIDTH: i64 = 19; // width of the widest label we expect to display
            let len: i64 = s.len().try_into().unwrap();
            let pad = (WIDTH-len).max(0) as usize;
            if left {
                format!("{}{}", " ".repeat(pad), s)
            } else {
                format!("{}{}", s, " ".repeat(pad))
            }
        }

        fn padl(s: &str) -> String { pad(s, true) }

        fn padr(s: &str) -> String { pad(s, false) }

        struct NvmlTable<'a> {
            tab: Table,
            devs: &'a [nvml::Device],
        }

        impl<'a> NvmlTable<'a> {
            fn new(devs: &'a [nvml::Device]) -> Option<Self> {
                let mut header = vec![padl("Nvidia GPU")];
                for dev in devs {
                    let id = if let Some(id) = dev.card().id() { id } else { continue; };
                    header.push(padr(&format!("{}", id)));
                }
                if header.len() == 1 { return None; }
                let header: Vec<&str> = header.iter().map(String::as_str).collect();
                let tab = Table::new(&header);
                let s = Self {
                    tab,
                    devs,
                };
                Some(s)
            }

            fn row<F>(&mut self, label: &str, mut f: F)
            where
                F: FnMut(&nvml::Device) -> String,
            {
                let mut row = vec![padl(label)];
                for dev in self.devs { row.push(f(dev)) }
                self.tab.row(&row);
            }

            fn into_table(self) -> Table { self.tab }
        }

        use nvml::Clock as _;
        const DEVICES_PER_TABLE: usize = 2;
        let devices = if let Some(d) = Self::devices() { d } else { return Ok(()); };
        if devices.is_empty() { return Ok(()); }
        let spam = regex::Regex::new("(?i)nvidia ").unwrap();
        for devs in devices.chunks(DEVICES_PER_TABLE) {
            let mut tab = if let Some(tab) = NvmlTable::new(devs) { tab } else { continue; };
            tab.row("Name", |d|
                d.hardware().name().map(|v| spam.replace_all(&v, "").into_owned()).unwrap_or_else(dot)
            );
            tab.row("PCI ID", |d| d.pci().bus_id().unwrap_or_else(dot));
            tab.row("Graphics cur/max", |d| {
                format!(
                    "{} / {}",
                    d.clocks().graphics().current().map(mhz).unwrap_or_else(dot),
                    d.clocks().graphics().max().map(mhz).unwrap_or_else(dot),
                )
            });
            tab.row("Memory cur/max", |d| {
                format!(
                    "{} / {}",
                    d.clocks().memory().current().map(mhz).unwrap_or_else(dot),
                    d.clocks().memory().max().map(mhz).unwrap_or_else(dot),
                )
            });
            tab.row("SM cur/max", |d| {
                format!(
                    "{} / {}",
                    d.clocks().sm().current().map(mhz).unwrap_or_else(dot),
                    d.clocks().sm().max().map(mhz).unwrap_or_else(dot),
                )
            });
            tab.row("Video cur/max", |d| {
                format!(
                    "{} / {}",
                    d.clocks().video().current().map(mhz).unwrap_or_else(dot),
                    d.clocks().video().max().map(mhz).unwrap_or_else(dot),
                )
            });
            tab.row("Memory used/total", |d| {
                format!(
                    "{} / {}",
                    d.memory().used().map(format_bytes).unwrap_or_else(dot),
                    d.memory().total().map(format_bytes).unwrap_or_else(dot),
                )
            });
            tab.row("Power used/limit", |d| {
                format!(
                    "{} / {}",
                    d.power().usage().map(mw).unwrap_or_else(dot),
                    d.power().limit().map(mw).unwrap_or_else(dot),
                )
            });
            tab.row("Power limit min/max", |d| {
                format!(
                    "{} / {}",
                    d.power().min().map(mw).unwrap_or_else(dot),
                    d.power().max().map(mw).unwrap_or_else(dot),
                )
            });
            w.write_all(nl(tab.into_table().to_string()).as_bytes()).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Format for sysfs::intel_pstate::IntelPstate {
    type Err = Error;

    async fn format_values<W: AsyncWrite + Send + Unpin>(&self, w: &mut W) -> Result<()> {

        fn system_status(status: &str) -> String {
            format!(" intel_pstate: {}\n\n", status)
        }

        fn epb_epp(policies: &[sysfs::intel_pstate::Policy]) -> Option<String> {
            let mut values: Vec<(u64, String)> = policies
                .iter()
                .filter_map(|p| p.energy_perf_bias
                    .and_then(|epb| p.energy_performance_preference
                        .as_ref()
                        .map(|epp| (epb, epp.to_string())))
                )
                .collect();
            if values.is_empty() { return None; }
            values.sort_unstable();
            values.dedup();
            let mut tab = Table::new(&["CPU", "EP bias", "EP preference"]);
            if values.len() == 1 {
                let values = values.into_iter().next().unwrap();
                tab.row(&[
                    "all".to_string(),
                    values.0.to_string(),
                    values.1,
                ]);
            } else {
                for policy in policies {
                    tab.row(&[
                        policy.id.map(|v| v.to_string()).unwrap_or_else(dot),
                        policy.energy_perf_bias.map(|v| v.to_string()).unwrap_or_else(dot),
                        policy.energy_performance_preference.clone().unwrap_or_else(dot),
                    ]);
                }
            }
            Some(nl(tab.to_string()))
        }

        fn epps(policies: &[sysfs::intel_pstate::Policy]) -> Option<String> {
            let mut prefs: Vec<String> = policies
                .iter()
                .filter_map(|p| p.energy_performance_available_preferences.clone().map(|p| p.join(" ")))
                .collect();
            prefs.sort_unstable();
            prefs.dedup();
            if prefs.is_empty() { return None; }
            let mut tab = Table::new(&["CPU", "Available EP preferences"]);
            if prefs.len() == 1 {
                tab.row(&["all", &prefs[0]]);
            } else {
                for policy in policies {
                    tab.row(&[
                        policy.id.map(|v| v.to_string()).unwrap_or_else(dot),
                        policy.energy_performance_available_preferences.clone().map(|v| v.join(" ")).unwrap_or_else(dot),
                    ]);
                }
            }
            Some(nl(tab.to_string()))
        }

        use zysfs::io::intel_pstate::tokio::status;

        if let Some(p) = &self.policies {
            if !p.is_empty() {
                if let Ok(status) = status().await {
                    w.write_all(system_status(&status).as_bytes()).await?;
                    if status == "active" {
                        if let Some(s) = epb_epp(p) { w.write_all(s.as_bytes()).await?; }
                        if let Some(s) = epps(p) { w.write_all(s.as_bytes()).await?; }
                    }
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Format for sysfs::intel_rapl::IntelRapl {
    type Err = Error;

    async fn format_values<W: AsyncWrite + Send + Unpin>(&self, w: &mut W) -> Result<()> {
        let policies = if let Some(p) = self.policies.as_ref() { p } else { return Ok(()); };
        if policies.is_empty() { return Ok(()); }
        let mut tab = Table::new(&["Zone name", "Zone", "Long lim", "Short lim", "Long win", "Short win", "Energy"]);
        for policy in policies {
            let id = if let Some(id) = policy.id { id } else { continue; };
            let long = policy.constraints
                .as_deref()
                .and_then(|v| v
                    .iter()
                    .find(|p| p.name.as_ref().map(|s| s == "long_term").unwrap_or(false)));
            let short = policy.constraints
                .as_deref()
                .and_then(|v| v
                    .iter()
                    .find(|p| p.name.as_ref().map(|s| s == "short_term").unwrap_or(false)));
            tab.row(&[
                policy.name.clone().unwrap_or_else(dot),
                format!(
                    "{}{}",
                    id.zone,
                    id.subzone.map(|v| format!(":{}", v)).unwrap_or_else(String::new)
                ),
                long
                    .and_then(|v| v.power_limit_uw)
                    .map(format_uw)
                    .unwrap_or_else(dot),
                short
                    .and_then(|v| v.power_limit_uw)
                    .map(format_uw)
                    .unwrap_or_else(dot),
                long
                    .and_then(|v| v.time_window_us)
                    .map(|v| format!("{} us", v))
                    .unwrap_or_else(dot),
                short
                    .and_then(|v| v.time_window_us)
                    .map(|v| format!("{} us", v))
                    .unwrap_or_else(dot),
                policy.energy_uj
                    .map(format_uj)
                    .unwrap_or_else(dot),
            ]);
        }
        w.write_all(nl(tab.to_string()).as_bytes()).await?;
        Ok(())
    }
}
