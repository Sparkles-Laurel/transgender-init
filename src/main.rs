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

// immport the necessary functions for reading files.
use std::fs::File;
use std::io::prelude::*;

// import the functions for delaying the execution.
use std::thread;
use std::time::Duration;

fn main() {
    // check if we are PID 1.
    if process::id() == 1 {
        eprintln!("Nice try, cutie. But you are not PID 1.");
    } else {
        // okay so we to load os-release first.
        let os_release = OsRelease::new().unwrap();
        // also we need the color of the os name.
        // so read the os color from /etc/os-release.


        let _INIT_NAME: String = format!(
            "{}{}{}{}{}",
            "Tr".truecolor(0x74, 0xc7, 0xec),
            "an".truecolor(0xEB, 0xA0, 0xAC),
            "sge".truecolor(0xFF, 0xFF, 0xFF),
            "nd".truecolor(0xEB, 0xA0, 0xAC),
            "er".truecolor(0x74, 0xc7, 0xec)
        );

        let _OS_NAME: String = format!("\x1B[{}m{}", os_release.extra["ANSI_COLOR"], os_release.pretty_name);
        // now print the message "Transgender is starting {os_name} {kernel_version}..."
        eprintln!("{} is starting {}...",_INIT_NAME, _OS_NAME);

        // temporary delay for now.
        thread::sleep(Duration::from_secs(10));
        // abnormally exit and cause a kernel panic
        // TODO: actually invoke the init driver.
        process::exit(31);
    }
}
