pub mod sampler;

pub use sampler::*;

pub mod path {
    use std::path::PathBuf;

    pub fn root() -> PathBuf {
        PathBuf::from("/sys/devices/virtual/powercap/intel-rapl")
    }

    pub fn zone(zone_id: u64) -> PathBuf {
        let mut p = root();
        p.push(&format!("intel-rapl:{}", zone_id));
        p
    }

    pub fn subzone(zone_id: u64, subzone_id: u64) -> PathBuf {
        let mut p = zone(zone_id);
        p.push(&format!("intel-rapl:{}:{}", zone_id, subzone_id));
        p
    }

    pub fn device(zone_id: u64, subzone_id: Option<u64>) -> PathBuf {
        match subzone_id {
            Some(subzone_id) => subzone(zone_id, subzone_id),
            None => zone(zone_id),
        }
    }

    pub fn device_attr(zone_id: u64, subzone_id: Option<u64>, a: &str) -> PathBuf {
        let mut p = device(zone_id, subzone_id);
        p.push(a);
        p
    }

    pub fn enabled(zone_id: u64, subzone_id: Option<u64>) -> PathBuf {
        device_attr(zone_id, subzone_id, "enabled")
    }

    pub fn energy_uj(zone_id: u64, subzone_id: Option<u64>) -> PathBuf {
        device_attr(zone_id, subzone_id, "energy_uj")
    }

    pub fn max_energy_range_uj(zone_id: u64, subzone_id: Option<u64>) -> PathBuf {
        device_attr(zone_id, subzone_id, "max_energy_range_uj")
    }

    pub fn name(zone_id: u64, subzone_id: Option<u64>) -> PathBuf {
        device_attr(zone_id, subzone_id, "name")
    }

    pub fn constraint_name(zone_id: u64, subzone_id: Option<u64>, constraint: u64) -> PathBuf {
        device_attr(
            zone_id,
            subzone_id,
            &format!("constraint_{}_name", constraint),
        )
    }

    pub fn constraint_max_power_uw(
        zone_id: u64,
        subzone_id: Option<u64>,
        constraint: u64,
    ) -> PathBuf {
        device_attr(
            zone_id,
            subzone_id,
            &format!("constraint_{}_max_power_uw", constraint),
        )
    }

    pub fn constraint_power_limit_uw(
        zone_id: u64,
        subzone_id: Option<u64>,
        constraint: u64,
    ) -> PathBuf {
        device_attr(
            zone_id,
            subzone_id,
            &format!("constraint_{}_power_limit_uw", constraint),
        )
    }

    pub fn constraint_time_window_us(
        zone_id: u64,
        subzone_id: Option<u64>,
        constraint: u64,
    ) -> PathBuf {
        device_attr(
            zone_id,
            subzone_id,
            &format!("constraint_{}_time_window_us", constraint),
        )
    }
}

use async_trait::async_trait;
use tokio::sync::OnceCell;

use crate::sysfs::{self, Result};
use crate::{Feature, Values};

pub async fn zones() -> Result<Vec<u64>> {
    sysfs::read_ids(&path::root(), "intel-rapl:").await
}

