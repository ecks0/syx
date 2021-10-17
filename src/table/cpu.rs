use measurements::Frequency;
use zysfs::types::blocking::Read as _;
use zysfs::types::devices::system::cpu::Policy as CpuPolicy;
use zysfs::types::devices::system::cpu::cpufreq::Policy as CpufreqPolicy;
use crate::table::{dot, Table};

fn khz(khz: u64) -> String {
    let f = Frequency::from_kilohertz(khz as f64);
    if khz >= 1000u64.pow(2) { format!("{:.1}", f) } else { format!("{:.0}", f) }
}

fn format_cpu_cpufreq(cpu_pols: &[CpuPolicy], cpufreq_pols: &[CpufreqPolicy]) -> Option<String> {
    if cpu_pols.is_empty() { return None; }
    let cpufreq_pol_default = CpufreqPolicy::default();
    let cpufreq_policy = |id: u64| -> &CpufreqPolicy {
        cpufreq_pols.iter().find(|p| Some(id) == p.id).unwrap_or(&cpufreq_pol_default)
    };
    let mut tab = Table::new(&["CPU", "Online", "Governor", "Cur", "Min", "Max", "CPU min", "CPU max"]);
    for cpu_pol in cpu_pols {
        let id = if let Some(id) = cpu_pol.id { id } else { continue; };
        let cpufreq_pol = cpufreq_policy(id);
        tab.row(&[
            id.to_string(),
            cpu_pol.cpu_online.map(|v| v.to_string()).unwrap_or_else(dot),
            cpufreq_pol.scaling_governor.clone().unwrap_or_else(dot),
            cpufreq_pol.scaling_cur_freq.map(khz).unwrap_or_else(dot),
            cpufreq_pol.scaling_min_freq.map(khz).unwrap_or_else(dot),
            cpufreq_pol.scaling_max_freq.map(khz).unwrap_or_else(dot),
            cpufreq_pol.cpuinfo_min_freq.map(khz).unwrap_or_else(dot),
            cpufreq_pol.cpuinfo_max_freq.map(khz).unwrap_or_else(dot),
        ]);
    }
    Some(tab.to_string())
}

fn format_governors(policies: &[CpufreqPolicy]) -> Option<String> {
    let mut govs: Vec<String> = policies
        .iter()
        .filter_map(|p| p.scaling_available_governors.clone().map(|g| g.join(" ")))
        .collect();
    govs.sort_unstable();
    govs.dedup();
    if govs.is_empty() { return None; }
    let mut tab = Table::new(&["CPU", "Available governors"]);
    if govs.len() == 1 {
        tab.row(&["all", &govs[0]]);
    } else {
        for p in policies {
            tab.row(&[
                &p.id.map(|v| v.to_string()).unwrap_or_else(dot),
                &p.scaling_available_governors.clone().map(|v| v.join(" ")).unwrap_or_else(dot),
            ]);
        }
    }
    Some(tab.to_string())
}

pub fn format() -> Option<String> {
    let cpu_pols = CpuPolicy::all()?;
    let cpufreq_pols = CpufreqPolicy::all().unwrap_or_else(Vec::new);
    let mut s = vec![];
    if let Some(ss) = format_cpu_cpufreq(&cpu_pols, &cpufreq_pols) { s.push(ss); }
    if let Some(ss) = format_governors(&cpufreq_pols) { s.push(ss); }
    if s.is_empty() { None } else { Some(s.join("\n")) }
}
