pub(crate) mod path {
    use std::path::PathBuf;

    pub(crate) fn root() -> PathBuf {
        PathBuf::from("/sys/devices/virtual/powercap/intel-rapl")
    }

    pub(crate) fn zone(package: u64) -> PathBuf {
        let mut p = root();
        p.push(&format!("intel-rapl:{}", package));
        p
    }

    pub(crate) fn subzone(package: u64, subzone: u64) -> PathBuf {
        let mut p = zone(package);
        p.push(&format!("intel-rapl:{}:{}", package, subzone));
        p
    }

    pub(crate) fn device(package: u64, subzone_: Option<u64>) -> PathBuf {
        match subzone_ {
            Some(subzone_) => subzone(package, subzone_),
            None => zone(package),
        }
    }

    pub(crate) fn device_attr(package: u64, subzone: Option<u64>, a: &str) -> PathBuf {
        let mut p = device(package, subzone);
        p.push(a);
        p
    }

    pub(crate) fn enabled(package: u64, subzone: Option<u64>) -> PathBuf {
        device_attr(package, subzone, "enabled")
    }

    pub(crate) fn energy_uj(package: u64, subzone: Option<u64>) -> PathBuf {
        device_attr(package, subzone, "energy_uj")
    }

    pub(crate) fn max_energy_range_uj(package: u64, subzone: Option<u64>) -> PathBuf {
        device_attr(package, subzone, "max_energy_range_uj")
    }

    pub(crate) fn name(package: u64, subzone: Option<u64>) -> PathBuf {
        device_attr(package, subzone, "name")
    }

    pub(crate) fn constraint_attr(
        package: u64,
        subzone: Option<u64>,
        index: u64,
        a: &str,
    ) -> PathBuf {
        device_attr(package, subzone, &format!("constraint_{}_{}", index, a))
    }

    pub(crate) fn constraint_name(package: u64, subzone: Option<u64>, index: u64) -> PathBuf {
        constraint_attr(package, subzone, index, "name")
    }

    pub(crate) fn constraint_max_power_uw(
        package: u64,
        subzone: Option<u64>,
        index: u64,
    ) -> PathBuf {
        constraint_attr(package, subzone, index, "max_power_uw")
    }

    pub(crate) fn constraint_power_limit_uw(
        package: u64,
        subzone: Option<u64>,
        index: u64,
    ) -> PathBuf {
        constraint_attr(package, subzone, index, "power_limit_uw")
    }

    pub(crate) fn constraint_time_window_us(
        package: u64,
        subzone: Option<u64>,
        index: u64,
    ) -> PathBuf {
        constraint_attr(package, subzone, index, "time_window_us")
    }
}

use crate::{sysfs, Cached, Result};

pub async fn available() -> bool {
    path::root().is_dir()
}

pub async fn zones() -> Result<Vec<u64>> {
    sysfs::read_ids(&path::root(), "intel-rapl:").await
}

pub async fn subzones(package: u64) -> Result<Vec<u64>> {
    sysfs::read_ids(&path::zone(package), &format!("intel-rapl:{}:", package)).await
}

pub async fn devices() -> Result<Vec<(u64, Option<u64>)>> {
    let mut devices = vec![];
    for zone in zones().await.unwrap_or_default() {
        devices.push((zone, None));
        if let Ok(subzones) = subzones(zone).await {
            for subzone in subzones {
                devices.push((zone, Some(subzone)));
            }
        };
    }
    Ok(devices)
}

pub async fn enabled(package: u64, subzone: Option<u64>) -> Result<bool> {
    sysfs::read_bool(&path::enabled(package, subzone)).await
}

pub async fn energy_uj(package: u64, subzone: Option<u64>) -> Result<u64> {
    sysfs::read_u64(&path::energy_uj(package, subzone)).await
}

pub async fn max_energy_range_uj(package: u64, subzone: Option<u64>) -> Result<u64> {
    sysfs::read_u64(&path::max_energy_range_uj(package, subzone)).await
}

