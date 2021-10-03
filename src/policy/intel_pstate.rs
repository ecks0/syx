use crate::cli::Cli;
use zysfs::types::devices::system::cpu::intel_pstate::{IntelPstate, Policy};

pub fn policy(cli: &Cli, cpu_ids: &[u64]) -> Option<IntelPstate> {
    if !cli.has_intel_pstate_args() || cpu_ids.is_empty() { return None; }
    let mut policies = vec![];
    for cpu_id in cpu_ids {
        let policy = Policy {
            id: Some(*cpu_id),
            energy_perf_bias: cli.pstate_epb,
            energy_performance_preference: cli.pstate_epp.clone(),
            ..Default::default()
        };
        policies.push(policy);
    }
    Some(IntelPstate { policies: Some(policies), ..Default::default() })
}
