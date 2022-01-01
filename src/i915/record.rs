use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

use crate::{i915, Result};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Record {
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

impl Record {
    pub fn available() -> impl Future<Output = Result<bool>> {
        i915::available()
    }

    pub fn exists(id: u64) -> impl Future<Output = Result<bool>> {
        i915::exists(id)
    }

    pub fn ids() -> impl Stream<Item = Result<u64>> {
        i915::ids()
    }

    pub async fn load(id: u64) -> Self {
        let mut s = Self::new(id);
        s.read().await;
        s
    }

    pub fn all() -> impl Stream<Item = Result<Self>> {
        i915::ids().and_then(|id| async move { Ok(Self::load(id).await) })
    }

    pub fn new(id: u64) -> Self {
        Self {
            id,
            act_freq_mhz: Option::default(),
            boost_freq_mhz: Option::default(),
            cur_freq_mhz: Option::default(),
            max_freq_mhz: Option::default(),
            min_freq_mhz: Option::default(),
            rp0_freq_mhz: Option::default(),
            rp1_freq_mhz: Option::default(),
            rpn_freq_mhz: Option::default(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn is_empty(&self) -> bool {
        self == &Self::new(self.id)
    }

    pub async fn read(&mut self) -> bool {
        self.act_freq_mhz = i915::act_freq_mhz(self.id).await.ok();
        self.boost_freq_mhz = i915::boost_freq_mhz(self.id).await.ok();
        self.cur_freq_mhz = i915::cur_freq_mhz(self.id).await.ok();
        self.max_freq_mhz = i915::max_freq_mhz(self.id).await.ok();
        self.min_freq_mhz = i915::min_freq_mhz(self.id).await.ok();
        self.rp0_freq_mhz = i915::rp0_freq_mhz(self.id).await.ok();
        self.rp1_freq_mhz = i915::rp1_freq_mhz(self.id).await.ok();
        self.rpn_freq_mhz = i915::rpn_freq_mhz(self.id).await.ok();
        !self.is_empty()
    }

    pub async fn act_freq_mhz(&self) -> Option<u64> {
        self.act_freq_mhz
    }

    pub async fn boost_freq_mhz(&self) -> Option<u64> {
        self.boost_freq_mhz
    }

    pub async fn cur_freq_mhz(&self) -> Option<u64> {
        self.cur_freq_mhz
    }

    pub async fn max_freq_mhz(&self) -> Option<u64> {
        self.max_freq_mhz
    }

    pub async fn min_freq_mhz(&self) -> Option<u64> {
        self.min_freq_mhz
    }

    pub async fn rp0_freq_mhz(&self) -> Option<u64> {
        self.rp0_freq_mhz
    }

    pub async fn rp1_freq_mhz(&self) -> Option<u64> {
        self.rp1_freq_mhz
    }

    pub async fn rpn_freq_mhz(&self) -> Option<u64> {
        self.rpn_freq_mhz
    }
}
