// me when the binary is plural

use std::env::args;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let name = args().next().expect("first arg");
    let name_as_path = Path::new(&name);
    let bin_name = name_as_path.file_name().expect("file name").to_str();

    match bin_name {
        Some("init") => kanit_init::handle_cli(),
        #[cfg(feature = "cli")]
        Some("kanit") => kanit_cli::handle_cli(),
        Some("kanit-supervisor") => kanit_supervisor::handle_cli(),
        _ => {
            eprintln!("was unable to locate `{}`", bin_name.unwrap_or(""));
            ExitCode::FAILURE
        }
    }
}
