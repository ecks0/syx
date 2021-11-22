use std::fmt::Display;

use comfy_table as ct;
use measurements::{Frequency, Power};
use tokio::io::{AsyncWrite, AsyncWriteExt, Error as IoError};

#[cfg(feature = "nvml")]
use crate::nvml;
use crate::sysfs::intel_rapl::Samplers;
use crate::{sysfs, Machine};

type Result<T> = std::result::Result<T, IoError>;

const DOT: &str = "\u{2022}";

fn dot() -> String {
    DOT.to_string()
}

fn nl(mut s: String) -> String {
    s.push('\n');
    s
}

fn uw(uw: u64) -> String {
    match uw {
        0 => "0 W".to_string(),
        _ => {
            let scale = 10u64.pow(match uw {
                v if v > 10u64.pow(18) => 15,
                v if v > 10u64.pow(15) => 12,
                v if v > 10u64.pow(12) => 9,
                v if v > 10u64.pow(9) => 6,
                v if v > 10u64.pow(6) => 3,
                _ => 0,
            });
            let uw = (uw / scale) * scale;
            Power::from_microwatts(uw as f64).to_string()
        },
    }
}

fn hz(hz: u64) -> String {
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
fn bytes(b: u64) -> String {
    if b < 1000 {
        format!("{} B", b)
    } else if b < 1000u64.pow(2) {
        format!("{:.1} kB", b as f64 / 1000f64)
    } else if b < 1000u64.pow(3) {
        format!("{:.1} MB", b as f64 / (1000u64.pow(2) as f64))
    } else if b < 1000u64.pow(4) {
        format!("{:.1} GB", b as f64 / (1000u64.pow(3) as f64))
    } else if b < 1000u64.pow(5) {
        format!("{:.1} TB", b as f64 / (1000u64.pow(4) as f64))
    } else if b < 1000u64.pow(6) {
        format!("{:.1} PB", b as f64 / (1000u64.pow(5) as f64))
    } else {
        format!("{:.1} TB", b as f64 / (1000u64.pow(4) as f64))
    }
}

#[derive(Debug)]
struct Table(ct::Table);

impl Table {
    pub(self) fn new(header: &[&str]) -> Self {
        let mut tab = ct::Table::new();
        tab.load_preset(ct::presets::NOTHING);
        tab.set_header(header);
        tab.add_row(
            header
                .iter()
                .map(|h| "-".repeat(h.len()))
                .collect::<Vec<String>>(),
        );
        Self(tab)
    }

    pub(self) fn row<S: Display>(&mut self, row: &[S]) {
        self.0.add_row(row);
    }
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.0)
    }
}

pub(in crate::cli) async fn cpu<W>(w: &mut W, machine: &Machine) -> Result<()>
where
    W: AsyncWrite + Send + Unpin,
{
    fn khz(khz: u64) -> String {
        hz(khz * 10u64.pow(3))
    }

    fn cpu_cpufreq(machine: &Machine) -> Option<String> {
        let cpu_devs = &machine.cpu.as_ref()?.devices;
        if cpu_devs.is_empty() {
            return None;
        }
        let cpufreq_devs = machine.cpufreq.as_ref().map(|c| &c.devices);
        let cpufreq_dev_default = sysfs::cpufreq::Device::default();
        let mut tab = Table::new(&[
            "CPU", "Online", "Governor", "Cur", "Min", "Max", "CPU min", "CPU max",
        ]);
        for cpu_dev in cpu_devs {
            let cpufreq_dev = cpufreq_devs
                .and_then(|p| p.iter().find(|p| cpu_dev.id == p.id))
                .unwrap_or(&cpufreq_dev_default);
            tab.row(&[
                cpu_dev.id.to_string(),
                cpu_dev.online.map(|v| v.to_string()).unwrap_or_else(dot),
                cpufreq_dev.scaling_governor.clone().unwrap_or_else(dot),
                cpufreq_dev.scaling_cur_freq.map(khz).unwrap_or_else(dot),
                cpufreq_dev.scaling_min_freq.map(khz).unwrap_or_else(dot),
                cpufreq_dev.scaling_max_freq.map(khz).unwrap_or_else(dot),
                cpufreq_dev.cpuinfo_min_freq.map(khz).unwrap_or_else(dot),
                cpufreq_dev.cpuinfo_max_freq.map(khz).unwrap_or_else(dot),
            ]);
        }
        Some(nl(tab.to_string()))
    }

    fn governors(machine: &Machine) -> Option<String> {
        let devices = &machine.cpufreq.as_ref()?.devices;
        let mut govs: Vec<String> = devices
            .iter()
            .filter_map(|d| {
                d.scaling_available_governors
                    .as_deref()
                    .map(|g| g.join(" "))
            })
            .collect();
        if govs.is_empty() {
            return None;
        }
        govs.sort_unstable();
        govs.dedup();
        let mut tab = Table::new(&["CPU", "Available governors"]);
        if govs.len() == 1 {
            tab.row(&["all", &govs[0]]);
        } else {
            for d in devices {
                tab.row(&[
                    d.id.to_string(),
                    d.scaling_available_governors
                        .as_ref()
                        .map(|v| v.join(" "))
                        .unwrap_or_else(dot),
                ])
            }
        }
        Some(nl(tab.to_string()))
    }

    if let Some(s) = cpu_cpufreq(machine) {
        w.write_all(s.as_bytes()).await?;
    }
    if let Some(s) = governors(machine) {
        w.write_all(s.as_bytes()).await?;
    }
    Ok(())
}

