use chrono::{Duration, prelude::*};
use log::debug;

fn format_dt(dt: Duration) -> String {
    if let Some(ns) = dt.num_nanoseconds() {
        format!("{} ns", ns)
    } else if let Some(us) = dt.num_microseconds() {
        format!("{} us", us)
    } else {
        format!("{} ms", dt.num_milliseconds())
    }
}

#[derive(Debug)]
pub struct Timer {
    start: DateTime<Utc>,
}

impl Timer {

    pub fn start() -> Self {
        Self { start: Utc::now() }
    }

    pub fn reset(&mut self) {
        self.start = Utc::now();
    }

    pub fn end(&mut self, mark: &str) {
        let dt = Utc::now() - self.start;
        debug!("{} took {}", mark, format_dt(dt));
        self.reset();
    }
}
