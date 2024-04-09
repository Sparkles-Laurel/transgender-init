use async_fs::File;
use async_signal::{Signal, Signals};
use futures_lite::{AsyncReadExt, StreamExt};
use nix::errno::Errno;
use nix::sys::reboot::RebootMode;
use nix::sys::stat::Mode;
use nix::unistd::mkfifo;

use kanit_common::constants;
use kanit_common::error::{Context, Result};
use kanit_executor::{block, spawn};

use crate::{event_rc, teardown};

async fn pipe_handler(data: Vec<u8>) -> Result<()> {
    if data.starts_with(b"halt") {
        teardown(Some(RebootMode::RB_HALT_SYSTEM))?;
    } else if data.starts_with(b"poweroff") {
        teardown(Some(RebootMode::RB_POWER_OFF))?;
    } else if data.starts_with(b"reboot") {
        teardown(Some(RebootMode::RB_AUTOBOOT))?;
    } else if data.starts_with(b"kexec") {
        teardown(Some(RebootMode::RB_KEXEC))?;
    } else {
        event_rc(data).await?;
    }

    Ok(())
}

async fn listen_signal() -> Result<()> {
    let mut signals = Signals::new([Signal::Int, Signal::Term, Signal::Child])
        .context("failed to register signals")?;

    while let Some(signal) = signals.next().await {
        let signal = if let Ok(signal) = signal {
            signal
        } else {
            continue;
        };

        println!("*boop* {}", signal as i32);
    }

    Ok(())
}

async fn listen_file() -> Result<()> {
    mkfifo(constants::KAN_PIPE, Mode::S_IRUSR | Mode::S_IWUSR).context("failed to create pipe")?;

    loop {
        let mut buff = vec![0u8; 512];

        let mut file = match File::open(constants::KAN_PIPE).await {
            Ok(f) => f,
            Err(e) => match e.raw_os_error() {
                None => return Err(e.into()),
                Some(n) => {
                    if Errno::from_raw(n) == Errno::EINTR {
                        continue;
                    } else {
                        return Err(e.into());
                    }
                }
            },
        };

        file.read(&mut buff).await.context("failed to read pipe")?;

        spawn(pipe_handler(buff)).detach();
    }
}

async fn inner_ev_loop() -> Result<()> {
    let handles = [spawn(listen_signal()), spawn(listen_file())];

    for handle in handles {
        handle.await?;
    }

    Ok(())
}

pub fn ev_loop() -> Result<()> {
    block(inner_ev_loop())?;

    Ok(())
}
