use std::path::Path;

use async_trait::async_trait;
use blocking::unblock;
use log::info;
use nix::mount::{mount, MsFlags};

use kanit_common::error::{Context, ErrorKind, Result, StaticError};
use kanit_executor::join_all;
use kanit_unit::{Dependencies, Unit};

use crate::mounts::{is_fs_available, is_fs_mounted, try_mount_from_fstab};
use crate::oneshot::ProcFs;
use crate::unit_name;

pub struct SysFs;

async fn mount_misc_fs<P: AsRef<Path>>(path: P, name: &'static str) -> Result<()> {
    let path = path.as_ref().to_owned();

    if path.exists() && is_fs_available(name).await? && !is_fs_mounted(&path).await? {
        info!("mounting {}", name);

        unblock(move || {
            mount(
                Some("none"),
                &path,
                Some(name),
                MsFlags::MS_NODEV | MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID,
                Some(""),
            )
        })
        .await
        .with_context(move || format!("failed to mount {}", name))?;
    }

    Ok(())
}

impl SysFs {
    async fn mount_sys() -> Result<()> {
        // check if sysfs exists
        if !is_fs_available("sysfs").await? {
            Err(StaticError("failed to mount sysfs")).kind(ErrorKind::Unrecoverable)?;
        };

        // check if its mounted
        let path = Path::new("/sys");

        if is_fs_mounted(path).await? {
            return Ok(());
        }

        // create /sys if it doesn't exist
        if async_fs::metadata(path).await.is_err() {
            async_fs::create_dir(path)
                .await
                .context("failed to create /sys")?;
        }

        info!("mounting /sys");

        // try mount from fstab
        if try_mount_from_fstab(path).await? {
            return Ok(());
        }

        // mount with sysfs fs
        unblock(move || {
            mount(
                Some("none"),
                path,
                Some("sysfs"),
                MsFlags::MS_NODEV | MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID,
                Some(""),
            )
        })
        .await
        .context("failed to mount sysfs")?;

        Ok(())
    }

    async fn mount_misc() -> Result<()> {
        join_all([
            mount_misc_fs("/sys/kernel/security", "securityfs"),
            mount_misc_fs("/sys/kernel/debug", "debugfs"),
            mount_misc_fs("/sys/kernel/config", "configfs"),
            mount_misc_fs("/sys/fs/fuse/connections", "fusectl"),
            mount_misc_fs("/sys/fs/pstore", "pstore"),
        ])
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

        // TODO; SELinux, efivarfs

        Ok(())
    }
}

#[async_trait]
impl Unit for SysFs {
    unit_name!("sysfs");

    fn dependencies(&self) -> Dependencies {
        Dependencies::new().need(ProcFs.name()).clone()
    }

    async fn start(&mut self) -> Result<()> {
        Self::mount_sys().await?;
        Self::mount_misc().await?;

        Ok(())
    }
}
