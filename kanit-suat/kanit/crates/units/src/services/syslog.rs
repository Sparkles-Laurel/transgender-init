use async_trait::async_trait;
use blocking::unblock;
use log::info;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

use kanit_common::error::Result;
use kanit_supervisor::RestartPolicy;
use kanit_unit::supervisor::SupervisorBuilder;
use kanit_unit::{Dependencies, Unit};

use crate::oneshot::{Clock, Hostname, LocalMount};
use crate::unit_name;

pub struct Syslog {
    pid: u32,
}

impl Syslog {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { pid: 0 }
    }
}

#[async_trait]
impl Unit for Syslog {
    unit_name!("syslog");

    fn dependencies(&self) -> Dependencies {
        Dependencies::new()
            .need(Clock.name())
            .need(Hostname.name())
            .need(LocalMount.name())
            .clone()
    }

    async fn start(&mut self) -> Result<()> {
        info!("starting syslog");

        let child = SupervisorBuilder::new("syslogd", [])
            .restart_policy(RestartPolicy::OnFailure)
            .spawn()?;

        self.pid = child.id();

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        let pid = self.pid;
        let _ = unblock(move || kill(Pid::from_raw(pid as i32), Signal::SIGKILL)).await;

        Ok(())
    }
}
