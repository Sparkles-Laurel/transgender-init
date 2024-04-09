use std::collections::HashSet;

use async_process::{Command, Stdio};
use async_trait::async_trait;
use blocking::unblock;
use futures_lite::stream::iter;
use futures_lite::StreamExt;
use log::info;
use walkdir::WalkDir;

use kanit_common::error::{Context, Result};
use kanit_unit::{Dependencies, Unit};

use crate::oneshot::{DevFs, SysFs};
use crate::unit_name;

pub struct HwDrivers;

impl HwDrivers {
    async fn load_modules() -> Result<()> {
        // todo; better walker
        let modules = iter(
            unblock(move || {
                WalkDir::new("/sys")
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file() && e.file_name() == "modalias")
            })
            .await,
        )
        .then(|e| async { async_fs::read_to_string(e.into_path()).await })
        .filter_map(|e| e.ok())
        .collect::<HashSet<String>>()
        .await;

        // we don't care about status
        Command::new("modprobe")
            .stderr(Stdio::null())
            .args(["-b", "-a"])
            .args(modules)
            .spawn()
            .context("failed to spawn modprobe")?
            .status()
            .await
            .context("failed to wait")?;

        Ok(())
    }
}

#[async_trait]
impl Unit for HwDrivers {
    unit_name!("hwdrivers");

    fn dependencies(&self) -> Dependencies {
        Dependencies::new()
            .need(DevFs.name())
            .need(SysFs.name())
            .clone()
    }

    async fn start(&mut self) -> Result<()> {
        info!("loading modules for hardware");

        Self::load_modules().await?;
        Self::load_modules().await?;

        Ok(())
    }
}
