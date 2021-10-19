use crate::cli::Cli;

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

impl From<&Cli> for Option<Nvml> {
    fn from(cli: &Cli) -> Self {
        if !cli.has_nvml_args() { return None; }
        let ids = if let Some(ids) = cli.nvml.clone() { ids } else { nvml_facade::Nvml::ids()? };
        let gpu_clock = cli
            .nvml_gpu_clock
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
                gpu_clock_reset: cli.nvml_gpu_clock_reset,
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