pub async fn subzones(zone_id: u64) -> Result<Vec<u64>> {
    sysfs::read_ids(&path::zone(zone_id), &format!("intel-rapl:{}:", zone_id)).await
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

pub async fn enabled(zone_id: u64, subzone_id: Option<u64>) -> Result<bool> {
    sysfs::read_bool(&path::enabled(zone_id, subzone_id)).await
}

pub async fn energy_uj(zone_id: u64, subzone_id: Option<u64>) -> Result<u64> {
    sysfs::read_u64(&path::energy_uj(zone_id, subzone_id)).await
}

pub async fn max_energy_range_uj(zone_id: u64, subzone_id: Option<u64>) -> Result<u64> {
    sysfs::read_u64(&path::max_energy_range_uj(zone_id, subzone_id)).await
}

pub async fn name(zone_id: u64, subzone_id: Option<u64>) -> Result<String> {
    sysfs::read_str(&path::name(zone_id, subzone_id)).await
}

pub async fn constraint_name(
    zone_id: u64,
    subzone_id: Option<u64>,
    constraint: u64,
) -> Result<String> {
    sysfs::read_str(&path::constraint_name(zone_id, subzone_id, constraint)).await
}

pub async fn constraint_max_power_uw(
    zone_id: u64,
    subzone_id: Option<u64>,
    constraint: u64,
) -> Result<u64> {
    sysfs::read_u64(&path::constraint_max_power_uw(
        zone_id, subzone_id, constraint,
    ))
    .await
}

pub async fn constraint_power_limit_uw(
    zone_id: u64,
    subzone_id: Option<u64>,
    constraint: u64,
) -> Result<u64> {
    sysfs::read_u64(&path::constraint_power_limit_uw(
        zone_id, subzone_id, constraint,
    ))
    .await
}

pub async fn constraint_time_window_us(
    zone_id: u64,
    subzone_id: Option<u64>,
    constraint: u64,
) -> Result<u64> {
    sysfs::read_u64(&path::constraint_time_window_us(
        zone_id, subzone_id, constraint,
    ))
    .await
}

pub async fn constraint_id_for_name(
    zone_id: u64,
    subzone_id: Option<u64>,
    name: &str,
) -> Option<u64> {
    for id in 0.. {
        if let Ok(n) = constraint_name(zone_id, subzone_id, id).await {
            if n == name {
                return Some(id);
            }
        }
    }
    None
}

pub async fn set_constraint_power_limit_uw(
    zone_id: u64,
    subzone_id: Option<u64>,
    constraint: u64,
    v: u64,
) -> Result<()> {
    sysfs::write_u64(
        &path::constraint_power_limit_uw(zone_id, subzone_id, constraint),
        v,
    )
    .await
}

pub async fn set_constraint_time_window_us(
    zone_id: u64,
    subzone_id: Option<u64>,
    constraint: u64,
    v: u64,
) -> Result<()> {
    sysfs::write_u64(
        &path::constraint_time_window_us(zone_id, subzone_id, constraint),
        v,
    )
    .await
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ZoneId {
    pub zone: u64,
    pub subzone: Option<u64>,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Constraint {
    pub id: u64,
    pub name: Option<String>,
    pub max_power_uw: Option<u64>,
    pub power_limit_uw: Option<u64>,
    pub time_window_us: Option<u64>,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Device {
    pub id: ZoneId,
    pub enabled: Option<bool>,
    pub energy_uj: Option<u64>,
    pub max_energy_range_uj: Option<u64>,
    pub name: Option<String>,
    pub constraints: Vec<Constraint>,
}

#[async_trait]
impl Values for Device {
    type Id = ZoneId;
    type Output = Self;

    async fn ids() -> Vec<ZoneId> {
        devices()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|(zone, subzone)| ZoneId { zone, subzone })
            .collect()
    }

    async fn read(id: ZoneId) -> Option<Self> {
        let enabled = enabled(id.zone, id.subzone).await.ok();
        let energy_uj = energy_uj(id.zone, id.subzone).await.ok();
        let max_energy_range_uj = max_energy_range_uj(id.zone, id.subzone).await.ok();
        let name = name(id.zone, id.subzone).await.ok();
        let constraints = {
            let mut constraints = vec![];
            for cid in 0.. {
                match constraint_name(id.zone, id.subzone, cid).await {
                    Ok(name) => {
                        let name = Some(name);
                        let max_power_uw =
                            constraint_max_power_uw(id.zone, id.subzone, cid).await.ok();
                        let power_limit_uw = constraint_power_limit_uw(id.zone, id.subzone, cid)
                            .await
                            .ok();
                        let time_window_us = constraint_time_window_us(id.zone, id.subzone, cid)
                            .await
                            .ok();
                        let id = cid;
                        let c = Constraint {
                            id,
                            name,
                            max_power_uw,
                            power_limit_uw,
                            time_window_us,
                        };
                        constraints.push(c);
                    },
                    Err(_) => break, // FIXME
                }
            }
            constraints
        };
        let s = Self {
            id,
            enabled,
            energy_uj,
            max_energy_range_uj,
            name,
            constraints,
        };
        Some(s)
    }

    async fn write(&self) {
        for c in &self.constraints {
            if let Some(v) = c.power_limit_uw {
                let _ = set_constraint_power_limit_uw(self.id.zone, self.id.subzone, c.id, v).await;
            };
            if let Some(v) = c.time_window_us {
                let _ = set_constraint_time_window_us(self.id.zone, self.id.subzone, c.id, v).await;
            };
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IntelRapl {
    pub devices: Vec<Device>,
}

#[async_trait]
impl Feature for IntelRapl {
    async fn present() -> bool {
        static PRESENT: OnceCell<bool> = OnceCell::const_new();
        async fn present() -> bool {
            path::root().is_dir()
        }
        *PRESENT.get_or_init(present).await
    }
}

#[async_trait]
impl Values for IntelRapl {
    type Id = ();
    type Output = Self;

    async fn ids() -> Vec<()> {
        vec![()]
    }

    async fn read(_: ()) -> Option<Self> {
        if !Self::present().await {
            return None;
        }
        let devices = Device::all().await;
        let s = Self { devices };
        Some(s)
    }

    async fn write(&self) {
        for device in &self.devices {
            device.write().await;
        }
    }
}