pub(in crate::cli) async fn i915<W>(w: &mut W, machine: &Machine) -> Result<()>
where
    W: AsyncWrite + Send + Unpin,
{
    fn mhz(mhz: u64) -> String {
        hz(mhz * 10u64.pow(6))
    }

    let devices = match machine.i915.as_ref().map(|i| &i.devices) {
        Some(d) => {
            if d.is_empty() {
                return Ok(());
            } else {
                d
            }
        },
        None => return Ok(()),
    };
    let mut tab = Table::new(&[
        "Card", "Driver", "Actual", "Req'd", "Min", "Max", "Boost", "GPU min", "GPU max",
    ]);
    for device in devices {
        tab.row(&[
            device.id.to_string(),
            "i915".to_string(),
            device.act_freq_mhz.map(mhz).unwrap_or_else(dot),
            device.cur_freq_mhz.map(mhz).unwrap_or_else(dot),
            device.min_freq_mhz.map(mhz).unwrap_or_else(dot),
            device.max_freq_mhz.map(mhz).unwrap_or_else(dot),
            device.boost_freq_mhz.map(mhz).unwrap_or_else(dot),
            device.rpn_freq_mhz.map(mhz).unwrap_or_else(dot),
            device.rp0_freq_mhz.map(mhz).unwrap_or_else(dot),
        ]);
    }
    let s = nl(tab.to_string());
    w.write_all(s.as_bytes()).await?;
    Ok(())
}

