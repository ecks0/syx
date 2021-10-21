use zysfs::types::devices::system::cpu::{Cpu, Policy};
use zysfs::types::std::Read as _;
use crate::cli::Cli;

impl From<&Cli> for Option<Cpu> {
    fn from(cli: &Cli) -> Self {
        if !cli.has_cpu_args() { return None; }
        let cpu_ids = if let Some(ids) = cli.cpu.clone() { ids } else { Policy::ids()? };
        let mut policies = vec![];
        if let Some(cpu_on) = cli.cpu_on {
            for cpu_id in cpu_ids {
                let policy = Policy {
                    id: Some(cpu_id),
                    cpu_online: Some(cpu_on)
                };
                policies.push(policy);
            }
        }
        if let Some(cpu_on_each) = cli.cpu_on_each.clone() {
            for (cpu_id, cpu_on) in cpu_on_each {
                if let Some(policy) = policies
                    .iter_mut()
                    .find(|p| Some(cpu_id) == p.id)
                {
                    policy.cpu_online = Some(cpu_on);
                }
                else
                {
                    let policy = Policy {
                        id: Some(cpu_id),
                        cpu_online: Some(cpu_on),
                    };
                    policies.push(policy);
                }
            }
        }
        policies.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        let s = Cpu {
            policies: Some(policies),
        };
        Some(s)
    }
}
