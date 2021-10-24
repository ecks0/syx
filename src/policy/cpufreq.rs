use crate::cli::Cli;
use zysfs::types::devices::system::cpufreq::{Cpufreq, Policy};
use zysfs::types::std::Read as _;

impl From<&Cli> for Option<Cpufreq> {
    fn from(cli: &Cli) -> Self {
        if !cli.has_cpufreq_args() { return None; }
        let policy_ids = cli.cpu.clone().or_else(Policy::ids)?;
        let scaling_min_freq = cli.cpufreq_min.map(|f| f.as_kilohertz() as u64);
        let scaling_max_freq = cli.cpufreq_max.map(|f| f.as_kilohertz() as u64);
        let mut policies = vec![];
        for policy_id in policy_ids {
            let policy = Policy {
                id: Some(policy_id),
                scaling_governor: cli.cpufreq_gov.clone(),
                scaling_min_freq,
                scaling_max_freq,
                ..Default::default()
            };
            policies.push(policy);
        }
        let s = Cpufreq {
            policies: Some(policies),
            ..Default::default()
        };
        Some(s)
    }
}
