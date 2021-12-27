use futures::Future;

use crate::cpu;
use crate::util::stream::prelude::*;
use crate::Result;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Record {
    id: u64,
    online: Option<bool>,
}

impl Record {
    pub fn available() -> impl Future<Output=Result<bool>> {
        cpu::available()
    }

    pub async fn exists(id: u64) -> impl Future<Output=Result<bool>> {
        cpu::exists(id)
    }

    pub fn ids() -> impl Stream<Item=Result<u64>> {
        cpu::ids()
    }

    pub async fn load(id: u64) -> Self {
        let mut s = Self::new(id);
        s.read().await;
        s
    }

    pub fn new(id: u64) -> Self {
        Self {
            id,
            online: None,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn is_empty(&self) -> bool {
        self == &Self::new(self.id)
    }

    pub async fn read(&mut self) -> bool {
        self.online = cpu::online(self.id).await.ok();
        !self.is_empty()
    }

    pub fn online(&self) -> Option<bool> {
        self.online
    }
}
