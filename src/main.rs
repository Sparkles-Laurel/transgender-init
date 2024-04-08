// main.rs - Entry point for Transgender (init system)
// Copyright (c) 2023-2024 Kıvılcım Leyla Öztürk.
// Licensed under the MIT License <http://opensource.org/licenses/MIT>

// required to deal with the warnings about uppercase pseudoglobal variable names
#![allow(non_snake_case)]

// Used for colors
use colored::*;
// for now we only need to check if we are PID 1.
// after that we are going to exit and cause a kernel panic.
// import the necessary libraries for self PID checking.
use std::process;

// also, cutie, don't forget to load /etc/os-release
// and print the OS name and version on the screen.
// we are going to use the os-release crate for that.
// import the necessary libraries for os-release.
use os_release::OsRelease;

mod init_driver;
mod units;
use init_driver::{InitArgs, InitDriver};

/// Entry point for the init daemon
fn main() {
    // check if we are PID 1.
    if process::id() == 1 {
        eprintln!("Nice try, cutie. But you are not PID 1.");
        process::exit(23);
    } else {
        // okay so we to load os-release first.
        let os_release = OsRelease::new().unwrap();
        // also we need the color of the os name.
        // so read the os color from /etc/os-release.

        let _INIT_NAME: String = format!(
            "{}{}{}{}{}",
            "Tr".truecolor(0x74, 0xc7, 0xec), // Blue
            "an".truecolor(0xEB, 0xA0, 0xAC), // Pink
            "sge".truecolor(0xFF, 0xFF, 0xFF), // White
            "nd".truecolor(0xEB, 0xA0, 0xAC), // Pink
            "er".truecolor(0x74, 0xc7, 0xEC) // Blue
        );

        let _OS_NAME: String = format!(
            "\x1B[{}m{}",
            os_release.extra["ANSI_COLOR"], os_release.pretty_name
        ); // Colorise os_name according to /etc/os-release
        // now print the message "Transgender is starting {os_name} {kernel_version}..."
        eprintln!("{} is starting {}...", _INIT_NAME, _OS_NAME);

        // create a new init driver
        let mut init_driver = InitDriver::new(InitArgs {
            target_name: "default.target".to_string(),
            rootfs: "/".to_string(),
            cmdline: "".to_string(),
            init_corpus: "/etc/suat/init/corpus.o".to_string(),
            kernel_args: vec![],
        });

        // abnormally exit and cause a kernel panic
        // TODO: actually invoke the init driver.
        process::exit(31);
    }
}
