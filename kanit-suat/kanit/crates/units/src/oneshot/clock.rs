use std::path::Path;

use async_process::{Command, Stdio};
use async_trait::async_trait;
use futures_lite::StreamExt;
use log::{info, warn};

use kanit_common::error::{Context, ErrorKind, Result, StaticError};
use kanit_unit::{Dependencies, Unit};

use crate::oneshot::Modules;
use crate::unit_name;

pub struct Clock;

fn check_rtc(name: &Path) -> bool {
    let f_str = name.to_string_lossy();

    if f_str == "/dev/rtc"
        || f_str.starts_with("/dev/rtc")
            && f_str
                .chars()
                .nth(8)
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
    {
        return true;
    }

    false
}

impl Clock {
    async fn rtc_exists() -> Result<bool> {
        Ok(async_fs::read_dir("/dev")
            .await
            .context("failed to open /dev")?
            .filter_map(|res| res.map(|e| e.path()).ok())
            .find(|p| check_rtc(p))
            .await
            .is_some())
    }
}

const RTC_MODS: [&str; 3] = ["rtc-cmos", "rtc", "genrtc"];

#[async_trait]
impl Unit for Clock {
    unit_name!("clock");

    fn dependencies(&self) -> Dependencies {
        Dependencies::new().want(Modules.name()).clone()
    }

    async fn start(&mut self) -> Result<()> {
        info!("setting time with hardware clock");

        if !Self::rtc_exists().await? {
            let mut loaded = false;

            for module in RTC_MODS {
                let succ = Command::new("modprobe")
                    .args(["-q", module])
                    .spawn()
                    .context("failed to spawn modprobe")?
                    .status()
                    .await
                    .context("failed to wait")?
                    .success();

                if succ && Self::rtc_exists().await? {
                    warn!("module {} should be built in or configured to load", module);
                    loaded = true;
                    break;
                }
            }

            if !loaded {
                Err(StaticError("failed to set hardware clock")).kind(ErrorKind::Recoverable)?;
            }
        }

        // todo; UTC

        let succ = Command::new("hwclock")
            .args(["--systz", "--localtime"])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .context("failed to spawn hwclock")?
            .output()
            .await
            .context("failed to wait")?
            .stderr
            .is_empty();

        if !succ {
            warn!("failed to set system timezone");
        }

        let succ = Command::new("hwclock")
            .args(["--hctosys", "--localtime"])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .context("failed to spawn hwclock")?
            .output()
            .await
            .context("failed to wait")?
            .stderr
            .is_empty();

        if !succ {
            warn!("failed to set system time");
        }

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        info!("setting hardware clock with system time");

        let succ = Command::new("hwclock")
            .args(["--systohc", "--localtime"])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .context("failed to spawn hwclock")?
            .output()
            .await
            .context("failed to wait")?
            .stderr
            .is_empty();

        if !succ {
            warn!("failed to set hardware clock");
        }

        Ok(())
    }
}
