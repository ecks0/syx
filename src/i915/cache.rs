use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

use crate::i915::{self, Values};
use crate::util::cell::Cell;
use crate::Result;

#[derive(Clone, Debug)]
pub struct Cache {
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

impl Cache {
    pub fn available() -> impl Future<Output = Result<bool>> {
        i915::available()
    }

    pub fn exists(id: u64) -> impl Future<Output = Result<bool>> {
        i915::exists(id)
    }

    pub async fn ids() -> impl Stream<Item = Result<u64>> {
        i915::ids()
    }

    pub fn all() -> impl Stream<Item = Result<Self>> {
        i915::ids().map_ok(Self::new)
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
        self.act_freq_mhz
            .get_or_load(i915::act_freq_mhz(self.id))
            .await
    }

    pub async fn boost_freq_mhz(&self) -> Result<u64> {
        self.boost_freq_mhz
            .get_or_load(i915::boost_freq_mhz(self.id))
            .await
    }

    pub async fn cur_freq_mhz(&self) -> Result<u64> {
        self.cur_freq_mhz
            .get_or_load(i915::cur_freq_mhz(self.id))
            .await
    }

    pub async fn max_freq_mhz(&self) -> Result<u64> {
        self.max_freq_mhz
            .get_or_load(i915::max_freq_mhz(self.id))
            .await
    }

    pub async fn min_freq_mhz(&self) -> Result<u64> {
        self.min_freq_mhz
            .get_or_load(i915::min_freq_mhz(self.id))
            .await
    }

    pub async fn rp0_freq_mhz(&self) -> Result<u64> {
        self.rp0_freq_mhz
            .get_or_load(i915::rp0_freq_mhz(self.id))
            .await
    }

    pub async fn rp1_freq_mhz(&self) -> Result<u64> {
        self.rp1_freq_mhz
            .get_or_load(i915::rp1_freq_mhz(self.id))
            .await
    }

    pub async fn rpn_freq_mhz(&self) -> Result<u64> {
        self.rpn_freq_mhz
            .get_or_load(i915::rpn_freq_mhz(self.id))
            .await
    }

    pub async fn set_boost_freq_mhz(&self, v: u64) -> Result<()> {
        self.boost_freq_mhz
            .clear_if_ok(i915::set_boost_freq_mhz(self.id, v))
            .await
    }

    pub async fn set_max_freq_mhz(&self, v: u64) -> Result<()> {
        self.max_freq_mhz
            .clear_if_ok(i915::set_max_freq_mhz(self.id, v))
            .await
    }

    pub async fn set_min_freq_mhz(&self, v: u64) -> Result<()> {
        self.min_freq_mhz
            .clear_if_ok(i915::set_min_freq_mhz(self.id, v))
            .await
    }

    pub async fn set_rp0_freq_mhz(&self, v: u64) -> Result<()> {
        self.rp0_freq_mhz
            .clear_if_ok(i915::set_rp0_freq_mhz(self.id, v))
            .await
    }

    pub async fn set_rp1_freq_mhz(&self, v: u64) -> Result<()> {
        self.rp1_freq_mhz
            .clear_if_ok(i915::set_rp1_freq_mhz(self.id, v))
            .await
    }

    pub async fn set_rpn_freq_mhz(&self, v: u64) -> Result<()> {
        self.rpn_freq_mhz
            .clear_if_ok(i915::set_rpn_freq_mhz(self.id, v))
            .await
    }
}

impl From<Values> for Cache {
    fn from(v: Values) -> Self {
        Self::new(v.id())
    }
}

impl From<&Values> for Cache {
    fn from(v: &Values) -> Self {
        Self::new(v.id())
    }
}
