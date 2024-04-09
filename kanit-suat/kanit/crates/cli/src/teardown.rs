use std::fs::write;

use nix::sys::reboot::{reboot, RebootMode};
use nix::unistd::getuid;

use kanit_common::constants::KAN_PIPE;
use kanit_common::error::{Context, Result, StaticError};

pub fn teardown(cmd: &str, force: bool) -> Result<()> {
    if !getuid().is_root() {
        Err(StaticError("operation not permitted"))?;
    }

    if force {
        match cmd {
            "poweroff" => reboot(RebootMode::RB_POWER_OFF),
            "reboot" => reboot(RebootMode::RB_AUTOBOOT),
            "halt" => reboot(RebootMode::RB_HALT_SYSTEM),
            "kexec" => reboot(RebootMode::RB_KEXEC),
            _ => unreachable!(),
        }
        .context("Failed to reboot")?;
    }

    write(KAN_PIPE, cmd).context("failed to write to pipe")?;

    Ok(())
}
