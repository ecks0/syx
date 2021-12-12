pub(crate) mod path {
    use std::path::PathBuf;

    pub(crate) fn root() -> PathBuf {
        PathBuf::from("/sys/class/drm")
    }

    pub(crate) fn device(id: u64) -> PathBuf {
        let mut p = root();
        p.push(format!("card{}", id));
        p
    }

    pub(crate) fn device_attr(id: u64, s: &str) -> PathBuf {
        let mut p = device(id);
        p.push(s);
        p
    }

    pub(crate) fn driver(id: u64) -> PathBuf {
        let mut p = device_attr(id, "device");
        p.push("driver");
        p
    }

    pub(crate) fn act_freq_mhz(id: u64) -> PathBuf {
        device_attr(id, "gt_act_freq_mhz")
    }

    pub(crate) fn boost_freq_mhz(id: u64) -> PathBuf {
        device_attr(id, "gt_boost_freq_mhz")
    }

    pub(crate) fn cur_freq_mhz(id: u64) -> PathBuf {
        device_attr(id, "gt_cur_freq_mhz")
    }

    pub(crate) fn max_freq_mhz(id: u64) -> PathBuf {
        device_attr(id, "gt_max_freq_mhz")
    }

    pub(crate) fn min_freq_mhz(id: u64) -> PathBuf {
        device_attr(id, "gt_min_freq_mhz")
    }

    pub(crate) fn rp0_freq_mhz(id: u64) -> PathBuf {
        device_attr(id, "gt_RP0_freq_mhz")
    }

    pub(crate) fn rp1_freq_mhz(id: u64) -> PathBuf {
        device_attr(id, "gt_RP1_freq_mhz")
    }

    pub(crate) fn rpn_freq_mhz(id: u64) -> PathBuf {
        device_attr(id, "gt_RPn_freq_mhz")
    }
}

use async_trait::async_trait;

use crate::util::sysfs;
use crate::{Feature, Multi, Read, Result, Single, Values, Write};

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
    id: u64,
    act_freq_mhz: Option<u64>,
    boost_freq_mhz: Option<u64>,
    cur_freq_mhz: Option<u64>,
    max_freq_mhz: Option<u64>,
    min_freq_mhz: Option<u64>,
    rp0_freq_mhz: Option<u64>,
    rp1_freq_mhz: Option<u64>,
    rpn_freq_mhz: Option<u64>,
}

impl Device {
    pub fn act_freq_mhz(&self) -> Option<u64> {
        self.act_freq_mhz
    }

    pub fn boost_freq_mhz(&self) -> Option<u64> {
        self.boost_freq_mhz
    }

    pub fn cur_freq_mhz(&self) -> Option<u64> {
        self.cur_freq_mhz
    }

    pub fn max_freq_mhz(&self) -> Option<u64> {
        self.max_freq_mhz
    }

    pub fn min_freq_mhz(&self) -> Option<u64> {
        self.min_freq_mhz
    }

    pub fn rp0_freq_mhz(&self) -> Option<u64> {
        self.rp0_freq_mhz
    }

    pub fn rp1_freq_mhz(&self) -> Option<u64> {
        self.rp1_freq_mhz
    }

    pub fn rpn_freq_mhz(&self) -> Option<u64> {
        self.rpn_freq_mhz
    }

    pub fn set_boost_freq_mhz(&mut self, v: impl Into<Option<u64>>) -> &mut Self {
        self.boost_freq_mhz = v.into();
        self
    }

    pub fn set_max_freq_mhz(&mut self, v: impl Into<Option<u64>>) -> &mut Self {
        self.max_freq_mhz = v.into();
        self
    }

    pub fn set_min_freq_mhz(&mut self, v: impl Into<Option<u64>>) -> &mut Self {
        self.min_freq_mhz = v.into();
        self
    }

    pub fn set_rp0_freq_mhz(&mut self, v: impl Into<Option<u64>>) -> &mut Self {
        self.rp0_freq_mhz = v.into();
        self
    }

    pub fn set_rp1_freq_mhz(&mut self, v: impl Into<Option<u64>>) -> &mut Self {
        self.rp1_freq_mhz = v.into();
        self
    }

    pub fn set_rpn_freq_mhz(&mut self, v: impl Into<Option<u64>>) -> &mut Self {
        self.rpn_freq_mhz = v.into();
        self
    }
}

#[async_trait]
impl Read for Device {
    async fn read(&mut self) {
        self.act_freq_mhz = act_freq_mhz(self.id).await.ok();
        self.boost_freq_mhz = boost_freq_mhz(self.id).await.ok();
        self.cur_freq_mhz = cur_freq_mhz(self.id).await.ok();
        self.max_freq_mhz = max_freq_mhz(self.id).await.ok();
        self.min_freq_mhz = min_freq_mhz(self.id).await.ok();
        self.rp0_freq_mhz = rp0_freq_mhz(self.id).await.ok();
        self.rp1_freq_mhz = rp1_freq_mhz(self.id).await.ok();
        self.rpn_freq_mhz = rpn_freq_mhz(self.id).await.ok();
    }
}

#[async_trait]
impl Write for Device {
    async fn write(&self) {
        if let Some(v) = self.boost_freq_mhz {
            let _ = set_boost_freq_mhz(self.id, v).await;
        }
        if let Some(v) = self.max_freq_mhz {
            let _ = set_max_freq_mhz(self.id, v).await;
        }
        if let Some(v) = self.min_freq_mhz {
            let _ = set_min_freq_mhz(self.id, v).await;
        }
        if let Some(v) = self.rp0_freq_mhz {
            let _ = set_rp0_freq_mhz(self.id, v).await;
        }
        if let Some(v) = self.rp1_freq_mhz {
            let _ = set_rp1_freq_mhz(self.id, v).await;
        }
        if let Some(v) = self.rpn_freq_mhz {
            let _ = set_rpn_freq_mhz(self.id, v).await;
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
    type Id = u64;

    async fn ids() -> Vec<Self::Id> {
        devices().await.unwrap_or_default()
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

impl System {
    pub fn push_device(&mut self, v: Device) -> &mut Self {
        if let Some(i) = self.devices.iter().position(|d| v.id.eq(&d.id)) {
            self.devices[i] = v;
        } else {
            self.devices.push(v);
            self.devices.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        }
        self
    }

    pub fn push_devices(&mut self, v: impl IntoIterator<Item = Device>) -> &mut Self {
        for d in v.into_iter() {
            self.push_device(d);
        }
        self
    }

    pub fn devices(&self) -> std::slice::Iter<'_, Device> {
        self.devices.iter()
    }

    pub fn into_devices(self) -> impl IntoIterator<Item = Device> {
        self.devices.into_iter()
    }
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
        !Device::ids().await.is_empty()
    }
}
