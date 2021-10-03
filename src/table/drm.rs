use measurements::Frequency;
use tabular::{Table, Row};
use zysfs::io::class::drm::blocking::driver;
use zysfs::types::blocking::Read as _;
use zysfs::types::class::drm::{Card, DriverPolicy};
use super::dot;

fn mhz(mhz: u64) -> String {
    let f = Frequency::from_megahertz(mhz as f64);
    if mhz >= 1000 { format!("{:.1}", f) } else { format!("{:.0}", f) }
}

fn format_i915(id_driver: &[(u64, String)]) -> Option<String> {
    let cards: Vec<Card> = id_driver
        .iter()
        .filter_map(|(id, driver)| if "i915" == driver { Card::read(*id) } else { None })
        .collect();
    if cards.is_empty() { return None; }
    let mut tab = Table::new("{:<} {:<} {:<} {:<} {:<} {:<} {:<} {:<} {:<}");
    let mut row = |a: &str, b: &str, c: &str, d: &str, e: &str, f: &str, g: &str, h: &str, i: &str| {
        tab.add_row(Row::new()
            .with_cell(a).with_cell(b).with_cell(c).with_cell(d).with_cell(e)
            .with_cell(f).with_cell(g).with_cell(h).with_cell(i));
    };
    row("Card", "Driver", "Actual", "Req'd", "Min", "Max", "Boost", "Min limit", "Max limit");
    row("-----", "-------", "----------", "----------", "----------", "----------", "----------", "----------", "----------");
    for card in cards {
        if let Some(DriverPolicy::I915(policy)) = card.driver_policy {
            row(
                &card.id.map(|v| v.to_string()).unwrap_or_else(dot),
                &card.driver.clone().unwrap_or_else(dot),
                &policy.act_freq_mhz.map(mhz).unwrap_or_else(dot),
                &policy.cur_freq_mhz.map(mhz).unwrap_or_else(dot),
                &policy.min_freq_mhz.map(mhz).unwrap_or_else(dot),
                &policy.max_freq_mhz.map(mhz).unwrap_or_else(dot),
                &policy.boost_freq_mhz.map(mhz).unwrap_or_else(dot),
                &policy.rpn_freq_mhz.map(mhz).unwrap_or_else(dot),
                &policy.rp0_freq_mhz.map(mhz).unwrap_or_else(dot),
            );
        }
    }
    Some(tab.to_string())
}

fn format() -> Option<String> { 
    let card_ids = Card::ids()?;
    let id_driver: Vec<(u64, String)> = card_ids
        .into_iter()
        .filter_map(|id| driver(id).ok().map(|d| (id, d)))
        .collect();
    if id_driver.is_empty() { return None; }
    let mut s = vec![];
    if let Some(ss) = format_i915(&id_driver) {
        s.push(ss);
    }
    if s.is_empty() { None } else { Some(s.join("\n")) }
}

pub fn print() {
    if let Some(s) = format() {
        println!("{}", s);
    }
}
