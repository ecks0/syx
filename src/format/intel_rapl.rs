use zysfs::types::intel_rapl::Policy;
use zysfs::types::std::Read as _;
use crate::format::{Table, dot, format_uj, format_uw};

pub fn format() -> Option<String> {
    let pols = Policy::all()?;
    if pols.is_empty() { return None; }
    let mut tab = Table::new(&["Name", "Zone", "C0 limit", "C1 limit", "C0 window", "C1 window", "Energy"]);
    for pol in pols {
        let id = if let Some(id) = pol.id { id } else { continue; };
        let c0 = pol.constraints
            .as_deref()
            .and_then(|v| v
                .iter()
                .find(|p| matches!(p.id, Some(0))));
        let c1 = pol.constraints
            .as_deref()
            .and_then(|v| v
                .iter()
                .find(|p| matches!(p.id, Some(1))));
        tab.row(&[
            pol.name.clone().unwrap_or_else(dot),
            format!(
                "{}{}",
                id.zone,
                id.subzone.map(|v| format!(":{}", v)).unwrap_or_else(String::new)
            ),
            c0
                .and_then(|v| v.power_limit_uw)
                .map(format_uw)
                .unwrap_or_else(dot),
            c1
                .and_then(|v| v.power_limit_uw)
                .map(format_uw)
                .unwrap_or_else(dot),
            c0
                .and_then(|v| v.time_window_us)
                .map(|v| format!("{} us", v))
                .unwrap_or_else(dot),
            c1
                .and_then(|v| v.time_window_us)
                .map(|v| format!("{} us", v))
                .unwrap_or_else(dot),
            pol.energy_uj
                .map(format_uj)
                .unwrap_or_else(dot),
        ]);
    }
    Some(tab.to_string())
}
