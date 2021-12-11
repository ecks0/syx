pub mod sampler;

pub use sampler::*;

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
        constraint: u64,
        a: &str,
    ) -> PathBuf {
        device_attr(
            package,
            subzone,
            &format!("constraint_{}_{}", constraint, a),
        )
    }

    pub(crate) fn constraint_name(package: u64, subzone: Option<u64>, constraint: u64) -> PathBuf {
        constraint_attr(package, subzone, constraint, "name")
    }

    pub(crate) fn constraint_max_power_uw(
        package: u64,
        subzone: Option<u64>,
        constraint: u64,
    ) -> PathBuf {
        constraint_attr(package, subzone, constraint, "max_power_uw")
    }

    pub(crate) fn constraint_power_limit_uw(
        package: u64,
        subzone: Option<u64>,
        constraint: u64,
    ) -> PathBuf {
        constraint_attr(package, subzone, constraint, "power_limit_uw")
    }

    pub(crate) fn constraint_time_window_us(
        package: u64,
        subzone: Option<u64>,
        constraint: u64,
    ) -> PathBuf {
        constraint_attr(package, subzone, constraint, "time_window_us")
    }
}

use async_trait::async_trait;

use crate::util::sysfs::{self, Result};
use crate::{Feature, Multi, Read, Single, Values, Write};

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
    sysfs::read_str(&path::name(package, subzone)).await
}

pub async fn set_enabled(package: u64, subzone: Option<u64>, v: bool) -> Result<()> {
    sysfs::write_bool(&path::enabled(package, subzone), v).await
}

pub async fn constraints(package: u64, subzone: Option<u64>) -> Vec<u64> {
    let mut indices = vec![];
    for i in 0.. {
        if path::constraint_name(package, subzone, i).is_file() {
            indices.push(i);
        } else {
            break;
        }
    }
    indices
}

