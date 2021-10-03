use zysfs::types::devices::system::cpu::{Cpu, Policy};
use crate::cli::Cli;

pub fn policy(cli: &Cli, cpu_ids: &[u64]) -> Option<Cpu> {
    if !cli.has_cpu_args() { return None; }
    let mut policies = vec![];
    if let Some(cpu_on) = &cli.cpu_on {
        for cpu_id in cpu_ids {
            let policy = Policy { 
                id: Some(*cpu_id),
                cpu_online: Some(*cpu_on)
            };
            policies.push(policy);
        }
    }
    if let Some(cpu_on_each) = &cli.cpu_on_each {
        for (cpu_id, cpu_on) in cpu_on_each {
            if let Some(policy) = policies
                .iter_mut()
                .find(|p| Some(*cpu_id) == p.id)
            {
                policy.cpu_online = Some(*cpu_on);
            }
            else
            {
                let policy = Policy {
                    id: Some(*cpu_id),
                    cpu_online: Some(*cpu_on),
                };
                policies.push(policy);
            }
        }
    }
    if policies.is_empty() { None }
    else {
        policies.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        Some(Cpu { policies: Some(policies) })
    }
}
