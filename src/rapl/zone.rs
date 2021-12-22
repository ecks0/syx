pub(crate) mod path {
    use std::path::PathBuf;

    pub(crate) use crate::rapl::path::{package, root, subzone};
    use crate::rapl::zone::Id;

    pub(crate) fn zone_attr(id: Id, a: &str) -> PathBuf {
        use crate::rapl::path::zone_attr;
        zone_attr(id.package, id.subzone, a)
    }

    pub(crate) fn enabled(id: Id) -> PathBuf {
        zone_attr(id, "enabled")
    }

    pub(crate) fn energy_uj(id: Id) -> PathBuf {
        zone_attr(id, "energy_uj")
    }

    pub(crate) fn max_energy_range_uj(id: Id) -> PathBuf {
        zone_attr(id, "max_energy_range_uj")
    }

    pub(crate) fn name(id: Id) -> PathBuf {
        zone_attr(id, "name")
    }
}

pub use crate::rapl::available;
use crate::util::cell::Cell;
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

pub async fn packages() -> Result<Vec<Id>> {
    sysfs::read_ids(&path::root(), "intel-rapl:")
        .await
        .map(|v| v.into_iter().map(|v| Id::from((v, None))).collect())
}

pub async fn subzones(package: u64) -> Result<Vec<Id>> {
    sysfs::read_ids(&path::package(package), &format!("intel-rapl:{}:", package))
        .await
        .map(|v| {
            v.into_iter()
                .map(|v| Id::from((package, Some(v))))
                .collect()
        })
}

pub async fn ids() -> Result<Vec<Id>> {
    let mut ids = vec![];
    for id in packages().await? {
        ids.push(id);
        let v = subzones(id.package).await?;
        ids.extend(v);
    }
    Ok(ids)
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

#[derive(Clone, Debug)]
pub struct Zone {
    id: Id,
    enabled: Cell<bool>,
    energy_uj: Cell<u64>,
    max_energy_range_uj: Cell<u64>,
    name: Cell<String>,
}

impl Zone {
    pub async fn available() -> Result<bool> {
        available().await
    }

    pub async fn exists(id: Id) -> Result<bool> {
        exists(id).await
    }

    pub async fn ids() -> Result<Vec<Id>> {
        ids()
            .await
            .map(|ids| ids.into_iter().map(Id::from).collect())
    }

    pub fn new(id: impl Into<Id>) -> Self {
        let id = id.into();
        let enabled = Cell::default();
        let energy_uj = Cell::default();
        let max_energy_range_uj = Cell::default();
        let name = Cell::default();
        Self {
            id,
            enabled,
            energy_uj,
            max_energy_range_uj,
            name,
        }
    }

    pub async fn clear(&self) {
        tokio::join!(
            self.enabled.clear(),
            self.energy_uj.clear(),
            self.max_energy_range_uj.clear(),
            self.name.clear(),
        );
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub async fn enabled(&self) -> Result<bool> {
        self.enabled.get_or_load(enabled(self.id)).await
    }

    pub async fn energy_uj(&self) -> Result<u64> {
        self.energy_uj.get_or_load(energy_uj(self.id)).await
    }

    pub async fn max_energy_range_uj(&self) -> Result<u64> {
        self.max_energy_range_uj
            .get_or_load(max_energy_range_uj(self.id))
            .await
    }

    pub async fn name(&self) -> Result<String> {
        self.name.get_or_load(name(self.id)).await
    }

    pub async fn set_enabled(&self, v: bool) -> Result<()> {
        self.enabled.clear_if_ok(set_enabled(self.id, v)).await
    }
}
