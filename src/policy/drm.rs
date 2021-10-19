use zysfs::io::class::drm::blocking::driver as drm_driver;
use zysfs::types::class::drm::{Card, DriverPolicy, Drm, I915};
use zysfs::types::blocking::Read as _;
use crate::cli::Cli;

fn ids_for_driver(
    driver: &str,
    cli_arg: &Option<Vec<u64>>,
    id_driver: &[(u64, Option<String>)]
) -> Vec<u64>
{
    if let Some(ids) = cli_arg.clone() { ids }
    else {
        id_driver
            .iter()
            .filter_map(|(id, d)|
                match d {
                    None => None,
                    Some(d) =>
                        if d == driver { Some(*id) } else { None },
                }
            )
            .collect()
    }
}

impl From<&Cli> for Option<Drm> {
    fn from(cli: &Cli) -> Self {
        if !cli.has_drm_args() { return None; }
        let id_driver: Vec<(u64, Option<String>)> = Card::ids()?
            .into_iter()
            .map(|i| (i, drm_driver(i).ok()))
            .collect();
        let mut cards = vec![];
        if cli.has_drm_i915_args() {
            let driver_policy = DriverPolicy::I915(
                I915 {
                    min_freq_mhz: cli.drm_i915_min.map(|f| f.as_megahertz() as u64),
                    max_freq_mhz: cli.drm_i915_max.map(|f| f.as_megahertz() as u64),
                    boost_freq_mhz: cli.drm_i915_boost.map(|f| f.as_megahertz() as u64),
                    ..Default::default()
                }
            );
            for card_id in ids_for_driver("i915", &cli.drm_i915, &id_driver) {
                let card = Card {
                    id: Some(card_id),
                    driver_policy: Some(driver_policy.clone()),
                    ..Default::default()
                };
                cards.push(card);
            };
        }
        if cards.is_empty() { return None; }
        let s = Drm {
            cards: Some(cards),
        };
        Some(s)
    }
}
