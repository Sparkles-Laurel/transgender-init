use std::os::unix::fs::FileTypeExt;
use std::path::Path;

use async_fs::unix::symlink;
use async_trait::async_trait;
use blocking::unblock;
use libc::dev_t;
use log::info;
use nix::mount::{mount, MsFlags};
use nix::sys::stat::{makedev, mknod, Mode, SFlag};
use nix::unistd::mkdir;

use kanit_common::error::{Context, ErrorKind, Result, StaticError};
use kanit_executor::join_all;
use kanit_unit::{Dependencies, Unit};

use crate::mounts::{
    is_fs_available, is_fs_mounted, try_mount_from_fstab, try_mount_from_fstab_action, MountAction,
};
use crate::oneshot::MDev;
use crate::unit_name;

pub struct DevFs;

async fn character_device<P: AsRef<Path>>(path: P) -> bool {
    async_fs::metadata(path)
        .await
        .ok()
        .map(|m| m.file_type().is_char_device())
        .unwrap_or(false)
}

async fn exists<P: AsRef<Path>>(path: P) -> bool {
    async_fs::metadata(path).await.is_ok()
}

async fn mount_opt(
    path: &'static str,
    name: &'static str,
    mode: Mode,
    flags: MsFlags,
    opts: &'static str,
) -> Result<()> {
    if is_fs_available(name).await? && !is_fs_mounted(&path).await? {
        if !exists(path).await {
            unblock(move || mkdir(path, mode))
                .await
                .context("failed to make directory")?;
        }

        info!("mounting {}", path);

        if try_mount_from_fstab(&path).await? {
            return Ok(());
        }

        unblock(move || {
            mount(
                Some("none"),
                path,
                Some(name),
                flags | MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID,
                Some(opts),
            )
        })
        .await
        .with_context(move || format!("failed to mount {}", name))?;
    }

    Ok(())
}

async fn dev_device(path: &'static str, kind: SFlag, perm: Mode, dev: dev_t) -> Result<()> {
    if character_device(path).await {
        return Ok(());
    }

    unblock(move || mknod(path, kind, perm, dev))
        .await
        .with_context(move || format!("failed to create {}", path))
}

async fn sym(src: &'static str, dst: &'static str) -> Result<()> {
    if !exists(dst).await {
        symlink(src, dst)
            .await
            .with_context(move || format!("failed to link {}", dst))?;
    }

    Ok(())
}

impl DevFs {
    async fn mount_dev() -> Result<()> {
        let path = Path::new("/dev");

        let mut opts = MsFlags::MS_NOSUID;
        let mut action = MountAction::Mount;

        if is_fs_mounted(path).await? {
            info!("remounting devfs");
            opts |= MsFlags::MS_REMOUNT;
            action = MountAction::Remount;
        } else {
            info!("mounting devfs");
        }

        if try_mount_from_fstab_action(path, action).await? {
            return Ok(());
        }

        let fs = if is_fs_available("devtmpfs").await? {
            "devtmpfs"
        } else if is_fs_available("tmpfs").await? {
            "tmpfs"
        } else {
            Err(StaticError(
                "devtmpfs, tmpfs, nor fstab entry available, /dev will not be mounted",
            ))
            .kind(ErrorKind::Recoverable)?
        };

        unblock(move || mount(Some("none"), path, Some(fs), opts, Some("")))
            .await
            .context("failed to mount devfs")?;

        Ok(())
    }

    async fn populate_dev() -> Result<()> {
        // create dev devices if they don't exist

        join_all([
            dev_device(
                "/dev/console",
                SFlag::S_IFCHR,
                Mode::S_IRUSR | Mode::S_IWUSR,
                makedev(5, 1),
            ),
            dev_device(
                "/dev/tty1",
                SFlag::S_IFCHR,
                Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IWGRP,
                makedev(4, 1),
            ),
            dev_device(
                "/dev/tty",
                SFlag::S_IFCHR,
                Mode::S_IRUSR
                    | Mode::S_IWUSR
                    | Mode::S_IRGRP
                    | Mode::S_IWGRP
                    | Mode::S_IROTH
                    | Mode::S_IWOTH,
                makedev(4, 1),
            ),
            dev_device(
                "/dev/null",
                SFlag::S_IFCHR,
                Mode::S_IRUSR
                    | Mode::S_IWUSR
                    | Mode::S_IRGRP
                    | Mode::S_IWGRP
                    | Mode::S_IROTH
                    | Mode::S_IWOTH,
                makedev(1, 3),
            ),
            dev_device(
                "/dev/kmsg",
                SFlag::S_IFCHR,
                Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IWGRP,
                makedev(1, 11),
            ),
        ])
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

        join_all([
            sym("/proc/self/fd", "/dev/fd"),
            sym("/proc/self/fd/0", "/dev/stdin"),
            sym("/proc/self/fd/1", "/dev/stdout"),
            sym("/proc/self/fd/2", "/dev/stderr"),
        ])
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

        join_all([
            mount_opt(
                "/dev/mqueue",
                "mqueue",
                Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO | Mode::S_ISVTX,
                MsFlags::MS_NODEV,
                "",
            ),
            mount_opt(
                "/dev/pts",
                "devpts",
                Mode::S_IRWXU | Mode::S_IRGRP | Mode::S_IXGRP | Mode::S_IROTH | Mode::S_IXOTH,
                MsFlags::empty(),
                "gid=5,mode=0620",
            ),
            mount_opt(
                "/dev/shm",
                "tmpfs",
                Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO | Mode::S_ISVTX,
                MsFlags::MS_NODEV,
                "",
            ),
        ])
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

        if Path::new("/proc/kcore").exists() {
            symlink("/proc/kcore", "/dev/core")
                .await
                .context("failed to link kcore")?;
        }

        Ok(())
    }
}

#[async_trait]
impl Unit for DevFs {
    unit_name!("devfs");

    fn dependencies(&self) -> Dependencies {
        Dependencies::new().before(MDev.name()).clone()
    }

    async fn start(&mut self) -> Result<()> {
        Self::mount_dev().await?;
        Self::populate_dev().await?;

        Ok(())
    }
}
