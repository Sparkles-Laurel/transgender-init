use async_trait::async_trait;
use blocking::unblock;
use log::{info, warn};
use nix::mount::{mount, MsFlags};
use nix::unistd::{access, AccessFlags};

use kanit_common::error::{Context, ErrorKind, Result};
use kanit_executor::join_all;
use kanit_unit::{Dependencies, Unit};

use crate::mounts::{is_fs_mounted, parse_mounts, MountAction, MountEntry};
use crate::oneshot::Clock;
use crate::unit_name;

pub struct RootFs;

async fn remount_entry(entry: MountEntry<'_>) -> Result<()> {
    if is_fs_mounted(entry.fs_file).await? && !entry.mount(MountAction::Remount).await? {
        warn!("failed to remount {}", entry.fs_file);
    }

    Ok(())
}

#[async_trait]
impl Unit for RootFs {
    unit_name!("rootfs");

    fn dependencies(&self) -> Dependencies {
        Dependencies::new().after(Clock.name()).clone()
    }

    async fn start(&mut self) -> Result<()> {
        if access("/", AccessFlags::W_OK).is_ok() {
            return Ok(()); // rootfs already writable
        }

        info!("remounting rootfs as rw");

        unblock(move || {
            mount(
                Some(""), // ignored in remount
                "/",
                Some(""),
                MsFlags::MS_REMOUNT,
                Some("rw"),
            )
        })
        .await
        .context("failed to remount rootfs")?;

        info!("remounting filesystems");

        let fstab = async_fs::read_to_string("/etc/fstab")
            .await
            .context_kind("failed to read fstab", ErrorKind::Recoverable)?;

        // TODO; leak probably bad
        join_all(parse_mounts(fstab.leak())?.into_iter().map(remount_entry)).await;

        Ok(())
    }
}
