use async_process::{Command, Stdio};
use async_trait::async_trait;
use log::info;

use kanit_common::error::{Context, ErrorKind, Result, StaticError};
use kanit_unit::{Dependencies, Unit};

use crate::oneshot::{Clock, LocalMount, RootFs};
use crate::unit_name;

pub struct Swap;

#[async_trait]
impl Unit for Swap {
    unit_name!("swap");

    fn dependencies(&self) -> Dependencies {
        Dependencies::new()
            .need(RootFs.name())
            .after(Clock.name())
            .before(LocalMount.name())
            .clone()
    }

    async fn start(&mut self) -> Result<()> {
        info!("mounting swap");

        let succ = Command::new("swapon")
            .stdout(Stdio::null())
            .arg("-a")
            .spawn()
            .context("failed to spawn swapon")?
            .status()
            .await
            .context("failed to wait on swapon")?
            .success();

        if !succ {
            Err(StaticError("failed to enable swap")).kind(ErrorKind::Recoverable)?;
        }

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        info!("unmounting swap");

        let succ = Command::new("swapoff")
            .stdout(Stdio::null())
            .arg("-a")
            .spawn()
            .context("failed to spawn swapon")?
            .status()
            .await
            .context("failed to wait on swapon")?
            .success();

        if !succ {
            Err(StaticError("failed to disable swap")).kind(ErrorKind::Recoverable)?;
        }

        Ok(())
    }
}
