use crate::cli::Cli;
use zysfs::types::intel_rapl::{Constraint, IntelRapl, Policy, ZoneId};
use std::convert::TryInto;

impl From<&Cli> for Option<IntelRapl> {
    fn from(cli: &Cli) -> Self {
        if !cli.has_intel_rapl_args() { return None; }
        let s = IntelRapl {
            policies: Some([
                Policy {
                    id: Some(ZoneId { zone: cli.rapl_package?, subzone: cli.rapl_zone }),
                    constraints: Some([
                        Constraint {
                            id: Some(cli.rapl_constraint?),
                            power_limit_uw: cli.rapl_limit.map(|v| v.as_microwatts() as u64),
                            time_window_us: cli.rapl_window.map(|v| v.as_micros().try_into().unwrap()),
                            ..Default::default()
                        },
                    ].into()),
                    ..Default::default()
                },
            ].into()),
        };
        Some(s)
    }
}
