use crate::cli::Cli;
use zysfs::types::intel_pstate::{IntelPstate, Policy};
use zysfs::types::std::Read as _;

impl From<&Cli> for Option<IntelPstate> {
    fn from(cli: &Cli) -> Self {
        if !cli.has_pstate_args() { return None; }
        let policy_ids = cli.cpu.clone().or_else(Policy::ids)?;
        let mut policies = vec![];
        for policy_id in policy_ids {
            let policy = Policy {
                id: Some(policy_id),
                energy_perf_bias: cli.pstate_epb,
                energy_performance_preference: cli.pstate_epp.clone(),
                ..Default::default()
            };
            policies.push(policy);
        }
        let s = IntelPstate {
            policies: Some(policies),
            ..Default::default()
        };
        Some(s)
    }
}
