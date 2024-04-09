use std::os::unix::prelude::ExitStatusExt;
use std::process::{ExitCode, ExitStatus};

use nix::errno::Errno;
use nix::sys::signal;
use nix::sys::signal::Signal;
use nix::sys::signalfd::{SfdFlags, SigSet, SignalFd};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};

use crate::{spawn, spawn_restart, Supervisor};

pub fn handle_cli() -> ExitCode {
    match Supervisor::from_env() {
        Ok(mut cfg) => {
            let mut mask = SigSet::empty();
            mask.add(signal::SIGCHLD);
            mask.add(signal::SIGTERM);
            mask.thread_block().unwrap();

            let mut sfd = SignalFd::with_flags(&mask, SfdFlags::empty()).unwrap();

            let mut child = spawn(&cfg).expect("spawn child");

            loop {
                match sfd.read_signal() {
                    Ok(Some(sig)) => match Signal::try_from(sig.ssi_signo as i32) {
                        Ok(Signal::SIGCHLD) => {
                            loop {
                                let pid = waitpid(None, Some(WaitPidFlag::WNOHANG));

                                match pid {
                                    Ok(WaitStatus::StillAlive) => break,
                                    Err(Errno::ECHILD) => break,
                                    Err(e) => {
                                        eprintln!("failed to waitpid: {}", e);
                                        return ExitCode::FAILURE;
                                    }
                                    _ => {}
                                }
                            }

                            if let Some(c) =
                                spawn_restart(&mut cfg, ExitStatus::from_raw(sig.ssi_status), true)
                                    .expect("restart child")
                            {
                                child = c;
                            } else {
                                return ExitCode::SUCCESS;
                            }
                        }
                        Ok(Signal::SIGTERM) => {
                            child.kill().expect("kill child");
                            if let Some(attempts) = cfg.restart_attempts {
                                cfg.restart_attempts = Some(attempts + 1);
                            }
                        }
                        _ => {}
                    },
                    Ok(None) => unreachable!(),
                    Err(err) => {
                        eprintln!("{}", err);
                        return ExitCode::FAILURE;
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    }
}
