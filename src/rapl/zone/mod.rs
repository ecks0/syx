mod cache;
pub(crate) mod path;
mod record;

pub use crate::rapl::zone::cache::Cache;
pub use crate::rapl::zone::record::Record;

pub use crate::rapl::available;
use crate::util::stream::prelude::*;
use crate::util::sysfs;
use crate::Result;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id {
    package: u64,
    subzone: Option<u64>,
}

impl Id {
    pub fn new(package: u64, subzone: Option<u64>) -> Self {
        Self { package, subzone }
    }

    pub fn package(&self) -> u64 {
        self.package
    }

    pub fn subzone(&self) -> Option<u64> {
        self.subzone
    }
}

impl From<(u64, Option<u64>)> for Id {
    fn from(v: (u64, Option<u64>)) -> Self {
        Self::new(v.0, v.1)
    }
}

impl From<(u64, u64)> for Id {
    fn from(v: (u64, u64)) -> Self {
        Self::new(v.0, Some(v.1))
    }
}

impl From<Id> for (u64, Option<u64>) {
    fn from(v: Id) -> Self {
        (v.package, v.subzone)
    }
}

pub fn packages() -> impl Stream<Item=Result<Id>> {
    try_stream! {
        let s = sysfs::read_ids(path::root(), "intel-rapl:");
        for await v in s {
            let v = v?;
            let y = Id::from((v, None));
            yield y;
        }
    }
}

pub fn subzones(package: u64) -> impl Stream<Item=Result<Id>> {
    try_stream! {
        let s = sysfs::read_ids(path::package(package), format!("intel-rapl:{}:", package));
        for await v in s {
            let v = v?;
            let y = Id::from((package, Some(v)));
            yield y;
        }
    }
}

pub fn ids() -> impl Stream<Item=Result<Id>> {
    try_stream! {
        for await p in packages() {
            let p = p?;
            yield p;
            for await s in subzones(p.package) {
                let s = s?;
                yield s;
            }
        }
    }
}

pub async fn exists(id: impl Into<Id>) -> Result<bool> {
    let id = id.into();
    let r = if let Some(subzone) = &id.subzone {
        path::subzone(id.package, *subzone).is_dir()
    } else {
        path::package(id.package).is_dir()
    };
    Ok(r)
}

pub async fn enabled(id: impl Into<Id>) -> Result<bool> {
    sysfs::read_bool(&path::enabled(id.into())).await
}

pub async fn energy_uj(id: impl Into<Id>) -> Result<u64> {
    sysfs::read_u64(&path::energy_uj(id.into())).await
}

pub async fn max_energy_range_uj(id: impl Into<Id>) -> Result<u64> {
    sysfs::read_u64(&path::max_energy_range_uj(id.into())).await
}

pub async fn name(id: impl Into<Id>) -> Result<String> {
    sysfs::read_string(&path::name(id.into())).await
}

pub async fn set_enabled(id: impl Into<Id>, v: bool) -> Result<()> {
    sysfs::write_bool(&path::enabled(id.into()), v).await
}
