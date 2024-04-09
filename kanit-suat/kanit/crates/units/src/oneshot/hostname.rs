use async_trait::async_trait;
use blocking::unblock;
use nix::unistd::sethostname;

use kanit_common::error::{Context, ErrorKind, Result};
use kanit_unit::{Dependencies, Unit};

use crate::oneshot::Clock;
use crate::unit_name;

pub struct Hostname;

#[async_trait]
impl Unit for Hostname {
    unit_name!("hostname");

    fn dependencies(&self) -> Dependencies {
        Dependencies::new().after(Clock.name()).clone()
    }

    async fn start(&mut self) -> Result<()> {
        let hostname = async_fs::read_to_string("/etc/hostname")
            .await
            .unwrap_or_else(|_| "homosexual".to_string());

        unblock(move || sethostname(hostname.trim_start().trim_end()))
            .await
            .context_kind("failed to set hostname", ErrorKind::Recoverable)?;

        Ok(())
    }
}
