use std::path::Path;
use std::process::Command;

use kanit_common::error::{Context, ErrorKind, Result, StaticError};

pub fn initialize_rc() -> Result<()> {
    if !Path::new("/etc/rc.start").exists() {
        Err(StaticError("failed to find a start script")).kind(ErrorKind::Unrecoverable)?;
    }

    Command::new("/etc/rc.start")
        .spawn()
        .context("failed to start rc.start")?;

    Ok(())
}

pub fn teardown_rc() -> Result<()> {
    if !Path::new("/etc/rc.stop").exists() {
        Err(StaticError("failed to find a stop script")).kind(ErrorKind::Unrecoverable)?;
    }

    Command::new("/etc/rc.stop")
        .spawn()
        .context("failed to start rc.start")?;

    Ok(())
}

#[cfg(not(feature = "testing"))]
pub async fn event_rc(ev: Vec<u8>) -> Result<()> {
    if !Path::new("/etc/rc.event").exists() {
        return Ok(());
    }

    Command::new("/etc/rc.event")
        .arg(String::from_utf8_lossy(&ev).to_string())
        .spawn()
        .context_kind("failed to start rc.event", ErrorKind::Recoverable)?;

    Ok(())
}
