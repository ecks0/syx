use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

use crate::i915::{self, Cache};
use crate::Result;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Values {
    id: u64,
}

impl Values {
    pub fn available() -> impl Future<Output = Result<bool>> {
        i915::available()
    }

    pub fn exists(id: u64) -> impl Future<Output = Result<bool>> {
        i915::exists(id)
    }

    pub fn ids() -> impl Stream<Item = Result<u64>> {
        i915::ids()
    }

    pub fn all() -> impl Stream<Item = Result<Self>> {
        i915::ids().map_ok(Self::new)
    }

    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn act_freq_mhz(&self) -> impl Future<Output = Result<u64>> {
        i915::act_freq_mhz(self.id)
    }

    pub fn boost_freq_mhz(&self) -> impl Future<Output = Result<u64>> {
        i915::boost_freq_mhz(self.id)
    }

    pub fn cur_freq_mhz(&self) -> impl Future<Output = Result<u64>> {
        i915::cur_freq_mhz(self.id)
    }

    pub fn max_freq_mhz(&self) -> impl Future<Output = Result<u64>> {
        i915::max_freq_mhz(self.id)
    }

    pub fn min_freq_mhz(&self) -> impl Future<Output = Result<u64>> {
        i915::min_freq_mhz(self.id)
    }

    pub fn rp0_freq_mhz(&self) -> impl Future<Output = Result<u64>> {
        i915::rp0_freq_mhz(self.id)
    }

    pub fn rp1_freq_mhz(&self) -> impl Future<Output = Result<u64>> {
        i915::rp1_freq_mhz(self.id)
    }

    pub fn rpn_freq_mhz(&self) -> impl Future<Output = Result<u64>> {
        i915::rpn_freq_mhz(self.id)
    }

    pub fn set_boost_freq_mhz(&self, v: u64) -> impl Future<Output = Result<()>> {
        i915::set_boost_freq_mhz(self.id, v)
    }

    pub fn set_max_freq_mhz(&self, v: u64) -> impl Future<Output = Result<()>> {
        i915::set_max_freq_mhz(self.id, v)
    }

    pub fn set_min_freq_mhz(&self, v: u64) -> impl Future<Output = Result<()>> {
        i915::set_min_freq_mhz(self.id, v)
    }

    pub fn set_rp0_freq_mhz(&self, v: u64) -> impl Future<Output = Result<()>> {
        i915::set_rp0_freq_mhz(self.id, v)
    }

    pub fn set_rp1_freq_mhz(&self, v: u64) -> impl Future<Output = Result<()>> {
        i915::set_rp1_freq_mhz(self.id, v)
    }

    pub fn set_rpn_freq_mhz(&self, v: u64) -> impl Future<Output = Result<()>> {
        i915::set_rpn_freq_mhz(self.id, v)
    }
}

impl From<Cache> for Values {
    fn from(v: Cache) -> Self {
        Self::new(v.id())
    }
}

impl From<&Cache> for Values {
    fn from(v: &Cache) -> Self {
        Self::new(v.id())
    }
}
