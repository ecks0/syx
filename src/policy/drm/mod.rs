use zysfs::types::class::drm::Drm;
use crate::cli::Cli;

mod i915;

pub fn policy(cli: &Cli) -> Option<Drm> {
    if !cli.has_drm_args() { return None; }
    let mut cards = vec![];
    if let Some(cards_i915) = i915::policy(cli) { cards.extend(cards_i915); }
    if cards.is_empty() { None } else { Some(Drm { cards: Some(cards) }) }
}
