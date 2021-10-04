use comfy_table as ct;
use std::fmt::Display;

mod cpu;
mod drm;
mod intel_pstate;

fn dot() -> String { "•".to_string() }

#[derive(Debug)]
struct Table(ct::Table);

impl Table {
    pub fn new<S: Display>(header: &[S]) -> Self {
        let headers: Vec<String> = header
            .iter()
            .map(|d| d.to_string())
            .collect();
        let mut tab = ct::Table::new();
        tab.load_preset(ct::presets::NOTHING);
        tab.set_header(&headers);
        tab.add_row(
            headers
                .iter()
                .map(|h| "-".repeat(h.len().max(4)))
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

pub fn format() -> Option<String> {
    let mut s = vec![];
    if let Some(ss) = cpu::format() { s.push(ss); }
    if let Some(ss) = intel_pstate::format() { s.push(ss); }
    if let Some(ss) = drm::format() { s.push(ss); }
    if s.is_empty() { None } else { Some(s.join("\n")) }
}
