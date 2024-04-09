use async_process::{Command, Stdio};
use async_trait::async_trait;
use log::info;

use kanit_common::error::{Context, ErrorKind, Result, StaticError};
use kanit_unit::{Dependencies, Unit};

use crate::oneshot::Clock;
use crate::unit_name;

pub struct Sysctl;

#[async_trait]
impl Unit for Sysctl {
    unit_name!("sysctl");

    fn dependencies(&self) -> Dependencies {
        Dependencies::new().after(Clock.name()).clone()
    }

    async fn start(&mut self) -> Result<()> {
        info!("loading sysctl");

        let succ = Command::new("sysctl")
            .stdout(Stdio::null())
            .args(["-q", "--system"])
            .spawn()
            .context("failed to spawn sysctl")?
            .status()
            .await
            .context("failed to wait on sysctl")?
            .success();

        if !succ {
            Err(StaticError("failed load sysctl")).kind(ErrorKind::Recoverable)?;
        }

        Ok(())
    }
}
