use std::time::{Duration, Instant};

use tokio::time::sleep;

use crate::cli::Cli;
use crate::sysfs::intel_rapl as rapl;

#[derive(Clone, Debug)]
pub(in crate::cli) struct Samplers {
    begin: Instant,
    samplers: Option<rapl::Samplers>,
}

impl Samplers {
    // Sleep time between samples, per sampler/zone.
    const INTERVAL: Duration = Duration::from_millis(100);
    // Minimum run time required to get give useful data.
    const RUNTIME: Duration = Duration::from_millis(400);

    pub(in crate::cli) async fn start(cli: &Cli) -> Self {
        let sample = cli.quiet.is_some() && (!cli.has_show_args() || cli.show_rapl.is_some());
        let samplers = if sample {
            let mut s = rapl::Sampler::all(Self::INTERVAL).await;
            if s.count() > 0 {
                log::debug!("Starting rapl samplers");
                s.start().await;
                Some(s)
            } else {
                None
            }
        } else {
            None
        };
        let begin = Instant::now();
        Self { begin, samplers }
    }

    pub(in crate::cli) async fn stop(&mut self) {
        if let Some(s) = self.samplers.as_mut() {
            s.stop().await;
        }
    }

    pub(in crate::cli) async fn wait(&self) {
        if let Some(s) = self.samplers.as_ref() {
            if s.working().await {
                let runtime = Instant::now() - self.begin;
                if runtime < Self::RUNTIME {
                    sleep(Self::RUNTIME - runtime).await;
                }
            }
        }
    }

    pub(in crate::cli) fn into_samplers(self) -> Option<rapl::Samplers> {
        self.samplers
    }
}
