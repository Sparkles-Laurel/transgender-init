use std::path::Path;

use async_trait::async_trait;
use blocking::unblock;
use log::info;
use nix::mount::{mount, MsFlags};
use nix::sys::stat::Mode;
use nix::unistd::{chown, mkdir, Group, Uid};

use kanit_common::error::{Context, ErrorKind, Result, StaticError};
use kanit_unit::{Dependencies, Unit};

use crate::mounts::try_mount_from_fstab;
use crate::oneshot::ProcFs;
use crate::unit_name;

pub struct Run;

#[async_trait]
impl Unit for Run {
    unit_name!("run");

    fn dependencies(&self) -> Dependencies {
        Dependencies::new().need(ProcFs.name()).clone()
    }

    async fn start(&mut self) -> Result<()> {
        let path = Path::new("/run");

        if !path.exists() {
            Err(StaticError("/run doesn't exist")).kind(ErrorKind::Unrecoverable)?;
        }

        info!("mounting /run");

        if try_mount_from_fstab(path).await? {
            return Ok(());
        }

        unblock(move || {
            mount(
                Some("none"),
                path,
                Some("tmpfs"),
                MsFlags::MS_NODEV | MsFlags::MS_STRICTATIME | MsFlags::MS_NOSUID,
                Some("mode=0755,nr_inodes=500k,size=10%"),
            )
        })
        .await
        .context("failed to mount run")?;

        info!("creating /run/lock");

        let lock = path.join("lock");

        unblock(move || {
            mkdir(&lock, Mode::S_IROTH | Mode::S_IXOTH | Mode::S_IWUSR)?;

            let gid = Group::from_name("uucp")
                .context("failed to get group uucp")?
                .map(|g| g.gid);

            chown(&lock, Some(Uid::from_raw(0)), gid)
                .context("failed to set permissions on /run/lock")?;

            Ok(())
        })
        .await
    }
}
