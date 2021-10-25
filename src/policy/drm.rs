use zysfs::io::drm::std::driver as drm_driver;
use zysfs::types::drm::{Card, DriverPolicy, Drm, I915};
use zysfs::types::std::Read as _;
use crate::cli::{CardId, Cli};

fn card_ids(ids: Vec<CardId>) -> Option<Vec<u64>> {
    fn card_id(id: CardId) -> Option<u64> {
        match id {
            CardId::Index(id) => Some(id),
            CardId::PciId(_) => panic!("Refrencing i915 cards by pci id is not yet supported"),
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

fn ids_for_driver(
    driver: &str,
    cli_arg: &Option<Vec<CardId>>,
    id_driver: &[(u64, Option<String>)]
) -> Option<Vec<u64>>
{
    if let Some(ids) = cli_arg.clone() { card_ids(ids) }
    else {
        let r = id_driver
            .iter()
            .filter_map(|(id, d)|
                match d {
                    None => None,
                    Some(d) =>
                        if d == driver { Some(*id) } else { None },
                }
            )
            .collect();
        Some(r)
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
            if let Some(card_ids) = ids_for_driver("i915", &cli.drm_i915, &id_driver) {
                let driver_policy = DriverPolicy::I915(
                    I915 {
                        min_freq_mhz: cli.drm_i915_min.map(|f| f.as_megahertz() as u64),
                        max_freq_mhz: cli.drm_i915_max.map(|f| f.as_megahertz() as u64),
                        boost_freq_mhz: cli.drm_i915_boost.map(|f| f.as_megahertz() as u64),
                        ..Default::default()
                    }
                );
                for card_id in card_ids {
                    let card = Card {
                        id: Some(card_id),
                        driver_policy: Some(driver_policy.clone()),
                        ..Default::default()
                    };
                    cards.push(card);
                }
            }
        }
        if cards.is_empty() { return None; }
        let s = Drm {
            cards: Some(cards),
        };
        Some(s)
    }
}
