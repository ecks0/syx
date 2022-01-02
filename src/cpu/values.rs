use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

use crate::cpu::{self, Cache};
use crate::Result;

#[derive(Clone, Debug)]
pub struct Values {
    id: u64,
}

impl Values {
    pub fn available() -> impl Future<Output = Result<bool>> {
        cpu::available()
    }

    pub fn exists(id: u64) -> impl Future<Output = Result<bool>> {
        cpu::exists(id)
    }

    pub fn ids() -> impl Stream<Item = Result<u64>> {
        cpu::ids()
    }

    pub fn all() -> impl Stream<Item = Result<Self>> {
        cpu::ids().map_ok(Self::new)
    }

    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn online(&self) -> impl Future<Output=Result<bool>> {
        cpu::online(self.id)
    }

    pub fn set_online(&self, v: bool) -> impl Future<Output=Result<()>> {
        cpu::set_online(self.id, v)
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
