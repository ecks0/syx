use zysfs::types::intel_rapl::Policy;
use zysfs::types::std::Read as _;
use crate::format::{dot, Table};

pub fn format() -> Option<String> {
    let pols = Policy::all()?;
    if pols.is_empty() { return None; }
    let mut tab = Table::new(&["Name", "ID", "C0 lim", "C1 lim", "C0 max", "C1 max", "C0 win", "C1 win"]);
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
                .map(|v| v.to_string())
                .unwrap_or_else(dot),
            c1
                .and_then(|v| v.power_limit_uw)
                .map(|v| v.to_string())
                .unwrap_or_else(dot),
            c0
                .and_then(|v| v.max_power_uw)
                .map(|v| v.to_string())
                .unwrap_or_else(dot),
            c1
                .and_then(|v| v.max_power_uw)
                .map(|v| v.to_string())
                .unwrap_or_else(dot),
            c0
                .and_then(|v| v.time_window_us)
                .map(|v| v.to_string())
                .unwrap_or_else(dot),
            c1
                .and_then(|v| v.time_window_us)
                .map(|v| v.to_string())
                .unwrap_or_else(dot),
        ]);
    }
    Some(tab.to_string())
}
