use async_process::Command;
use async_trait::async_trait;
use log::info;

use kanit_common::error::{Context, ErrorKind, Result, StaticError};
use kanit_unit::{Dependencies, Unit};

use crate::oneshot::{DevFs, SysFs};
use crate::unit_name;

// TODO; write one for udev as well
pub struct MDev;

#[async_trait]
impl Unit for MDev {
    unit_name!("mdev");

    fn dependencies(&self) -> Dependencies {
        Dependencies::new()
            .need(SysFs.name())
            .need(DevFs.name())
            .clone()
    }

    async fn start(&mut self) -> Result<()> {
        info!("initializing mdev");

        async_fs::write("/proc/sys/kernel/hotplug", "/sbin/mdev")
            .await
            .context("failed to initialize mdev for hotplug")?;

        info!("loading hardware for mdev");

        let succ = Command::new("mdev")
            .arg("-s")
            .spawn()
            .context("failed to spawn mdev")?
            .status()
            .await
            .context("failed to wait")?
            .success();

        if !succ {
            Err(StaticError("failed to spawn mdev")).kind(ErrorKind::Recoverable)?;
        }

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        info!("stopping mdev");

        async_fs::write("/proc/sys/kernel/hotplug", "/sbin/mdev")
            .await
            .context_kind("failed to stop mdev", ErrorKind::Recoverable)?;

        Ok(())
    }
}
