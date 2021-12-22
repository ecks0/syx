pub(crate) mod path {
    use std::path::PathBuf;

    use crate::drm::path::card_attr;

    pub(crate) fn module() -> PathBuf {
        PathBuf::from("/sys/module/i915")
    }

    pub(crate) fn act_freq_mhz(id: u64) -> PathBuf {
        card_attr(id, "gt_act_freq_mhz")
    }

    pub(crate) fn boost_freq_mhz(id: u64) -> PathBuf {
        card_attr(id, "gt_boost_freq_mhz")
    }

    pub(crate) fn cur_freq_mhz(id: u64) -> PathBuf {
        card_attr(id, "gt_cur_freq_mhz")
    }

    pub(crate) fn max_freq_mhz(id: u64) -> PathBuf {
        card_attr(id, "gt_max_freq_mhz")
    }

    pub(crate) fn min_freq_mhz(id: u64) -> PathBuf {
        card_attr(id, "gt_min_freq_mhz")
    }

    pub(crate) fn rp0_freq_mhz(id: u64) -> PathBuf {
        card_attr(id, "gt_RP0_freq_mhz")
    }

    pub(crate) fn rp1_freq_mhz(id: u64) -> PathBuf {
        card_attr(id, "gt_RP1_freq_mhz")
    }

    pub(crate) fn rpn_freq_mhz(id: u64) -> PathBuf {
        card_attr(id, "gt_RPn_freq_mhz")
    }
}

use crate::util::cell::Cell;
use crate::util::sysfs;
use crate::{drm, Result};

pub async fn available() -> Result<bool> {
    Ok(path::module().is_dir())
}

pub async fn exists(id: u64) -> Result<bool> {
    if drm::exists(id).await? {
        Ok("i915" == drm::driver(id).await?.as_str())
    } else {
        Ok(false)
    }
}

pub async fn ids() -> Result<Vec<u64>> {
    drm::ids_for_driver("i915").await
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
    act_freq_mhz: Cell<u64>,
    boost_freq_mhz: Cell<u64>,
    cur_freq_mhz: Cell<u64>,
    max_freq_mhz: Cell<u64>,
    min_freq_mhz: Cell<u64>,
    rp0_freq_mhz: Cell<u64>,
    rp1_freq_mhz: Cell<u64>,
    rpn_freq_mhz: Cell<u64>,
}

impl Card {
    pub async fn available() -> Result<bool> {
        available().await
    }

    pub async fn exists(id: u64) -> Result<bool> {
        exists(id).await
    }

    pub async fn ids() -> Result<Vec<u64>> {
        ids().await
    }

    pub fn new(id: u64) -> Self {
        Self {
            id,
            act_freq_mhz: Cell::default(),
            boost_freq_mhz: Cell::default(),
            cur_freq_mhz: Cell::default(),
            max_freq_mhz: Cell::default(),
            min_freq_mhz: Cell::default(),
            rp0_freq_mhz: Cell::default(),
            rp1_freq_mhz: Cell::default(),
            rpn_freq_mhz: Cell::default(),
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
        self.act_freq_mhz.get_or_load(act_freq_mhz(self.id)).await
    }

    pub async fn boost_freq_mhz(&self) -> Result<u64> {
        self.boost_freq_mhz
            .get_or_load(boost_freq_mhz(self.id))
            .await
    }

    pub async fn cur_freq_mhz(&self) -> Result<u64> {
        self.cur_freq_mhz.get_or_load(cur_freq_mhz(self.id)).await
    }

    pub async fn max_freq_mhz(&self) -> Result<u64> {
        self.max_freq_mhz.get_or_load(max_freq_mhz(self.id)).await
    }

    pub async fn min_freq_mhz(&self) -> Result<u64> {
        self.min_freq_mhz.get_or_load(min_freq_mhz(self.id)).await
    }

    pub async fn rp0_freq_mhz(&self) -> Result<u64> {
        self.rp0_freq_mhz.get_or_load(rp0_freq_mhz(self.id)).await
    }

    pub async fn rp1_freq_mhz(&self) -> Result<u64> {
        self.rp1_freq_mhz.get_or_load(rp1_freq_mhz(self.id)).await
    }

    pub async fn rpn_freq_mhz(&self) -> Result<u64> {
        self.rpn_freq_mhz.get_or_load(rpn_freq_mhz(self.id)).await
    }

    pub async fn set_boost_freq_mhz(&self, v: u64) -> Result<()> {
        self.boost_freq_mhz
            .clear_if_ok(set_boost_freq_mhz(self.id, v))
            .await
    }

    pub async fn set_max_freq_mhz(&self, v: u64) -> Result<()> {
        self.max_freq_mhz
            .clear_if_ok(set_max_freq_mhz(self.id, v))
            .await
    }

    pub async fn set_min_freq_mhz(&self, v: u64) -> Result<()> {
        self.min_freq_mhz
            .clear_if_ok(set_min_freq_mhz(self.id, v))
            .await
    }

    pub async fn set_rp0_freq_mhz(&self, v: u64) -> Result<()> {
        self.rp0_freq_mhz
            .clear_if_ok(set_rp0_freq_mhz(self.id, v))
            .await
    }

    pub async fn set_rp1_freq_mhz(&self, v: u64) -> Result<()> {
        self.rp1_freq_mhz
            .clear_if_ok(set_rp1_freq_mhz(self.id, v))
            .await
    }

    pub async fn set_rpn_freq_mhz(&self, v: u64) -> Result<()> {
        self.rpn_freq_mhz
            .clear_if_ok(set_rpn_freq_mhz(self.id, v))
            .await
    }
}
