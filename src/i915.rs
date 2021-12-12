pub(crate) mod path {
    use std::path::PathBuf;

    pub(crate) fn module() -> PathBuf {
        PathBuf::from("/sys/module/i915")
    }

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

use crate::{sysfs, Cached, Result};

pub async fn available() -> bool {
    path::module().is_dir()
}

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

#[derive(Clone, Debug)]
pub struct Card {
    id: u64,
    act_freq_mhz: Cached<u64>,
    boost_freq_mhz: Cached<u64>,
    cur_freq_mhz: Cached<u64>,
    max_freq_mhz: Cached<u64>,
    min_freq_mhz: Cached<u64>,
    rp0_freq_mhz: Cached<u64>,
    rp1_freq_mhz: Cached<u64>,
    rpn_freq_mhz: Cached<u64>,
}

impl Card {
    pub async fn available() -> bool {
        available().await
    }

    pub async fn ids() -> Result<Vec<u64>> {
        devices().await
    }

    pub fn new(id: u64) -> Self {
        let act_freq_mhz = Cached::default();
        let boost_freq_mhz = Cached::default();
        let cur_freq_mhz = Cached::default();
        let max_freq_mhz = Cached::default();
        let min_freq_mhz = Cached::default();
        let rp0_freq_mhz = Cached::default();
        let rp1_freq_mhz = Cached::default();
        let rpn_freq_mhz = Cached::default();
        Self {
            id,
            act_freq_mhz,
            boost_freq_mhz,
            cur_freq_mhz,
            max_freq_mhz,
            min_freq_mhz,
            rp0_freq_mhz,
            rp1_freq_mhz,
            rpn_freq_mhz,
        }
    }

    pub async fn clear(&self) {
        tokio::join!(
            self.act_freq_mhz.clear(),
            self.boost_freq_mhz.clear(),
            self.cur_freq_mhz.clear(),
            self.max_freq_mhz.clear(),
            self.min_freq_mhz.clear(),
            self.rp0_freq_mhz.clear(),
            self.rp1_freq_mhz.clear(),
            self.rpn_freq_mhz.clear(),
        );
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub async fn act_freq_mhz(&self) -> Result<u64> {
        self.act_freq_mhz.get_with(act_freq_mhz(self.id)).await
    }

    pub async fn boost_freq_mhz(&self) -> Result<u64> {
        self.boost_freq_mhz.get_with(boost_freq_mhz(self.id)).await
    }

    pub async fn cur_freq_mhz(&self) -> Result<u64> {
        self.cur_freq_mhz.get_with(cur_freq_mhz(self.id)).await
    }

    pub async fn max_freq_mhz(&self) -> Result<u64> {
        self.max_freq_mhz.get_with(max_freq_mhz(self.id)).await
    }

    pub async fn min_freq_mhz(&self) -> Result<u64> {
        self.min_freq_mhz.get_with(min_freq_mhz(self.id)).await
    }

    pub async fn rp0_freq_mhz(&self) -> Result<u64> {
        self.rp0_freq_mhz.get_with(rp0_freq_mhz(self.id)).await
    }

    pub async fn rp1_freq_mhz(&self) -> Result<u64> {
        self.rp1_freq_mhz.get_with(rp1_freq_mhz(self.id)).await
    }

    pub async fn rpn_freq_mhz(&self) -> Result<u64> {
        self.rpn_freq_mhz.get_with(rpn_freq_mhz(self.id)).await
    }

    pub async fn set_boost_freq_mhz(&self, v: u64) -> Result<()> {
        self.boost_freq_mhz
            .clear_if(set_boost_freq_mhz(self.id, v))
            .await
    }

    pub async fn set_max_freq_mhz(&self, v: u64) -> Result<()> {
        self.max_freq_mhz
            .clear_if(set_max_freq_mhz(self.id, v))
            .await
    }

    pub async fn set_min_freq_mhz(&self, v: u64) -> Result<()> {
        self.min_freq_mhz
            .clear_if(set_min_freq_mhz(self.id, v))
            .await
    }

    pub async fn set_rp0_freq_mhz(&self, v: u64) -> Result<()> {
        self.rp0_freq_mhz
            .clear_if(set_rp0_freq_mhz(self.id, v))
            .await
    }

    pub async fn set_rp1_freq_mhz(&self, v: u64) -> Result<()> {
        self.rp1_freq_mhz
            .clear_if(set_rp1_freq_mhz(self.id, v))
            .await
    }

    pub async fn set_rpn_freq_mhz(&self, v: u64) -> Result<()> {
        self.rpn_freq_mhz
            .clear_if(set_rpn_freq_mhz(self.id, v))
            .await
    }
}
