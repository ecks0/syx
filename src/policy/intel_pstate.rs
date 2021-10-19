use crate::cli::Cli;
use zysfs::types::devices::system::cpu::intel_pstate::{IntelPstate, Policy};
use zysfs::types::blocking::Read as _;

impl From<&Cli> for Option<IntelPstate> {
    fn from(cli: &Cli) -> Self {
        if !cli.has_intel_pstate_args() { return None; }
        let policy_ids =
            if let Some(mut ids) = cli.cpu.clone() {
                ids.sort_unstable();
                ids.dedup();
                Some(ids)
            } else {
                Policy::ids()
            }?;
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
