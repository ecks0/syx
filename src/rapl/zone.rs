pub(crate) mod path {
    use std::path::PathBuf;

    pub(crate) use crate::rapl::path::{root, package, subzone};

    pub(crate) fn device_attr(id: (u64, Option<u64>), a: &str) -> PathBuf {
        use crate::rapl::path::device_attr;
        device_attr(id.0, id.1, a)
    }

    pub(crate) fn enabled(id: (u64, Option<u64>)) -> PathBuf {
        device_attr(id, "enabled")
    }

    pub(crate) fn energy_uj(id: (u64, Option<u64>)) -> PathBuf {
        device_attr(id, "energy_uj")
    }

    pub(crate) fn max_energy_range_uj(id: (u64, Option<u64>)) -> PathBuf {
        device_attr(id, "max_energy_range_uj")
    }

    pub(crate) fn name(id: (u64, Option<u64>)) -> PathBuf {
        device_attr(id, "name")
    }
}

pub use crate::rapl::available;
use crate::util::sysfs;
use crate::util::cell::Cell;
use crate::Result;

pub async fn packages() -> Result<Vec<(u64, Option<u64>)>> {
    sysfs::read_ids(&path::root(), "intel-rapl:").await
        .map(|v| v
            .into_iter()
            .map(|v| (v, None))
            .collect())
}

pub async fn subzones(package: u64) -> Result<Vec<(u64, Option<u64>)>> {
    sysfs::read_ids(&path::package(package), &format!("intel-rapl:{}:", package))
        .await
        .map(|v| v
            .into_iter()
            .map(|v| (package, Some(v)))
            .collect())
}

pub async fn ids() -> Result<Vec<(u64, Option<u64>)>> {
    let mut ids = vec![];
    for id in packages().await? {
        ids.push(id);
        let v = subzones(id.0).await?;
        ids.extend(v);
    }
    Ok(ids)
}

pub async fn exists(id: (u64, Option<u64>)) -> bool {
    if let Some(subzone) = id.1 {
        path::subzone(id.0, subzone).is_dir()
    } else {
        path::package(id.0).is_dir()
    }
}

pub async fn enabled(id: (u64, Option<u64>)) -> Result<bool> {
    sysfs::read_bool(&path::enabled(id)).await
}

pub async fn energy_uj(id: (u64, Option<u64>)) -> Result<u64> {
    sysfs::read_u64(&path::energy_uj(id)).await
}

pub async fn max_energy_range_uj(id: (u64, Option<u64>)) -> Result<u64> {
    sysfs::read_u64(&path::max_energy_range_uj(id)).await
}

pub async fn name(id: (u64, Option<u64>)) -> Result<String> {
    sysfs::read_string(&path::name(id)).await
}

pub async fn set_enabled(id: (u64, Option<u64>), v: bool) -> Result<()> {
    sysfs::write_bool(&path::enabled(id), v).await
}

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

impl From<Id> for (u64, Option<u64>) {
    fn from(v: Id) -> Self {
        (v.package, v.subzone)
    }
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
    pub async fn available() -> bool {
        available().await
    }

    pub async fn ids() -> Result<Vec<Id>> {
        ids().await.map(|ids| ids
            .into_iter()
            .map(Id::from)
            .collect())
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
        self.enabled
            .get_or_load(enabled(self.id.into()))
            .await
    }

    pub async fn energy_uj(&self) -> Result<u64> {
        self.energy_uj
            .get_or_load(energy_uj(self.id.into()))
            .await
    }

    pub async fn max_energy_range_uj(&self) -> Result<u64> {
        self.max_energy_range_uj
            .get_or_load(max_energy_range_uj(self.id.into()))
            .await
    }

    pub async fn name(&self) -> Result<String> {
        self.name
            .get_or_load(name(self.id.into()))
            .await
    }

    pub async fn set_enabled(&self, v: bool) -> Result<()> {
        self.enabled
            .clear_if_ok(set_enabled(self.id.into(), v))
            .await
    }
}
