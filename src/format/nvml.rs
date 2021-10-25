use nvml_facade::{
    Nvml,
    device::{Device, Clock as _},
};
use std::convert::TryInto;
use crate::format::{Table, dot, format_bytes, format_hz, format_uw};

fn mhz(mhz: u32) -> String { format_hz(mhz as u64 * 10u64.pow(6)) }

fn mw(mw: u32) -> String { format_uw(mw as u64 * 10u64.pow(3)) }

fn pad(s: &str, left: bool) -> String {
    const WIDTH: i64 = 19; // width of the widest label we expect to display
    let len: i64 = s.len().try_into().unwrap();
    let pad = (WIDTH-len).max(0) as usize;
    if left {
        format!("{}{}", " ".repeat(pad), s)
    } else {
        format!("{}{}", s, " ".repeat(pad))
    }
}

fn padl(s: &str) -> String { pad(s, true) }

fn padr(s: &str) -> String { pad(s, false) }

struct NvmlTable<'a> {
    tab: Table,
    devs: &'a [Device],
}

impl<'a> NvmlTable<'a> {
    fn new(devs: &'a [Device]) -> Option<Self> {
        let mut header = vec![padl("Nvidia GPU")];
        for dev in devs {
            let id = if let Some(id) = dev.card().id() { id } else { continue; };
            header.push(padr(&format!("{}", id)));
        }
        if header.len() == 1 { return None; }
        let header: Vec<&str> = header.iter().map(String::as_str).collect();
        let tab = Table::new(&header);
        let s = Self {
            tab,
            devs,
        };
        Some(s)
    }

    fn row<F>(&mut self, label: &str, mut f: F)
    where
        F: FnMut(&Device) -> String,
    {
        let mut row = vec![padl(label)];
        for dev in self.devs { row.push(f(dev)) }
        self.tab.row(&row);
    }

    fn into_table(self) -> Table {
        self.tab
    }
}

pub fn format() -> Option<String> {
    const DEVICES_PER_TABLE: usize = 2;
    let devices = Nvml::devices()?;
    let spam = regex::Regex::new("(?i)nvidia ").unwrap();
    let mut s = vec![];
    for devs in devices.chunks(DEVICES_PER_TABLE) {
        let mut tab = if let Some(tab) = NvmlTable::new(devs) { tab } else { continue; };
        tab.row("Name", |d|
            d.hardware().name().map(|v| spam.replace_all(&v, "").into_owned()).unwrap_or_else(dot)
        );
        tab.row("PCI ID", |d| d.pci().bus_id().unwrap_or_else(dot));
        tab.row("Graphics cur/max", |d| {
            format!(
                "{} / {}",
                d.clocks().graphics().current().map(mhz).unwrap_or_else(dot),
                d.clocks().graphics().max().map(mhz).unwrap_or_else(dot),
            )
        });
        tab.row("Memory cur/max", |d| {
            format!(
                "{} / {}",
                d.clocks().memory().current().map(mhz).unwrap_or_else(dot),
                d.clocks().memory().max().map(mhz).unwrap_or_else(dot),
            )
        });
        tab.row("SM cur/max", |d| {
            format!(
                "{} / {}",
                d.clocks().sm().current().map(mhz).unwrap_or_else(dot),
                d.clocks().sm().max().map(mhz).unwrap_or_else(dot),
            )
        });
        tab.row("Video cur/max", |d| {
            format!(
                "{} / {}",
                d.clocks().video().current().map(mhz).unwrap_or_else(dot),
                d.clocks().video().max().map(mhz).unwrap_or_else(dot),
            )
        });
        tab.row("Memory used/total", |d| {
            format!(
                "{} / {}",
                d.memory().used().map(format_bytes).unwrap_or_else(dot),
                d.memory().total().map(format_bytes).unwrap_or_else(dot),
            )
        });
        tab.row("Power used/limit", |d| {
            format!(
                "{} / {}",
                d.power().usage().map(mw).unwrap_or_else(dot),
                d.power().limit().map(mw).unwrap_or_else(dot),
            )
        });
        tab.row("Power limit min/max", |d| {
            format!(
                "{} / {}",
                d.power().min().map(mw).unwrap_or_else(dot),
                d.power().max().map(mw).unwrap_or_else(dot),
            )
        });
        s.push(tab.into_table().to_string())
    }
    if s.is_empty() { None } else { Some(s.join("\n")) }
}
