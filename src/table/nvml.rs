use measurements::Frequency;
use nvml_facade::{Nvml, device::Clock as _};
use crate::table::{Table, dot};

fn hertz(mhz: u32) -> String {
    let f = Frequency::from_megahertz(mhz as f64);
    if mhz >= 1000 { format!("{:.1}", f) } else { format!("{:.0}", f) }
}

fn watts(mw: u32) -> String {
    mw.to_string()
}

fn bytes(b: u64) -> String {
    if b < 1000 { format!("{} B", b) }
    else if b < 1000u64.pow(2) { format!("{:.1} kB", b as f64/1000f64) }
    else if b < 1000u64.pow(3) { format!("{:.1} MB", b as f64/(1000u64.pow(2) as f64)) }
    else if b < 10000u64.pow(4) { format!("{:.1} GB", b as f64/(1000u64.pow(3) as f64)) }
    else { format!("{:.1} TB", b as f64/(1000u64.pow(4) as f64)) }
}

pub fn format() -> Option<String> {
    let nvml = Nvml::open()?;
    let ids = nvml.ids()?;
    if ids.is_empty() { return None; }
    let spam = regex::Regex::new("(?i)nvidia ").unwrap();
    let mut tab1 = Table::new(&["PCI ID          ", "Name             ", "Mem used", "Mem total", "Power usage", "Power max"]);
    let mut tab2 = Table::new(&["PCI ID          ", "Gfx cur / max    ", "Mem cur / max    ", "SM cur / max     ", "Video cur / max  "]);
    for id in ids {
        let dev = if let Some(v) = nvml.device_for_id(id) { v } else { continue; };
        let bus_id = dev.pci().bus_id().unwrap_or_else(dot);
        let mem = dev.memory();
        let pow = dev.power();
        let clocks = dev.clocks();
        tab1.row(&[
            bus_id.clone(),
            dev.hardware().name().map(|v| spam.replace_all(&v, "").into_owned()).unwrap_or_else(dot),
            mem.used().map(bytes).unwrap_or_else(dot),
            mem.total().map(bytes).unwrap_or_else(dot),
            pow.usage().map(watts).unwrap_or_else(dot),
            pow.constraints_min_max().map(|(_, v)| watts(v)).unwrap_or_else(dot),
        ]);
        tab2.row(&[
            bus_id,
            format!(
                "{} / {}",
                clocks.graphics().current().map(hertz).unwrap_or_else(dot),
                clocks.graphics().max().map(hertz).unwrap_or_else(dot),
            ),
            format!(
                "{} / {}",
                clocks.memory().current().map(hertz).unwrap_or_else(dot),
                clocks.memory().max().map(hertz).unwrap_or_else(dot),
            ),
            format!(
                "{} / {}",
                clocks.sm().current().map(hertz).unwrap_or_else(dot),
                clocks.sm().max().map(hertz).unwrap_or_else(dot),
            ),
            format!(
                "{} / {}",
                clocks.video().current().map(hertz).unwrap_or_else(dot),
                clocks.video().max().map(hertz).unwrap_or_else(dot),
            ),
        ]);
    }
    Some([tab1.to_string(), tab2.to_string()].join("\n"))
}
