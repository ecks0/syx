use zysfs::io::class::drm::std::driver;
use zysfs::types::std::Read as _;
use zysfs::types::class::drm::Card;

mod i915;

pub fn format() -> Option<String> {
    let card_ids = Card::ids()?;
    let id_driver: Vec<(u64, String)> = card_ids
        .into_iter()
        .filter_map(|id| driver(id).ok().map(|d| (id, d)))
        .collect();
    let mut s = vec![];
    if let Some(ss) = i915::format(&id_driver) { s.push(ss); }
    if s.is_empty() { None } else { Some(s.join("\n")) }
}