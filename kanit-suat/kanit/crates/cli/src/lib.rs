use std::process::ExitCode;

#[cfg(feature = "service")]
use flags::ServiceCmd;
use flags::{Kanit, KanitCmd};

#[cfg(feature = "blame")]
mod blame;
mod flags;
#[cfg(feature = "service")]
mod service;
mod teardown;

pub fn handle_cli() -> ExitCode {
    let res = match Kanit::from_env() {
        Ok(app) => match app.subcommand {
            KanitCmd::Poweroff(opts) => teardown::teardown("poweroff", opts.force),
            KanitCmd::Reboot(opts) => teardown::teardown("reboot", opts.force),
            KanitCmd::Halt(opts) => teardown::teardown("halt", opts.force),
            KanitCmd::Kexec(opts) => teardown::teardown("kexec", opts.force),
            #[cfg(feature = "blame")]
            KanitCmd::Blame(opts) => blame::blame(opts),
            #[cfg(not(feature = "blame"))]
            KanitCmd::Blame(_) => {
                eprintln!("kanit compiled without blame");
                return ExitCode::FAILURE;
            }
            #[cfg(feature = "service")]
            KanitCmd::Service(svc) => match svc.subcommand {
                ServiceCmd::Enable(opts) => service::enable(opts),
                ServiceCmd::Disable(opts) => service::disable(opts),
                ServiceCmd::List(opts) => service::list(opts),
            },
            #[cfg(not(feature = "service"))]
            KanitCmd::Service(_) => {
                eprintln!("kanit compiled without service");
                return ExitCode::FAILURE;
            }
        },
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = res {
        eprintln!("{}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
