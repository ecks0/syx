use zysfs::types::class::drm::{Card, DriverPolicy, I915};
use crate::cli::Cli;

pub fn policy(cli: &Cli) -> Option<Vec<Card>> {
    if !cli.has_drm_i915_args() { return None; }
    let card_ids = cli.drm_i915()?;
    let driver_policy = DriverPolicy::I915(
        I915 {
            min_freq_mhz: cli.drm_i915_min.map(|f| f.as_megahertz() as u64),
            max_freq_mhz: cli.drm_i915_max.map(|f| f.as_megahertz() as u64),
            boost_freq_mhz: cli.drm_i915_boost.map(|f| f.as_megahertz() as u64),
            ..Default::default()
        }
    );
    let mut cards = vec![];
    for card_id in card_ids {
        let card = Card {
            id: Some(card_id),
            driver_policy: Some(driver_policy.clone()),
            ..Default::default()
        };
        cards.push(card);
    }
    Some(cards)
}
