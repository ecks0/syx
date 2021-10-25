use measurements::{Power, Energy};
use zysfs::types::intel_rapl::Policy;
use zysfs::types::std::Read as _;
use crate::format::{dot, Table};

pub fn format() -> Option<String> {
    let pols = Policy::all()?;
    if pols.is_empty() { return None; }
    let mut tab = Table::new(&["Name", "Pkg", "Zone", "Enabled", "Long-term", "Short-term", "Cur", "Max"]);
    for pol in pols {
        let id = if let Some(id) = pol.id { id } else { continue; };
        tab.row(&[
            pol.name.clone().unwrap_or_else(dot),
            id.zone.to_string(),
            id.subzone.map(|v| v.to_string()).unwrap_or_else(dot),
            pol.enabled.map(|v| v.to_string()).unwrap_or_else(dot),
            pol.constraints
                .as_deref()
                .and_then(|v| v
                    .iter()
                    .find(|p| matches!(p.name.as_deref(), Some("long_term")))
                    .and_then(|v| v.power_limit_uw)
                    .map(|v| format!("{:.1}", Power::from_microwatts(v as f64))))
                .unwrap_or_else(dot),
            pol.constraints
                .as_deref()
                .and_then(|v| v
                    .iter()
                    .find(|p| matches!(p.name.as_deref(), Some("short_term")))
                    .and_then(|v| v.power_limit_uw)
                    .map(|v| format!("{:.1}", Power::from_microwatts(v as f64))))
                .unwrap_or_else(dot),
            pol.energy_uj
                .map(|v| format!("{:.1}", Energy::from_joules((v as f64/10f64.powf(6.)) as f64)))
                .unwrap_or_else(dot),
            pol.max_energy_range_uj
                .map(|v| format!("{:.1}", Energy::from_joules((v as f64/10f64.powf(6.)) as f64)))
                .unwrap_or_else(dot),
        ]);
    }
    Some(tab.to_string())
}
