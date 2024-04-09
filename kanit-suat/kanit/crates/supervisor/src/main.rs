use std::process::ExitCode;

#[cfg(feature = "cli")]
fn main() -> ExitCode {
    kanit_supervisor::handle_cli()
}

#[cfg(not(feature = "cli"))]
fn main() -> ExitCode {
    eprintln!("supervisor compiled without command line");
    ExitCode::FAILURE
}