pub(in crate::cli) async fn intel_pstate<W>(w: &mut W, machine: &Machine) -> Result<()>
where
    W: AsyncWrite + Send + Unpin,
{
    fn status(machine: &Machine) -> Option<String> {
        let status = machine
            .intel_pstate
            .as_ref()?
            .system
            .as_ref()?
            .status
            .as_ref()?;
        Some(format!(" intel_pstate: {}\n\n", status))
    }

    fn epb_epp(devices: &[sysfs::intel_pstate::Device]) -> String {
        let mut values: Vec<(u64, String)> = devices
            .iter()
            .filter_map(|d| {
                d.energy_perf_bias.and_then(|epb| {
                    d.energy_performance_preference
                        .as_ref()
                        .map(|epp| (epb, epp.to_string()))
                })
            })
            .collect();
        values.sort_unstable();
        values.dedup();
        let mut tab = Table::new(&["CPU", "EP bias", "EP preference"]);
        if values.len() == 1 {
            let values = values.into_iter().next().unwrap();
            tab.row(&["all".to_string(), values.0.to_string(), values.1]);
        } else {
            for device in devices {
                tab.row(&[
                    device.id.to_string(),
                    device
                        .energy_perf_bias
                        .map(|v| v.to_string())
                        .unwrap_or_else(dot),
                    device
                        .energy_performance_preference
                        .clone()
                        .unwrap_or_else(dot),
                ]);
            }
        }
        nl(tab.to_string())
    }

    fn epps(devices: &[sysfs::intel_pstate::Device]) -> String {
        let mut prefs: Vec<String> = devices
            .iter()
            .filter_map(|d| {
                d.energy_performance_available_preferences
                    .clone()
                    .map(|p| p.join(" "))
            })
            .collect();
        prefs.sort_unstable();
        prefs.dedup();
        let mut tab = Table::new(&["CPU", "Available EP preferences"]);
        if prefs.len() == 1 {
            tab.row(&["all", &prefs[0]]);
        } else {
            for device in devices {
                tab.row(&[
                    device.id.to_string(),
                    device
                        .energy_performance_available_preferences
                        .clone()
                        .map(|v| v.join(" "))
                        .unwrap_or_else(dot),
                ]);
            }
        }
        nl(tab.to_string())
    }

    if let Some(s) = status(machine) {
        w.write_all(s.as_bytes()).await?;
    }
    let active = machine
        .intel_pstate
        .as_ref()
        .and_then(|i| {
            i.system
                .as_ref()
                .and_then(|s| s.status.as_ref().map(|s| "active" == s.as_str()))
        })
        .unwrap_or(false);
    if active {
        let devices = match machine.intel_pstate.as_ref().map(|i| &i.devices) {
            Some(p) => {
                if p.is_empty() {
                    return Ok(());
                } else {
                    p
                }
            },
            None => return Ok(()),
        };
        let s = epb_epp(devices);
        w.write_all(s.as_bytes()).await?;
        let s = epps(devices);
        w.write_all(s.as_bytes()).await?;
    }
    Ok(())
}

pub(in crate::cli) async fn intel_rapl<W>(
    w: &mut W,
    machine: &Machine,
    samplers: Option<Samplers>,
) -> Result<()>
where
    W: AsyncWrite + Send + Unpin,
{
    let devices = match machine.intel_rapl.as_ref().map(|i| &i.devices) {
        Some(d) => {
            if d.is_empty() {
                return Ok(());
            } else {
                d
            }
        },
        None => return Ok(()),
    };
    let mut tab = Table::new(&[
        "Zone name",
        "Zone",
        "Long lim",
        "Short lim",
        "Long win",
        "Short win",
        "Usage",
    ]);
    for device in devices {
        let long = device
            .constraints
            .iter()
            .find(|p| p.name.as_ref().map(|s| s == "long_term").unwrap_or(false));
        let short = device
            .constraints
            .iter()
            .find(|p| p.name.as_ref().map(|s| s == "short_term").unwrap_or(false));
        let watt_seconds = if let Some(s) = &samplers {
            s.watt_seconds(device.id).await.map(Power::from_watts)
        } else {
            None
        };
        tab.row(&[
            device.name.clone().unwrap_or_else(dot),
            format!(
                "{}{}",
                device.id.zone,
                device
                    .id
                    .subzone
                    .map(|v| format!(":{}", v))
                    .unwrap_or_else(String::new)
            ),
            long.and_then(|v| v.power_limit_uw)
                .map(uw)
                .unwrap_or_else(dot),
            short
                .and_then(|v| v.power_limit_uw)
                .map(uw)
                .unwrap_or_else(dot),
            long.and_then(|v| v.time_window_us)
                .map(|v| format!("{} us", v))
                .unwrap_or_else(dot),
            short
                .and_then(|v| v.time_window_us)
                .map(|v| format!("{} us", v))
                .unwrap_or_else(dot),
            watt_seconds
                .map(|p| {
                    if p.as_microwatts() == 0. {
                        "0 W".to_string()
                    } else {
                        format!("{:.1}", p)
                    }
                })
                .unwrap_or_else(dot),
        ]);
    }
    let s = nl(tab.to_string());
    w.write_all(s.as_bytes()).await?;
    Ok(())
}

