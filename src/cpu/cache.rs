use futures::stream::{Stream, TryStreamExt as _};
use futures::Future;

use crate::cpu::{self, Values};
use crate::util::cell::Cell;
use crate::Result;

#[derive(Clone, Debug)]
pub struct Cache {
    id: u64,
    online: Cell<bool>,
}

impl Cache {
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
        let online = Cell::default();
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
