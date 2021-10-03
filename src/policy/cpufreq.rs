use crate::cli::Cli;
use zysfs::types::devices::system::cpu::cpufreq::{Cpufreq, Policy};

pub fn policy(cli: &Cli, cpu_ids: &[u64]) -> Option<Cpufreq> {
    if !cli.has_cpufreq_args() || cpu_ids.is_empty() { return None; }
    let mut policies = vec![];
    for cpu_id in cpu_ids {
        let policy = Policy {
            id: Some(*cpu_id),
            scaling_governor: cli.cpufreq_gov.clone(),
            scaling_min_freq: cli.cpufreq_min.map(|f| f.as_kilohertz() as u64),
            scaling_max_freq: cli.cpufreq_max.map(|f| f.as_kilohertz() as u64),
            ..Default::default()
        };
        policies.push(policy);
    }
    Some(Cpufreq { policies: Some(policies), ..Default::default() })
}
