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
