use measurements::{Power, Energy};
use zysfs::types::intel_rapl::Policy;
use zysfs::types::std::Read as _;
use crate::format::{dot, Table};

pub fn format() -> Option<String> {
    let pols = Policy::all()?;
    if pols.is_empty() { return None; }
    let mut tab = Table::new(&["Name", "Pkg:Zone", "C0 Lim", "C1 Lim", "C0 Win", "C1 Win", "Energy", "Max"]);
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
                .map(|v| format!("{:.0}", Power::from_microwatts(v as f64)))
                .unwrap_or_else(dot),
            c1
                .and_then(|v| v.power_limit_uw)
                .map(|v| format!("{:.0}", Power::from_microwatts(v as f64)))
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
                .map(|v| format!("{:.1}", Energy::from_joules((v as f64/10f64.powf(6.)) as f64)))
                .unwrap_or_else(dot),
            pol.max_energy_range_uj
                .map(|v| format!("{:.1}", Energy::from_joules((v as f64/10f64.powf(6.)) as f64)))
                .unwrap_or_else(dot),
        ]);
    }
    Some(tab.to_string())
}
