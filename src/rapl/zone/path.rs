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
