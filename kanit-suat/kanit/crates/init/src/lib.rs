#[cfg(feature = "timings")]
use std::fs::File;
#[cfg(feature = "timings")]
use std::io::Write;
#[cfg(not(feature = "testing"))]
use std::os::unix::process::CommandExt;
use std::process;
#[cfg(not(feature = "testing"))]
use std::process::Command;
use std::process::ExitCode;
use std::thread::sleep;
use std::time::Duration;
use std::{env, panic};

#[cfg(feature = "timings")]
use log::warn;
#[cfg(not(feature = "testing"))]
use log::LevelFilter;
use log::{error, info};
use nix::sys::reboot::{reboot, set_cad_enabled, RebootMode};
use nix::sys::signal::{kill, SigSet, Signal};
#[cfg(not(feature = "testing"))]
use nix::sys::utsname::uname;
use nix::unistd::{sync, Pid};

#[cfg(feature = "baked-rc")]
use baked_rc::*;
#[cfg(not(feature = "testing"))]
use ev_loop::ev_loop;
use kanit_common::constants;
use kanit_common::error::{Context, Result};
#[cfg(feature = "testing")]
use kanit_diagnostics::tap as kanit_tap;
use kanit_diagnostics::timing as kanit_timing;
#[cfg(not(feature = "testing"))]
use kanit_diagnostics::Colors;
#[cfg(feature = "timings")]
use kanit_timing::Scope;
#[cfg(not(feature = "baked-rc"))]
use rc::*;

#[cfg(feature = "baked-rc")]
mod baked_rc;
#[cfg(not(feature = "testing"))]
mod bsod;
#[cfg(not(feature = "testing"))]
mod ev_loop;
#[cfg(not(feature = "baked-rc"))]
mod rc;

#[cfg(feature = "timings")]
fn write_scope(file: &mut File, scope: &Scope) -> Result<()> {
    let scope_fmt = format!(
        "{} {} {}\n",
        scope.name,
        scope.duration.unwrap_or(Duration::from_secs(0)).as_micros(),
        scope.level
    );

    file.write(scope_fmt.as_bytes())
        .context("failed to write scope")?;
    Ok(())
}

#[cfg(feature = "timings")]
fn write_timing() -> Result<()> {
    let mut file = File::create(constants::KAN_TIMINGS).context("failed to open times file")?;

    for scope in kanit_timing::get_scopes().iter() {
        write_scope(&mut file, scope)?;
    }

    Ok(())
}

fn initialize() -> Result<()> {
    let id = kanit_timing::push_scope("initialize");

    #[cfg(not(feature = "testing"))]
    let platform = uname()
        .map(|u| {
            format!(
                "{} {} ({})",
                u.sysname().to_string_lossy(),
                u.release().to_string_lossy(),
                u.machine().to_string_lossy()
            )
        })
        .unwrap_or_else(|_| "unknown".to_string());

    #[cfg(not(feature = "testing"))]
    println!(
        "== Kanit {}{}{} on {}{}{} ==\n",
        Colors::BrightGreen,
        constants::KAN_VERSION,
        Colors::reset(),
        Colors::BrightMagenta,
        platform,
        Colors::reset()
    );

    let mut set = SigSet::all();
    set.remove(Signal::SIGCHLD);

    set.thread_block().context("failed to block signals")?;

    set_cad_enabled(false).context("failed to ignore CAD")?;

    env::set_var("PATH", constants::KAN_PATH);

    initialize_rc()?;

    kanit_timing::pop_scope(id);

    #[cfg(feature = "timings")]
    if let Err(e) = write_timing() {
        warn!("failed to write timings: {}", e);
    };

    Ok(())
}

pub fn teardown(cmd: Option<RebootMode>) -> Result<()> {
    teardown_rc()?;

    info!("terminating remaining processes");

    kill(Pid::from_raw(-1), Signal::SIGTERM).context("failed to terminate all processes")?;

    sleep(Duration::from_secs(3));

    info!("killing remaining processes");

    kill(Pid::from_raw(-1), Signal::SIGKILL).context("failed to kill all processes")?;

    sync();

    // reboot will always return an error
    if let Some(cmd) = cmd {
        let _ = reboot(cmd);
    }

    Ok(())
}

#[cfg(feature = "testing")]
fn failure_handle() {
    kanit_tap::bail::<&str>(None);
    let _ = reboot(RebootMode::RB_POWER_OFF);
}

#[cfg(not(feature = "testing"))]
fn failure_handle() {
    eprintln!("dropping into emergency shell");

    Command::new("sh").exec();

    eprintln!("failed to drop into emergency shell; good luck o7");

    loop {
        // kernel panic if this loop doesn't exist
        sleep(Duration::from_secs(1));
    }
}

pub fn handle_cli() -> ExitCode {
    if process::id() != 1 {
        eprintln!("init must be ran as PID 1");
        return ExitCode::FAILURE;
    }

    #[cfg(feature = "testing")]
    panic::set_hook(Box::new(|info| {
        kanit_tap::bail(info.payload().downcast_ref::<&str>());
        let _ = reboot(RebootMode::RB_POWER_OFF);
    }));

    #[cfg(not(feature = "testing"))]
    panic::set_hook(Box::new(|info| {
        error!("panic detected; tearing down");
        let _ = teardown(None); // attempt a teardown
        bsod::bsod(info);
    }));

    kanit_diagnostics::tap::header();

    #[cfg(not(feature = "testing"))]
    if let Err(e) = kanit_diagnostics::InitializationLogger::init(LevelFilter::Debug) {
        eprintln!("{}", e);
    }

    kanit_timing::register();

    if let Err(e) = initialize() {
        error!("failed to initialize: {}", e);
        failure_handle();
    }

    #[cfg(not(feature = "testing"))] // no way to test yet
    if let Err(e) = ev_loop() {
        error!("event loop failed: {}", e);
        failure_handle();
    }

    println!("teardown");

    #[cfg(feature = "testing")]
    teardown(Some(RebootMode::RB_POWER_OFF)).expect("failed to unwrap");

    ExitCode::SUCCESS
}
