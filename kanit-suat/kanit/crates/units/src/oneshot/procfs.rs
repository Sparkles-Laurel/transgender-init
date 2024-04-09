use std::path::Path;

use async_trait::async_trait;
use blocking::unblock;
use log::info;
use nix::mount::{mount, MsFlags};

use kanit_common::error::{Context, Result};
use kanit_unit::Unit;

use crate::mounts::try_mount_from_fstab;
use crate::unit_name;

pub struct ProcFs;

#[async_trait]
impl Unit for ProcFs {
    unit_name!("procfs");

    async fn start(&mut self) -> Result<()> {
        // check if proc is already mounted
        if Path::new("/proc/mounts").exists() {
            info!("procfs already mounted");
            return Ok(());
        }

        info!("mounting /proc");

        let path = Path::new("/proc");

        if try_mount_from_fstab(path).await? {
            return Ok(());
        }

        unblock(move || {
            mount(
                Some("none"),
                path,
                Some("proc"),
                MsFlags::MS_NODEV | MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID,
                Some(""),
            )
        })
        .await
        .context("failed to mount procfs")?;

        Ok(())
    }
}
