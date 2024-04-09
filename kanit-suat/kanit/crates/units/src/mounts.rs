use std::collections::HashMap;
use std::path::Path;

use async_process::Command;

use kanit_common::error::{Context, Result};

pub async fn is_fs_available(fs: &str) -> Result<bool> {
    let filesystems = async_fs::read_to_string("/proc/filesystems")
        .await
        .context("failed to read filesystems")?;

    // prepend tab as the format is `nodev <fs>` or `   <fs>`
    // TODO; maybe something a bit more elegant
    Ok(filesystems.contains(&format!("\t{}", fs)))
}

pub async fn is_fs_mounted<P: AsRef<Path>>(path: P) -> Result<bool> {
    let mounted = async_fs::read_to_string("/proc/mounts")
        .await
        .context("failed to read mounts: {}")?;

    let path = path.as_ref().to_string_lossy();

    Ok(parse_mounts(&mounted)?.iter().any(|m| m.fs_file == path))
}

pub async fn try_mount_from_fstab<P: AsRef<Path>>(path: P) -> Result<bool> {
    try_mount_from_fstab_action(path, MountAction::Mount).await
}

pub async fn try_mount_from_fstab_action<P: AsRef<Path>>(
    path: P,
    action: MountAction,
) -> Result<bool> {
    let fstab = async_fs::read_to_string("/etc/fstab")
        .await
        .context("failed to read fstab")?;

    let path = path.as_ref().to_string_lossy();

    if let Some(entry) = parse_mounts(&fstab)?.iter().find(|m| m.fs_file == path) {
        Ok(entry.mount(action).await?)
    } else {
        Ok(false)
    }
}

pub struct MountEntry<'a> {
    pub fs_spec: &'a str,
    pub fs_file: &'a str,
    pub fs_vfstype: &'a str,
    pub fs_mntopts: HashMap<&'a str, Option<&'a str>>,
    pub fs_freq: u8,
    pub fs_passno: u8,
}

pub enum MountAction {
    Mount,
    Remount,
}

impl<'a> MountEntry<'a> {
    pub fn parse_single_mount(line: &'a str) -> Result<Self> {
        let mut parts = line.split_whitespace();

        let fs_spec = parts.next().context("expected `fs_spec`")?;

        let fs_file = parts.next().context("expected `fs_file`")?;

        let fs_vfstype = parts.next().context("expected `fs_vfstype`")?;

        let fs_mntopts = parts.next().context("expected `fs_mntopts`")?;

        let fs_mntopts: HashMap<&str, Option<&str>> = fs_mntopts
            .split(',')
            .map(|s| {
                let mut split = s.splitn(2, '=');
                // unwrap: split will always have at least 1
                let opt = split.next().unwrap();
                let val = split.next();

                (opt, val)
            })
            .collect();

        let fs_freq = parts
            .next()
            .context("expected `fs_freq`")?
            .parse::<u8>()
            .context("failed to parse `fs_freq`")?;

        let fs_passno = parts
            .next()
            .context("expected `fs_passno`")?
            .parse::<u8>()
            .context("failed to parse `fs_passno`")?;

        Ok(Self {
            fs_spec,
            fs_file,
            fs_vfstype,
            fs_mntopts,
            fs_freq,
            fs_passno,
        })
    }

    pub async fn mount(&self, action: MountAction) -> Result<bool> {
        let mut opts = self
            .fs_mntopts
            .iter()
            .map(|(k, v)| {
                if let Some(v) = v {
                    format!("{}={}", k, v)
                } else {
                    k.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join(",");

        if let MountAction::Remount = action {
            if opts.is_empty() {
                opts.push_str("remount");
            } else {
                opts.push_str(",remount");
            }
        }

        Ok(Command::new("mount")
            .arg("-o")
            .arg(opts)
            .arg("-t")
            .arg(self.fs_vfstype)
            .arg(self.fs_spec)
            .arg(self.fs_file)
            .spawn()
            .context("failed to start mount")?
            .status()
            .await
            .context("failed to wait on mount")?
            .success())
    }
}

pub fn parse_mounts(lines: &str) -> Result<Vec<MountEntry>> {
    lines
        .lines()
        .filter_map(|line| {
            if line.starts_with('#') || line.is_empty() {
                return None;
            }
            Some(MountEntry::parse_single_mount(line))
        })
        .collect()
}
