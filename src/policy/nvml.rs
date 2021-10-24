use crate::cli::{CardId, Cli};

#[derive(Clone, Debug, Default)]
pub struct Policy {
    id: Option<u32>,
    gpu_clock: Option<(u32, u32)>,
    gpu_clock_reset: Option<()>,
    power_limit: Option<u32>,
}

impl Policy {
    pub fn write(&self) {
        let id = if let Some(id) = self.id { id } else { return; };
        let device = if let Some(d) = nvml_facade::Nvml::device_for_id(id) { d } else { return; };
        if let Some((min, max)) = self.gpu_clock {
            device.clocks().set_gpu_locked_clocks(min, max);
        }
        if self.gpu_clock_reset.is_some() {
            device.clocks().reset_gpu_locked_clocks();
        }
        if let Some(max) = self.power_limit {
            device.power().set_limit(max);
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Nvml {
    policies: Option<Vec<Policy>>,
}

impl Nvml {
    pub fn write(&self) {
        if let Some(pols) = &self.policies {
            for pol in pols {
                pol.write();
            }
        }
    }
}

fn card_ids(ids: Vec<CardId>) -> Option<Vec<u32>> {
    fn card_id(id: CardId) -> Option<u32> {
        match id {
            CardId::Index(id) => Some(id as u32),
            CardId::PciId(id) => Some(nvml_facade::Nvml::device_for_pci_id(&id)?.card().id()?),
        }
    }
    let mut indices = vec![];
    for id in ids {
        match card_id(id) {
            Some(id) => indices.push(id),
            _ => continue,
        }
    }
    if indices.is_empty() { None } else {
        indices.sort_unstable();
        indices.dedup();
        Some(indices)
    }
}

impl From<&Cli> for Option<Nvml> {
    fn from(cli: &Cli) -> Self {
        if !cli.has_nvml_args() { return None; }
        let ids = cli.nvml.clone()
            .and_then(card_ids)
            .or_else(nvml_facade::Nvml::ids)?;
        let gpu_clock = cli
            .nvml_gpu_freq
            .map(|(min, max)| (
                min.as_megahertz() as u32,
                max.as_megahertz() as u32,
            ));
        let power_limit = cli
            .nvml_power_limit
            .map(|p| p.as_milliwatts() as u32);
        let mut policies = vec![];
        for id in ids {
            let pol = Policy {
                id: Some(id),
                gpu_clock,
                gpu_clock_reset: cli.nvml_gpu_freq_reset,
                power_limit,
            };
            policies.push(pol);
        }
        let s = Nvml {
            policies: Some(policies),
        };
        Some(s)
    }
}