pub async fn name(package: u64, subzone: Option<u64>) -> Result<String> {
    sysfs::read_string(&path::name(package, subzone)).await
}

pub async fn set_enabled(package: u64, subzone: Option<u64>, v: bool) -> Result<()> {
    sysfs::write_bool(&path::enabled(package, subzone), v).await
}

pub async fn constraints(package: u64, subzone: Option<u64>) -> Result<Vec<u64>> {
    let mut indices = vec![];
    for i in 0.. {
        if path::constraint_name(package, subzone, i).is_file() {
            indices.push(i);
        } else {
            break;
        }
    }
    Ok(indices)
}

pub async fn constraint_name(
    package: u64,
    subzone: Option<u64>,
    constraint: u64,
) -> Result<String> {
    sysfs::read_string(&path::constraint_name(package, subzone, constraint)).await
}

pub async fn constraint_max_power_uw(
    package: u64,
    subzone: Option<u64>,
    constraint: u64,
) -> Result<u64> {
    sysfs::read_u64(&path::constraint_max_power_uw(package, subzone, constraint)).await
}

pub async fn constraint_power_limit_uw(
    package: u64,
    subzone: Option<u64>,
    constraint: u64,
) -> Result<u64> {
    sysfs::read_u64(&path::constraint_power_limit_uw(
        package, subzone, constraint,
    ))
    .await
}

pub async fn constraint_time_window_us(
    package: u64,
    subzone: Option<u64>,
    constraint: u64,
) -> Result<u64> {
    sysfs::read_u64(&path::constraint_time_window_us(
        package, subzone, constraint,
    ))
    .await
}

pub async fn constraint_id_for_name(package: u64, subzone: Option<u64>, name: &str) -> Option<u64> {
    for id in 0.. {
        match constraint_name(package, subzone, id).await {
            Ok(n) => {
                if n == name {
                    return Some(id);
                }
            },
            _ => break,
        }
    }
    None
}

pub async fn set_constraint_power_limit_uw(
    package: u64,
    subzone: Option<u64>,
    constraint: u64,
    v: u64,
) -> Result<()> {
    sysfs::write_u64(
        &path::constraint_power_limit_uw(package, subzone, constraint),
        v,
    )
    .await
}

