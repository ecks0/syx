use comfy_table as ct;
use measurements::{Energy, Frequency, Power};
use std::fmt::Display;

mod cpu;
mod drm;
mod intel_pstate;
mod intel_rapl;
#[cfg(feature = "nvml")]
mod nvml;

pub use cpu::format as cpu;
pub use intel_pstate::format as intel_pstate;
pub use drm::format as drm;
pub use intel_rapl::format as intel_rapl;
#[cfg(feature = "nvml")]
pub use nvml::format as nvml;

fn dot() -> String { "\u{2022}".to_string() }

#[derive(Debug)]
struct Table(ct::Table);

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

fn format_bytes(b: u64) -> String {
    if b < 1000 { format!("{} B", b) }
    else if b < 1000u64.pow(2) { format!("{:.1} kB", b as f64/1000f64) }
    else if b < 1000u64.pow(3) { format!("{:.1} MB", b as f64/(1000u64.pow(2) as f64)) }
    else if b < 1000u64.pow(4) { format!("{:.1} GB", b as f64/(1000u64.pow(3) as f64)) }
    else if b < 1000u64.pow(5) { format!("{:.1} TB", b as f64/(1000u64.pow(4) as f64)) }
    else if b < 1000u64.pow(6) { format!("{:.1} PB", b as f64/(1000u64.pow(5) as f64)) }
    else { format!("{:.1} TB", b as f64/(1000u64.pow(4) as f64)) }
}
