pub mod path {
    use std::path::PathBuf;

    pub fn root() -> PathBuf {
        PathBuf::from("/sys/class/drm")
    }

    pub fn device(id: u64) -> PathBuf {
        let mut p = root();
        p.push(format!("card{}", id));
        p
    }

    pub fn device_attr(id: u64, s: &str) -> PathBuf {
        let mut p = device(id);
        p.push(s);
        p
    }

    pub fn driver(id: u64) -> PathBuf {
        let mut p = device_attr(id, "device");
        p.push("driver");
        p
    }

    pub fn act_freq_mhz(id: u64) -> PathBuf {
        device_attr(id, "gt_act_freq_mhz")
    }

    pub fn boost_freq_mhz(id: u64) -> PathBuf {
        device_attr(id, "gt_boost_freq_mhz")
    }

    pub fn cur_freq_mhz(id: u64) -> PathBuf {
        device_attr(id, "gt_cur_freq_mhz")
    }

    pub fn max_freq_mhz(id: u64) -> PathBuf {
        device_attr(id, "gt_max_freq_mhz")
    }

    pub fn min_freq_mhz(id: u64) -> PathBuf {
        device_attr(id, "gt_min_freq_mhz")
    }

    pub fn rp0_freq_mhz(id: u64) -> PathBuf {
        device_attr(id, "gt_RP0_freq_mhz")
    }

    pub fn rp1_freq_mhz(id: u64) -> PathBuf {
        device_attr(id, "gt_RP1_freq_mhz")
    }

    pub fn rpn_freq_mhz(id: u64) -> PathBuf {
        device_attr(id, "gt_RPn_freq_mhz")
    }
}

use async_trait::async_trait;

use crate::sysfs::{self, Result};
use crate::{Feature, Values};

pub async fn devices() -> Result<Vec<u64>> {
    let mut ids = vec![];
    for id in sysfs::read_ids(&path::root(), "card").await? {
        if let Ok(driver) = driver(id).await {
            if "i915" == driver.as_str() {
                ids.push(id);
            }
        }
    }
    Ok(ids)
}

pub async fn driver(id: u64) -> Result<String> {
    sysfs::read_link_name(&path::driver(id)).await
}

pub async fn act_freq_mhz(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::act_freq_mhz(id)).await
}

pub async fn boost_freq_mhz(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::boost_freq_mhz(id)).await
}

pub async fn cur_freq_mhz(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::cur_freq_mhz(id)).await
}

pub async fn max_freq_mhz(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::max_freq_mhz(id)).await
}

pub async fn min_freq_mhz(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::min_freq_mhz(id)).await
}

pub async fn rp0_freq_mhz(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::rp0_freq_mhz(id)).await
}

pub async fn rp1_freq_mhz(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::rp1_freq_mhz(id)).await
}

pub async fn rpn_freq_mhz(id: u64) -> Result<u64> {
    sysfs::read_u64(&path::rpn_freq_mhz(id)).await
}

pub async fn set_boost_freq_mhz(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::boost_freq_mhz(id), v).await
}

pub async fn set_max_freq_mhz(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::max_freq_mhz(id), v).await
}

pub async fn set_min_freq_mhz(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::min_freq_mhz(id), v).await
}

pub async fn set_rp0_freq_mhz(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::rp0_freq_mhz(id), v).await
}

pub async fn set_rp1_freq_mhz(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::rp1_freq_mhz(id), v).await
}

pub async fn set_rpn_freq_mhz(id: u64, v: u64) -> Result<()> {
    sysfs::write_u64(&path::rpn_freq_mhz(id), v).await
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Device {
    pub id: u64,
    pub act_freq_mhz: Option<u64>,
    pub boost_freq_mhz: Option<u64>,
    pub cur_freq_mhz: Option<u64>,
    pub max_freq_mhz: Option<u64>,
    pub min_freq_mhz: Option<u64>,
    pub rp0_freq_mhz: Option<u64>,
    pub rp1_freq_mhz: Option<u64>,
    pub rpn_freq_mhz: Option<u64>,
}

#[async_trait]
impl Values for Device {
    type Id = u64;
    type Output = Self;

    async fn ids() -> Vec<u64> {
        devices().await.ok().unwrap_or_default()
    }

    async fn read(id: u64) -> Option<Self> {
        let act_freq_mhz = act_freq_mhz(id).await.ok();
        let boost_freq_mhz = boost_freq_mhz(id).await.ok();
        let cur_freq_mhz = cur_freq_mhz(id).await.ok();
        let max_freq_mhz = max_freq_mhz(id).await.ok();
        let min_freq_mhz = min_freq_mhz(id).await.ok();
        let rp0_freq_mhz = rp0_freq_mhz(id).await.ok();
        let rp1_freq_mhz = rp1_freq_mhz(id).await.ok();
        let rpn_freq_mhz = rpn_freq_mhz(id).await.ok();
        let s = Self {
            id,
            act_freq_mhz,
            boost_freq_mhz,
            cur_freq_mhz,
            max_freq_mhz,
            min_freq_mhz,
            rp0_freq_mhz,
            rp1_freq_mhz,
            rpn_freq_mhz,
        };
        Some(s)
    }

    async fn write(&self) {
        if let Some(val) = self.boost_freq_mhz {
            let _ = set_boost_freq_mhz(self.id, val).await;
        }
        if let Some(val) = self.max_freq_mhz {
            let _ = set_max_freq_mhz(self.id, val).await;
        }
        if let Some(val) = self.min_freq_mhz {
            let _ = set_min_freq_mhz(self.id, val).await;
        }
        if let Some(val) = self.rp0_freq_mhz {
            let _ = set_rp0_freq_mhz(self.id, val).await;
        }
        if let Some(val) = self.rp1_freq_mhz {
            let _ = set_rp1_freq_mhz(self.id, val).await;
        }
        if let Some(val) = self.rpn_freq_mhz {
            let _ = set_rpn_freq_mhz(self.id, val).await;
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct I915 {
    pub devices: Vec<Device>,
}

#[async_trait]
impl Feature for I915 {
    async fn present() -> bool {
        !Self::ids().await.is_empty()
    }
}

#[async_trait]
impl Values for I915 {
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
