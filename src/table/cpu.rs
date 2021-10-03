use measurements::Frequency;
use tabular::{Row, Table};
use zysfs::types::blocking::Read as _;
use zysfs::types::devices::system::cpu::Policy as CpuPolicy;
use zysfs::types::devices::system::cpu::cpufreq::Policy as CpufreqPolicy;
use super::dot;

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
    let mut tab = Table::new("{:<} {:<} {:<} {:<} {:<} {:<} {:<} {:<}");
    let mut row = |a: &str, b: &str, c: &str, d: &str, e: &str, f: &str, g: &str, h: &str| {
        tab.add_row(Row::new()
            .with_cell(a).with_cell(b).with_cell(c).with_cell(d)
            .with_cell(e).with_cell(f).with_cell(g).with_cell(h));
    };
    row("CPU", "Online", "Governor", "Cur", "Min", "Max", "Min limit", "Max limit");
    row("----", "-------", "------------", "----------", "----------", "----------", "----------", "----------");
    for cpu_pol in cpu_pols {
        let id = if let Some(id) = cpu_pol.id { id } else { continue; };
        let cpufreq_pol = cpufreq_policy(id);
        row(
            &id.to_string(),
            &cpu_pol.cpu_online.map(|v| v.to_string()).unwrap_or_else(dot),
            &cpufreq_pol.scaling_governor.clone().unwrap_or_else(dot),
            &cpufreq_pol.scaling_cur_freq.map(khz).unwrap_or_else(dot),
            &cpufreq_pol.scaling_min_freq.map(khz).unwrap_or_else(dot),
            &cpufreq_pol.scaling_max_freq.map(khz).unwrap_or_else(dot),
            &cpufreq_pol.cpuinfo_min_freq.map(khz).unwrap_or_else(dot),
            &cpufreq_pol.cpuinfo_max_freq.map(khz).unwrap_or_else(dot),
        );
    }
    Some(tab.to_string())
}

fn format_governors(policies: &[CpufreqPolicy]) -> Option<String> {
    if policies.is_empty() { return None; }
    let mut governors: Vec<String> = policies
        .iter()
        .filter_map(|p| p.scaling_available_governors.clone().map(|g| g.join(" ")))
        .collect();
    governors.sort();
    governors.dedup();
    if governors.is_empty() { return None; }
    let mut tab = Table::new("{:<} {:<}");
    let mut row = |a: &str, b: &str| { tab.add_row(Row::new().with_cell(a).with_cell(b)); };
    row("CPU", "Available governors");
    row("----", "--------------------");
    if governors.len() == 1 {
        row("all", &governors[0]);
    } else {
        for p in policies {
            row(
                &p.id.map(|v| v.to_string()).unwrap_or_else(dot),
                &p.scaling_available_governors.clone().map(|v| v.join(" ")).unwrap_or_else(dot),
            );
        }
    }
    Some(tab.to_string())
}

fn format() -> Option<String> {
    let cpu_pols = CpuPolicy::all()?;
    let cpufreq_pols = CpufreqPolicy::all().unwrap_or_else(Vec::new);
    let mut s = vec![];
    if let Some(ss) = format_cpu_cpufreq(&cpu_pols, &cpufreq_pols) { s.push(ss); }
    if let Some(ss) = format_governors(&cpufreq_pols) { s.push(ss); }
    if s.is_empty() { None } else { Some(s.join("\n")) }
}

pub fn print() {
    if let Some(s) = format() {
        println!("{}", s);
    }
}
