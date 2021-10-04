use chrono::{Duration, prelude::*};
use log::debug;

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

    fn format(dt: Duration) -> String {
        if let Some(ns) = dt.num_nanoseconds() {
            format!("{} ns", ns)
        } else if let Some(us) = dt.num_microseconds() {
            format!("{} us", us)
        } else {
            format!("{} ms", dt.num_milliseconds())
        }
    }
    
    pub fn end(&mut self, mark: &str) {
        let dt = Utc::now() - self.start;
        debug!("{} took {}", mark, Self::format(dt));
        self.reset();
    }
}
