use zysfs::types::class::drm::{Drm, Card, DriverPolicy, I915};
use crate::cli::Cli;

fn policy_i915(cli: &Cli) -> Option<Vec<Card>> {
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

pub fn policy(cli: &Cli) -> Option<Drm> {
    if !cli.has_drm_args() { return None; }
    let mut cards = vec![];
    if let Some(cards_i915) = policy_i915(cli) {
        cards.extend(cards_i915);
    }
    if cards.is_empty() { None } else { Some(Drm { cards: Some(cards) }) }
}
