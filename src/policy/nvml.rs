use nvml_facade as nvml;
use crate::cli::Cli;

#[derive(Clone, Debug)]
pub struct Nvml {
    ids: Option<Vec<u32>>,
    gpu_clock: Option<(u32, u32)>,
    gpu_clock_reset: bool,
    power_limit: Option<u32>,
}

impl Nvml {
    pub fn write(&self) {
        let ids = if let Some(ids) = self.ids.clone() { ids } else { return; };
        for id in ids {
            let device = if let Some(d) = nvml::Nvml::device_for_id(id) { d } else { continue; };
            if let Some((min, max)) = self.gpu_clock {
                device.clocks().set_gpu_locked_clocks(min, max);
            }
            if self.gpu_clock_reset {
                device.clocks().reset_gpu_locked_clocks();
            }
            if let Some(max) = self.power_limit {
                device.power().set_limit(max);
            }
        }
    }
}

impl From<&Cli> for Nvml {
    fn from(cli: &Cli) -> Self {
        let ids = cli.nvml();
        let gpu_clock = cli
            .nvml_gpu_clock
            .map(|(min, max)| (
                min.as_megahertz() as u32,
                max.as_megahertz() as u32,
            ));
        let gpu_clock_reset = cli.nvml_gpu_clock_reset.is_some();
        let power_limit = cli
            .nvml_power_limit
            .map(|p| p.as_milliwatts() as u32);
        Self {
            ids,
            gpu_clock,
            gpu_clock_reset,
            power_limit,
        }
    }
}

pub fn policy(cli: &Cli) -> Option<Nvml> {
    if cli.has_nvml_args() { Some(cli.into()) } else { None }
}
