use once_cell::sync::Lazy;
use zysfs::io::devices::system::cpu::std::{cpu_online, set_cpu_online};
use zysfs::types::devices::system::cpu::{Cpu, Policy};
use crate::cli::Cli;

static CPU_IDS: Lazy<Option<Vec<u64>>> = Lazy::new(|| {
    zysfs::io::devices::system::cpu::std::cpus().ok()
});

fn cpu_ids() -> Option<Vec<u64>> { CPU_IDS.clone() }

pub(super) fn set_all_cpus_online() -> Option<Vec<(u64, Option<bool>)>> {
    if let Some(cpu_ids) = cpu_ids() {
        let cpu_id_online: Vec<(u64, Option<bool>)> = cpu_ids
            .into_iter()
            .map(|id| (id, cpu_online(id).ok()))
            .collect();
        for (id, online) in cpu_id_online.clone() {
            if let Some(v) = online {
                if !v { let _ = set_cpu_online(id, true); }
            }
        }
        Some(cpu_id_online)
    } else {
        None
    }
}

pub(super) fn set_cpus_online(cpu_id_online: Vec<(u64, Option<bool>)>) {
    for (id, online) in cpu_id_online {
        if let Some(v) = online {
            if !v { let _ = set_cpu_online(id, false); }
        }
    }
}

impl From<&Cli> for Option<Cpu> {
    fn from(cli: &Cli) -> Self {
        if !cli.has_cpu_args() { return None; }
        let cpu_ids = cli.cpu.clone().or_else(cpu_ids)?;
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
