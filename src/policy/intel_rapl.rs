use crate::cli::Cli;
use zysfs::types::intel_rapl::{Constraint, IntelRapl, Policy, ZoneId};
use std::convert::TryInto;

impl From<&Cli> for Option<IntelRapl> {
    fn from(cli: &Cli) -> Self {
        if !cli.has_rapl_args() { return None; }
        let id = ZoneId { zone: cli.rapl_package?, subzone: cli.rapl_zone };
        let constraints: Vec<Constraint> =
            [
                (0, cli.rapl_c0_limit, cli.rapl_c0_window),
                (1, cli.rapl_c1_limit, cli.rapl_c1_window),
            ]
                .iter()
                .filter_map(|(id, limit, window)|
                    if limit.is_some() || window.is_some() {
                        let c = Constraint {
                            id: Some(*id),
                            power_limit_uw: limit.map(|v| v.as_microwatts() as u64),
                            time_window_us: window.map(|v| v.as_micros().try_into().unwrap()),
                            ..Default::default()
                        };
                        Some(c)
                    } else {
                        None
                    }
                )
                .collect();
        let s = IntelRapl {
            policies: Some(vec![
                Policy {
                    id: Some(id),
                    constraints: Some(constraints),
                    ..Default::default()
                },
            ]),
        };
        Some(s)
    }
}
