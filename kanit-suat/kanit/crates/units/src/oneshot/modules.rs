use std::os::unix::ffi::OsStrExt;

use async_process::Command;
use async_trait::async_trait;
use blocking::unblock;
use futures_lite::stream::iter;
use futures_lite::StreamExt;
use log::{info, warn};
use walkdir::WalkDir;

use kanit_common::error::{Context, Result};
use kanit_unit::Unit;

use crate::unit_name;

pub struct Modules;

const LOADED_FOLDERS: [&str; 3] = [
    "/etc/modules-load.d/",
    "/run/modules-load./",
    "/usr/lib/modules-load.d/",
];

#[async_trait]
impl Unit for Modules {
    unit_name!("modules");

    async fn start(&mut self) -> Result<()> {
        if async_fs::metadata("/proc/modules").await.is_err() {
            return Ok(());
        }

        for dir in LOADED_FOLDERS.iter() {
            let modules = iter(
                unblock(move || {
                    WalkDir::new(dir)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|e| {
                            e.file_type().is_file() && e.file_name().as_bytes().ends_with(b".conf")
                        })
                })
                .await,
            )
            .then(|e| async { async_fs::read_to_string(e.into_path()).await })
            .filter_map(|e| e.ok())
            .collect::<Vec<String>>()
            .await
            .into_iter()
            .flat_map(|e| {
                e.lines()
                    .filter(|l| !l.starts_with('#') && !l.starts_with(';') && !l.is_empty())
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            });

            for module in modules {
                info!("loading module {}", module);

                let succ = Command::new("modprobe")
                    .args(["-b", "-a", "-v", &module])
                    .spawn()
                    .context("failed to spawn modprobe")?
                    .status()
                    .await
                    .context("failed to wait")?
                    .success();

                if !succ {
                    warn!("failed to load module {}", module);
                }
            }
        }

        Ok(())
    }
}
