use std::path::PathBuf;

use crate::intel_rapl::constraint::Id;

pub(crate) fn constraint_attr(id: Id, a: &str) -> PathBuf {
    use crate::intel_rapl::path::zone_attr;
    zone_attr(
        id.package,
        id.subzone,
        &format!("constraint_{}_{}", id.index, a),
    )
}

pub(crate) fn name(id: Id) -> PathBuf {
    constraint_attr(id, "name")
}

pub(crate) fn max_power_uw(id: Id) -> PathBuf {
    constraint_attr(id, "max_power_uw")
}

pub(crate) fn power_limit_uw(id: Id) -> PathBuf {
    constraint_attr(id, "power_limit_uw")
}

pub(crate) fn time_window_us(id: Id) -> PathBuf {
    constraint_attr(id, "time_window_us")
}
