use comfy_table as ct;
use std::fmt::Display;

mod cpu;
mod drm;
mod intel_pstate;
#[cfg(feature = "nvml")]
mod nvml;

pub use cpu::format as cpu;
pub use intel_pstate::format as intel_pstate;
pub use drm::format as drm;
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
