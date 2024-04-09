use std::fs::OpenOptions;
use std::os::unix::fs::chroot;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread::sleep;
use std::time::Duration;

use nix::unistd::{Group, User};

use kanit_common::error::{Context, Result};

use crate::flags::{RestartPolicy, Supervisor};

pub fn spawn_restart(
    cfg: &mut Supervisor,
    status: ExitStatus,
    delay: bool,
) -> Result<Option<Child>> {
    match cfg.restart_policy.unwrap_or(RestartPolicy::Never) {
        RestartPolicy::Never => return Ok(None),
        RestartPolicy::OnFailure if status.success() => return Ok(None),
        RestartPolicy::OnSuccess if !status.success() => return Ok(None),
        _ => {}
    }

    if let Some(attempts) = cfg.restart_attempts {
        if attempts == 0 {
            return Ok(None);
        } else {
            cfg.restart_attempts = Some(attempts - 1);
        }
    }

    if let Some(delay_sec) = cfg.restart_delay {
        if delay {
            sleep(Duration::from_secs(delay_sec))
        }
    }

    spawn(cfg).map(Some)
}

pub fn spawn(cfg: &Supervisor) -> Result<Child> {
    let mut cmd = Command::new(&cfg.cmd);

    cmd.args(&cfg.args);
    cmd.envs(cfg.env.iter().filter_map(|pair| pair.split_once('=')));

    if let Some(ref dir) = cfg.pwd {
        cmd.current_dir(dir);
    }

    if let Some(ref dir) = cfg.root {
        let dir = dir.clone();

        // SAFETY: we only call async-signal-safe functions (chroot)
        unsafe {
            cmd.pre_exec(move || chroot(&dir));
        }
    }

    if let Some(ref user) = cfg.user {
        let uid = user
            .parse::<u32>()
            .map(Some)
            .unwrap_or_else(|_| {
                User::from_name(user)
                    .map(|u| u.map(|u| u.uid.as_raw()))
                    .unwrap_or(None)
            })
            .context("failed to parse/locate user")?;

        cmd.uid(uid);
    }

    if let Some(ref group) = cfg.group {
        let gid = group
            .parse::<u32>()
            .map(Some)
            .unwrap_or_else(|_| {
                Group::from_name(group)
                    .map(|g| g.map(|g| g.gid.as_raw()))
                    .unwrap_or(None)
            })
            .context("failed to parse/locate group")?;

        cmd.gid(gid);
    }

    if let Some(ref stdout) = cfg.stdout {
        let f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(stdout)
            .context("failed to open stdout")?;

        cmd.stdout(f);
    } else {
        cmd.stdout(Stdio::null());
    }

    if let Some(ref stderr) = cfg.stderr {
        let f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(stderr)
            .context("failed to open stderr")?;

        cmd.stderr(f);
    } else {
        cmd.stderr(Stdio::null());
    }

    cmd.spawn().context("failed to spawn child")
}
