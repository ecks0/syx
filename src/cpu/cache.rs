use futures::Future;

use crate::cpu;
use crate::util::cell::Cached;
use crate::util::stream::prelude::*;
use crate::Result;

#[derive(Clone, Debug)]
pub struct Cache {
    id: u64,
    online: Cached<bool>,
}

impl Cache {
    pub fn available() -> impl Future<Output=Result<bool>> {
        cpu::available()
    }

    pub async fn exists(id: u64) -> impl Future<Output=Result<bool>> {
        cpu::exists(id)
    }

    pub fn ids() -> impl Stream<Item=Result<u64>> {
        cpu::ids()
    }

    pub fn new(id: u64) -> Self {
        let online = Cached::default();
        Self { id, online }
    }

    pub async fn clear(&self) {
        self.online.clear().await;
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub async fn online(&self) -> Result<bool> {
        self.online.get_or_load(cpu::online(self.id)).await
    }

    pub async fn set_online(&self, v: bool) -> Result<()> {
        self.online.clear_if_ok(cpu::set_online(self.id, v)).await
    }
}
