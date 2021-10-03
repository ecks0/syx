use tabular::{Row, Table};
use zysfs::io::devices::system::cpu::intel_pstate::blocking::status;
use zysfs::types::blocking::Read as _;
use zysfs::types::devices::system::cpu::intel_pstate::Policy;
use super::dot;

fn format_status(status: &str) -> Option<String> {
    Some(format!("intel_pstate: {}\n", status))
}

fn format_epb_epp(policies: &[Policy]) -> Option<String> {
    if policies.is_empty() { return None; }
    let mut tab = Table::new("{:<} {:<} {:<}");
    let mut row = |a: &str, b: &str, c: &str| { tab.add_row(Row::new().with_cell(a).with_cell(b).with_cell(c)); };
    row("CPU", "EP bias", "EP preference");
    row("----", "--------", "--------------");
    for policy in policies {
        row(
            &policy.id.map(|v| v.to_string()).unwrap_or_else(dot),
            &policy.energy_perf_bias.map(|v| v.to_string()).unwrap_or_else(dot),
            &policy.energy_performance_preference.clone().unwrap_or_else(dot),
        );
    }
    Some(tab.to_string())
}

fn format_available_epps(policies: &[Policy]) -> Option<String> {
    if policies.is_empty() { return None; }
    let mut prefs: Vec<String> = policies
        .iter()
        .filter_map(|p| p.energy_performance_available_preferences.clone().map(|p| p.join(" ")))
        .collect();
    prefs.sort_unstable();
    prefs.dedup();
    if prefs.is_empty() { return None; }
    let mut tab = Table::new("{:<} {:<}");
    let mut row = |a: &str, b: &str| { tab.add_row(Row::new().with_cell(a).with_cell(b)); };
    row("CPU", "Available EP preferences");
    row("----", "-------------------------");
    if prefs.len() == 1 {
        row("all", &prefs[0]);
    } else {
        for policy in policies {
            row(
                &policy.id.map(|v| v.to_string()).unwrap_or_else(dot),
                &policy.energy_performance_available_preferences.clone().map(|v| v.join(" ")).unwrap_or_else(dot),
            );
        }
    }
    Some(tab.to_string())
}

pub fn format() -> Option<String> {
    let status = status().ok()?;
    let mut s = vec![];
    if let Some(ss) = format_status(&status) { s.push(ss); }
    if status == "active" {
        if let Some(policies) = Policy::all() {
            if let Some(ss) = format_epb_epp(&policies) { s.push(ss); }
            if let Some(ss) = format_available_epps(&policies) { s.push(ss); }
        }
    }
    if s.is_empty() { None } else { Some(s.join("\n")) }
}