#[cfg(feature = "nvml")]
pub(in crate::cli) async fn nvml<W>(w: &mut W, machine: &Machine) -> Result<()>
where
    W: AsyncWrite + Send + Unpin,
{
    const WIDTH: i64 = 19; // width of the widest label we expect to display

    fn mhz(mhz: u32) -> String {
        hz(mhz as u64 * 10u64.pow(6))
    }

    fn mw(mw: u32) -> String {
        uw(mw as u64 * 10u64.pow(3))
    }

    fn pad(s: &str, left: bool) -> String {
        let len: i64 = s.len().try_into().unwrap();
        let pad = (WIDTH - len).max(0) as usize;
        if left {
            format!("{}{}", " ".repeat(pad), s)
        } else {
            format!("{}{}", s, " ".repeat(pad))
        }
    }

    fn padl(s: &str) -> String {
        pad(s, true)
    }

    fn padr(s: &str) -> String {
        pad(s, false)
    }

    struct NvmlTable<'a> {
        tab: Table,
        devs: &'a [nvml::Device],
    }

    impl<'a> NvmlTable<'a> {
        fn new(devs: &'a [nvml::Device]) -> Option<Self> {
            let mut header = vec![padl("Nvidia GPU")];
            for dev in devs {
                header.push(padr(&format!("{}", dev.id)));
            }
            if header.len() == 1 {
                return None;
            }
            let header: Vec<&str> = header.iter().map(String::as_str).collect();
            let tab = Table::new(&header);
            let s = Self { tab, devs };
            Some(s)
        }

        fn row<F>(&mut self, label: &str, mut f: F)
        where
            F: FnMut(&nvml::Device) -> String,
        {
            let mut row = vec![padl(label)];
            for dev in self.devs {
                row.push(f(dev))
            }
            self.tab.row(&row);
        }

        fn into_table(self) -> Table {
            self.tab
        }
    }

    const DEVICES_PER_TABLE: usize = 2;
    let devices = match machine.nvml.as_ref().map(|n| &n.devices) {
        Some(d) => {
            if d.is_empty() {
                return Ok(());
            } else {
                d
            }
        },
        None => return Ok(()),
    };
    let spam = regex::Regex::new("(?i)nvidia ").unwrap();
    for devs in devices.chunks(DEVICES_PER_TABLE) {
        let mut tab = if let Some(tab) = NvmlTable::new(devs) {
            tab
        } else {
            continue;
        };
        tab.row("Name", |d| {
            d.name
                .clone()
                .map(|v| spam.replace_all(&v, "").into_owned())
                .unwrap_or_else(dot)
        });
        tab.row("PCI ID", |d| d.pci_id.clone().unwrap_or_else(dot));
        tab.row("Graphics cur/max", |d| {
            format!(
                "{} / {}",
                d.gfx_freq_cur.map(mhz).unwrap_or_else(dot),
                d.gfx_freq_max.map(mhz).unwrap_or_else(dot),
            )
        });
        tab.row("Memory cur/max", |d| {
            format!(
                "{} / {}",
                d.mem_freq_cur.map(mhz).unwrap_or_else(dot),
                d.mem_freq_max.map(mhz).unwrap_or_else(dot),
            )
        });
        tab.row("SM cur/max", |d| {
            format!(
                "{} / {}",
                d.sm_freq_cur.map(mhz).unwrap_or_else(dot),
                d.sm_freq_max.map(mhz).unwrap_or_else(dot),
            )
        });
        tab.row("Video cur/max", |d| {
            format!(
                "{} / {}",
                d.video_freq_cur.map(mhz).unwrap_or_else(dot),
                d.video_freq_max.map(mhz).unwrap_or_else(dot),
            )
        });
        tab.row("Memory used/total", |d| {
            format!(
                "{} / {}",
                d.mem_used.map(bytes).unwrap_or_else(dot),
                d.mem_total.map(bytes).unwrap_or_else(dot),
            )
        });
        tab.row("Power cur/limit", |d| {
            format!(
                "{} / {}",
                d.power_cur.map(mw).unwrap_or_else(dot),
                d.power_limit.map(mw).unwrap_or_else(dot),
            )
        });
        tab.row("Power limit min/max", |d| {
            format!(
                "{} / {}",
                d.power_min.map(mw).unwrap_or_else(dot),
                d.power_max.map(mw).unwrap_or_else(dot),
            )
        });
        let s = nl(tab.into_table().to_string());
        w.write_all(s.as_bytes()).await?;
    }
    Ok(())
}
