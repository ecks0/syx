use zysfs::io::devices::system::cpu::intel_pstate::blocking::status;
use zysfs::types::blocking::Read as _;
use zysfs::types::devices::system::cpu::intel_pstate::Policy;
use crate::timer::Timer;
use super::{dot, Table};

fn format_status(status: &str) -> Option<String> {
    Some(format!(" intel_pstate: {}\n", status))
}

fn format_epb_epp(policies: &[Policy]) -> Option<String> {
    if policies.is_empty() { return None; }
    let mut tab = Table::new(&["CPU", "EP bias", "EP preference"]);
    for policy in policies {
        tab.row(&[
            &policy.id.map(|v| v.to_string()).unwrap_or_else(dot),
            &policy.energy_perf_bias.map(|v| v.to_string()).unwrap_or_else(dot),
            &policy.energy_performance_preference.clone().unwrap_or_else(dot),
        ]);
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
    let mut tab = Table::new(&["CPU", "Available EP preferences"]);
    if prefs.len() == 1 {
        tab.row(&["all", &prefs[0]]);
    } else {
        for policy in policies {
            tab.row(&[
                &policy.id.map(|v| v.to_string()).unwrap_or_else(dot),
                &policy.energy_performance_available_preferences.clone().map(|v| v.join(" ")).unwrap_or_else(dot),
            ]);
        }
    }
    Some(tab.to_string())
}

pub fn format() -> Option<String> {
    let mut t = Timer::start();

    let status = status().ok()?;
    t.end(".. Load intel_pstate status");

    let mut s = vec![];
    t.reset();

    if let Some(ss) = format_status(&status) {
        s.push(ss);
        t.end(".. Format intel_pstate status table");
    }

    if status == "active" {

        if let Some(policies) = Policy::all() {
            t.end(".. Load intel_pstate policies");

            if let Some(ss) = format_epb_epp(&policies) {
                s.push(ss);
                t.end(".. Format intel_pstate epb/epp table");
            }

            if let Some(ss) = format_available_epps(&policies) {
                s.push(ss);
                t.end(".. Format intel_pstate available epps table");
            }
        }
    }
    if s.is_empty() { None } else { Some(s.join("\n")) }
}