pub async fn constraint_name(
    package: u64,
    subzone: Option<u64>,
    constraint: u64,
) -> Result<String> {
    sysfs::read_str(&path::constraint_name(package, subzone, constraint)).await
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

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Constraint {
    id: ConstraintId,
    name: Option<String>,
    max_power_uw: Option<u64>,
    power_limit_uw: Option<u64>,
    time_window_us: Option<u64>,
}

impl Constraint {
    pub async fn ids_for_zone(zone: ZoneId) -> Vec<ConstraintId> {
        let mut ids = vec![];
        for index in constraints(zone.package, zone.subzone).await {
            let id = ConstraintId::new(zone, index);
            ids.push(id);
        }
        ids
    }

    pub async fn load_for_zone(zone: ZoneId) -> Vec<Constraint> {
        let mut constraints = vec![];
        for id in Self::ids_for_zone(zone).await {
            let s = Self::new(id);
            constraints.push(s);
        }
        constraints
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn max_power_uw(&self) -> Option<u64> {
        self.max_power_uw
    }

    pub fn power_limit_uw(&self) -> Option<u64> {
        self.power_limit_uw
    }

    pub fn time_window_us(&self) -> Option<u64> {
        self.time_window_us
    }

    pub fn set_power_limit_uw(&mut self, v: impl Into<Option<u64>>) -> &mut Self {
        self.power_limit_uw = v.into();
        self
    }

    pub fn set_time_window_us(&mut self, v: impl Into<Option<u64>>) -> &mut Self {
        self.time_window_us = v.into();
        self
    }
}

#[async_trait]
impl Read for Constraint {
    async fn read(&mut self) {
        let (package, subzone, index) = (self.id.zone.package, self.id.zone.subzone, self.id.index);
        self.name = constraint_name(package, subzone, index).await.ok();
        self.max_power_uw = constraint_max_power_uw(package, subzone, index).await.ok();
        self.power_limit_uw = constraint_power_limit_uw(package, subzone, index)
            .await
            .ok();
        self.time_window_us = constraint_time_window_us(package, subzone, index)
            .await
            .ok();
    }
}

#[async_trait]
impl Write for Constraint {
    async fn write(&self) {
        let (package, subzone, index) = (self.id.zone.package, self.id.zone.subzone, self.id.index);
        if let Some(v) = self.power_limit_uw {
            let _ = set_constraint_power_limit_uw(package, subzone, index, v).await;
        };
        if let Some(v) = self.time_window_us {
            let _ = set_constraint_time_window_us(package, subzone, index, v).await;
        };
    }
}

#[async_trait]
impl Values for Constraint {
    fn is_empty(&self) -> bool {
        self.eq(&Self::new(self.id))
    }

    fn clear(&mut self) -> &mut Self {
        *self = Self::new(self.id);
        self
    }
}

#[async_trait]
impl Multi for Constraint {
    type Id = ConstraintId;

    async fn ids() -> Vec<Self::Id> {
        let mut ids = vec![];
        for (package, subzone) in devices().await.unwrap_or_default() {
            let zone = ZoneId::new(package, subzone);
            ids.extend(Self::ids_for_zone(zone).await);
        }
        ids
    }

    fn id(&self) -> Self::Id {
        self.id
    }

    fn set_id(&mut self, v: Self::Id) -> &mut Self {
        self.id = v;
        self
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Device {
    id: ZoneId,
    constraints: Vec<Constraint>,
    enabled: Option<bool>,
    energy_uj: Option<u64>,
    max_energy_range_uj: Option<u64>,
    name: Option<String>,
}

impl Device {
    pub fn push_constraint(&mut self, v: Constraint) -> &mut Self {
        if let Some(i) = self.constraints.iter().position(|c| v.id.eq(&c.id)) {
            self.constraints[i] = v;
        } else {
            self.constraints.push(v);
            self.constraints.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        }
        self
    }

    pub fn push_constraints<I>(&mut self, v: impl IntoIterator<Item = Constraint>) -> &mut Self {
        for c in v.into_iter() {
            self.push_constraint(c);
        }
        self
    }

    pub fn constraints(&self) -> std::slice::Iter<'_, Constraint> {
        self.constraints.iter()
    }

    pub fn into_constraints(self) -> impl IntoIterator<Item = Constraint> {
        self.constraints.into_iter()
    }

    pub fn enabled(&self) -> Option<bool> {
        self.enabled
    }

    pub fn energy_uj(&self) -> Option<u64> {
        self.energy_uj
    }

    pub fn max_energy_range_uj(&self) -> Option<u64> {
        self.max_energy_range_uj
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn set_enabled(&mut self, v: impl Into<Option<bool>>) -> &mut Self {
        self.enabled = v.into();
        self
    }
}

#[async_trait]
impl Read for Device {
    async fn read(&mut self) {
        self.constraints.clear();
        self.constraints
            .extend(Constraint::load_for_zone(self.id).await);
        self.enabled = enabled(self.id.package, self.id.subzone).await.ok();
        self.energy_uj = energy_uj(self.id.package, self.id.subzone).await.ok();
        self.max_energy_range_uj = max_energy_range_uj(self.id.package, self.id.subzone)
            .await
            .ok();
        self.name = name(self.id.package, self.id.subzone).await.ok();
    }
}

#[async_trait]
impl Write for Device {
    async fn write(&self) {
        for constraint in &self.constraints {
            constraint.write().await;
        }
        if let Some(v) = self.enabled {
            let _ = set_enabled(self.id.package, self.id.subzone, v);
        }
    }
}

#[async_trait]
impl Values for Device {
    fn is_empty(&self) -> bool {
        self.eq(&Self::new(self.id))
    }

    fn clear(&mut self) -> &mut Self {
        *self = Self::new(self.id);
        self
    }
}

#[async_trait]
impl Multi for Device {
    type Id = ZoneId;

    async fn ids() -> Vec<ZoneId> {
        devices()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|(package, subzone)| ZoneId::new(package, subzone))
            .collect()
    }

    fn id(&self) -> Self::Id {
        self.id
    }

    fn set_id(&mut self, v: Self::Id) -> &mut Self {
        self.id = v;
        self
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct System {
    devices: Vec<Device>,
}

#[async_trait]
impl Read for System {
    async fn read(&mut self) {
        self.devices.clear();
        self.devices.extend(Device::load_all().await);
    }
}

#[async_trait]
impl Write for System {
    async fn write(&self) {
        for device in &self.devices {
            device.write().await;
        }
    }
}

#[async_trait]
impl Values for System {
    fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }

    fn clear(&mut self) -> &mut Self {
        self.devices.clear();
        self
    }
}

impl Single for System {}

#[async_trait]
impl Feature for System {
    async fn present() -> bool {
        path::root().is_dir()
    }
}
