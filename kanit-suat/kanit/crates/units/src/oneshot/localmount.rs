use async_process::{Command, Stdio};
use async_trait::async_trait;
use log::info;
use nix::unistd::sync;

use kanit_common::error::{Context, ErrorKind, Result, StaticError};
use kanit_unit::{Dependencies, Unit};

use crate::mounts::{is_fs_mounted, parse_mounts};
use crate::oneshot::{Clock, Modules, RootFs};
use crate::unit_name;

pub struct LocalMount;

async fn kill_fs_users(fs: &str) -> Result<()> {
    Command::new("fuser")
        .args(["-KILL", "-k", "-m", fs])
        .spawn()
        .context("failed to spawn fuser")?
        .status()
        .await
        .context("failed to wait fuser")?;

    Ok(())
}

#[async_trait]
impl Unit for LocalMount {
    unit_name!("localmount");

    fn dependencies(&self) -> Dependencies {
        Dependencies::new()
            .need(RootFs.name())
            .after(Clock.name())
            .after(Modules.name())
            .clone()
    }

    async fn start(&mut self) -> Result<()> {
        info!("mounting local filesystems");

        let succ = Command::new("mount")
            .stdout(Stdio::null())
            .args(["-a", "-t", "noproc"])
            .spawn()
            .context("failed to spawn mount")?
            .status()
            .await
            .context("failed to wait on mount")?
            .success();

        if !succ {
            Err(StaticError("failed to mount local filesystems")).kind(ErrorKind::Recoverable)?;
        }

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        sync();

        let mounted = async_fs::read_to_string("/proc/mounts")
            .await
            .context("failed to read mounts")?;
        // loopback
        info!("unmounting loopback");

        for mount in parse_mounts(&mounted)? {
            if !is_fs_mounted(mount.fs_file).await.unwrap_or(false) {
                continue;
            }

            if !mount.fs_file.starts_with("/dev/loop") {
                continue;
            }

            // should already be dead since init should've killed
            kill_fs_users(mount.fs_file).await?;

            Command::new("umount")
                .args(["-d", mount.fs_file])
                .spawn()
                .context("failed to spawn umount")?
                .status()
                .await
                .context("failed to wait umount")?;
        }

        // now everything but network
        info!("unmounting filesystems");
        for mount in parse_mounts(&mounted)? {
            let n = mount.fs_file;

            if !is_fs_mounted(n).await.unwrap_or(false) {
                continue;
            }

            if n == "/"
                || n == "/dev"
                || n == "/sys"
                || n == "/proc"
                || n == "/run"
                || n.starts_with("/dev/")
                || n.starts_with("/sys/")
                || n.starts_with("/proc/")
            {
                continue;
            }

            if mount.fs_mntopts.contains_key("_netdev") {
                continue;
            }

            kill_fs_users(mount.fs_file).await?;

            Command::new("umount")
                .arg(mount.fs_file)
                .spawn()
                .context("failed to spawn umount")?
                .status()
                .await
                .context("failed to wait umount")?;
        }

        Ok(())
    }
}
