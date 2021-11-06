use std::time::Duration;

use tokio::time::sleep;
use zysfs::types as sysfs;
use zysfs::types::tokio::Feature as _;

use crate::cli::Cli;
use crate::counter;
use crate::data::{RaplSampler, RaplSamplers};

#[derive(Clone, Debug)]
pub(super) struct Samplers {
    samplers: Option<RaplSamplers>,
}

impl Samplers {
    // Sleep time between samples, per sampler/zone.
    const INTERVAL: Duration = Duration::from_millis(100);
    // Minimum run time required to get give useful data.
    const RUNTIME: Duration = Duration::from_millis(400);

    pub(super) async fn new(cli: &Cli) -> Self {
        let samplers = if cli.quiet.is_none()
            && (!cli.has_show_args() || cli.show_rapl.is_some())
            && sysfs::intel_rapl::IntelRapl::present().await
        {
            if let Some(s) = RaplSampler::all(Self::INTERVAL).await {
                log::debug!("Starting rapl samplers");
                let mut s = RaplSamplers::from(s);
                s.start().await;
                Some(s)
            } else {
                None
            }
        } else {
            None
        };
        Self { samplers }
    }

    pub(super) async fn stop(&mut self) {
        if let Some(s) = self.samplers.as_mut() {
            s.stop().await;
        }
    }

    pub(super) async fn wait(&self, begin: Duration) {
        if let Some(s) = self.samplers.as_ref() {
            if s.working().await {
                let runtime = counter::delta().await - begin;
                if runtime < Self::RUNTIME {
                    sleep(Self::RUNTIME - runtime).await;
                }
            }
        }
    }

    pub(super) fn into_samplers(self) -> Option<RaplSamplers> { self.samplers }
}
