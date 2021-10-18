use measurements::Frequency;
use zysfs::types::blocking::Read as _;
use zysfs::types::class::drm::{Card, DriverPolicy};
use crate::format::{Table, dot};

pub(crate) fn mhz(mhz: u64) -> String {
    let f = Frequency::from_megahertz(mhz as f64);
    if mhz >= 1000 { format!("{:.1}", f) } else { format!("{:.0}", f) }
}

pub fn format(id_driver: &[(u64, String)]) -> Option<String> {
    let cards: Vec<Card> = id_driver
        .iter()
        .filter_map(|(id, driver)| if "i915" == driver { Card::read(*id) } else { None })
        .collect();
    if cards.is_empty() { return None; }
    let mut tab = Table::new(&["Card", "Driver", "Actual", "Req'd", "Min", "Max", "Boost", "GPU min", "GPU max"]);
    for card in cards {
        if let Some(DriverPolicy::I915(policy)) = card.driver_policy {
            tab.row(&[
                card.id.map(|v| v.to_string()).unwrap_or_else(dot),
                card.driver.clone().unwrap_or_else(dot),
                policy.act_freq_mhz.map(mhz).unwrap_or_else(dot),
                policy.cur_freq_mhz.map(mhz).unwrap_or_else(dot),
                policy.min_freq_mhz.map(mhz).unwrap_or_else(dot),
                policy.max_freq_mhz.map(mhz).unwrap_or_else(dot),
                policy.boost_freq_mhz.map(mhz).unwrap_or_else(dot),
                policy.rpn_freq_mhz.map(mhz).unwrap_or_else(dot),
                policy.rp0_freq_mhz.map(mhz).unwrap_or_else(dot),
            ]);
        }
    }
    Some(tab.to_string())
}