pub async fn set_constraint_time_window_us(
    package: u64,
    subzone: Option<u64>,
    constraint: u64,
    v: u64,
) -> Result<()> {
    sysfs::write_u64(
        &path::constraint_time_window_us(package, subzone, constraint),
        v,
    )
    .await
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ZoneId {
    package: u64,
    subzone: Option<u64>,
}

impl ZoneId {
    pub fn new(package: u64, subzone: Option<u64>) -> Self {
        Self { package, subzone }
    }

    pub fn package(&self) -> u64 {
        self.package
    }

    pub fn subzone(&self) -> Option<u64> {
        self.subzone
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConstraintId {
    zone: ZoneId,
    index: u64,
}

impl ConstraintId {
    pub fn new(zone: ZoneId, index: u64) -> Self {
        Self { zone, index }
    }

    pub fn zone(&self) -> ZoneId {
        self.zone
    }

    pub fn index(&self) -> u64 {
        self.index
    }

    fn decompose(&self) -> (u64, Option<u64>, u64) {
        (self.zone.package(), self.zone.subzone(), self.index)
    }
}

#[derive(Clone, Debug)]
pub struct Constraint {
    id: ConstraintId,
    name: Cached<String>,
    max_power_uw: Cached<u64>,
    power_limit_uw: Cached<u64>,
    time_window_us: Cached<u64>,
}

impl Constraint {
    pub async fn available() -> bool {
        available().await
    }

    pub async fn ids(zone: ZoneId) -> Result<Vec<ConstraintId>> {
        constraints(zone.package, zone.subzone).await.map(|ids| {
            ids.into_iter()
                .map(|index| ConstraintId::new(zone, index))
                .collect()
        })
    }

    pub fn new(id: ConstraintId) -> Self {
        let name = Cached::default();
        let max_power_uw = Cached::default();
        let power_limit_uw = Cached::default();
        let time_window_us = Cached::default();
        Self {
            id,
            name,
            max_power_uw,
            power_limit_uw,
            time_window_us,
        }
    }

    pub async fn clear(&self) {
        tokio::join!(
            self.name.clear(),
            self.max_power_uw.clear(),
            self.power_limit_uw.clear(),
            self.time_window_us.clear(),
        );
    }

    pub fn id(&self) -> ConstraintId {
        self.id
    }

    pub async fn name(&self) -> Result<String> {
        let (package, subzone, index) = self.id.decompose();
        self.name
            .get_or(constraint_name(package, subzone, index))
            .await
    }

    pub async fn max_power_uw(&self) -> Result<u64> {
        let (package, subzone, index) = self.id.decompose();
        self.max_power_uw
            .get_or(constraint_max_power_uw(package, subzone, index))
            .await
    }

    pub async fn power_limit_uw(&self) -> Result<u64> {
        let (package, subzone, index) = self.id.decompose();
        self.power_limit_uw
            .get_or(constraint_power_limit_uw(package, subzone, index))
            .await
    }

    pub async fn time_window_us(&self) -> Result<u64> {
        let (package, subzone, index) = self.id.decompose();
        self.time_window_us
            .get_or(constraint_time_window_us(package, subzone, index))
            .await
    }

    pub async fn set_power_limit_uw(&mut self, v: u64) -> Result<()> {
        let (package, subzone, index) = self.id.decompose();
        let f = set_constraint_power_limit_uw(package, subzone, index, v);
        self.power_limit_uw.clear_if(f).await
    }

    pub async fn set_time_window_us(&mut self, v: u64) -> Result<()> {
        let (package, subzone, index) = self.id.decompose();
        let f = set_constraint_time_window_us(package, subzone, index, v);
        self.time_window_us.clear_if(f).await
    }
}

#[derive(Clone, Debug)]
pub struct Zone {
    id: ZoneId,
    enabled: Cached<bool>,
    energy_uj: Cached<u64>,
    max_energy_range_uj: Cached<u64>,
    name: Cached<String>,
}

impl Zone {
    pub async fn available() -> bool {
        available().await
    }

    pub async fn ids() -> Result<Vec<ZoneId>> {
        devices().await.map(|ids| {
            ids.into_iter()
                .map(|(package, subzone)| ZoneId::new(package, subzone))
                .collect()
        })
    }

    pub fn new(id: ZoneId) -> Self {
        let enabled = Cached::default();
        let energy_uj = Cached::default();
        let max_energy_range_uj = Cached::default();
        let name = Cached::default();
        Self {
            id,
            enabled,
            energy_uj,
            max_energy_range_uj,
            name,
        }
    }

    pub async fn clear(&self) {
        tokio::join!(
            self.enabled.clear(),
            self.energy_uj.clear(),
            self.max_energy_range_uj.clear(),
            self.name.clear(),
        );
    }

    pub fn id(&self) -> ZoneId {
        self.id
    }

    pub async fn enabled(&self) -> Result<bool> {
        self.enabled
            .get_or(enabled(self.id.package, self.id.subzone))
            .await
    }

    pub async fn energy_uj(&self) -> Result<u64> {
        self.energy_uj
            .get_or(energy_uj(self.id.package, self.id.subzone))
            .await
    }

    pub async fn max_energy_range_uj(&self) -> Result<u64> {
        self.max_energy_range_uj
            .get_or(max_energy_range_uj(self.id.package, self.id.subzone))
            .await
    }

    pub async fn name(&self) -> Result<String> {
        self.name
            .get_or(name(self.id.package, self.id.subzone))
            .await
    }

    pub async fn set_enabled(&self, v: bool) -> Result<()> {
        self.enabled
            .clear_if(set_enabled(self.id.package, self.id.subzone, v))
            .await
    }
}
