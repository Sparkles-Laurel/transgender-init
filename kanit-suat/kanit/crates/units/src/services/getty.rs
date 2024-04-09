use std::sync::OnceLock;

use async_trait::async_trait;
use blocking::unblock;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

use kanit_common::error::Result;
use kanit_supervisor::RestartPolicy;
use kanit_unit::supervisor::SupervisorBuilder;
use kanit_unit::{Unit, UnitName};

pub struct GeTTY {
    name: String,
    pid: u32,
    tty: &'static str,
    serial: bool,
}

impl GeTTY {
    pub fn new(tty: &'static str, serial: bool) -> Self {
        Self {
            name: format!("getty@{}", tty),
            pid: 0,
            tty,
            serial,
        }
    }
}

#[async_trait]
impl Unit for GeTTY {
    fn name(&self) -> UnitName {
        static NAME: OnceLock<UnitName> = OnceLock::new();

        NAME.get_or_init(|| UnitName::from(self.name.clone()))
            .clone()
    }

    async fn start(&mut self) -> Result<()> {
        let child = if self.serial {
            SupervisorBuilder::new("getty", ["-L", "0", self.tty, "vt100"])
        } else {
            SupervisorBuilder::new("getty", ["38400", self.tty])
        }
        .restart_policy(RestartPolicy::Always)
        .restart_delay(2)
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
