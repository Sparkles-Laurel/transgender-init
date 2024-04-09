// we all love to see a bsod

use std::backtrace::Backtrace;
use std::io::{stdin, stdout, Write};
use std::os::fd::RawFd;
use std::os::unix::process::CommandExt;
use std::panic::PanicInfo;
use std::process::Command;

use nix::fcntl::{open, OFlag};
use nix::ioctl_write_int_bad;
use nix::sys::reboot::{reboot, RebootMode};
use nix::sys::stat::Mode;
use nix::unistd::isatty;

const BLUE_BG: &str = "\x1b]P0000080";
const WHITE_TEXT: &str = "\x1b]P7FFFFFF";

const CLEAR_SCREEN: &str = "\x1b[1;1H\x1b[2J";

const VT_ACTIVATE: i32 = 0x5606;
const VT_WAIT_ACTIVE: i32 = 0x5607;

ioctl_write_int_bad!(vt_activate, VT_ACTIVATE);
ioctl_write_int_bad!(vt_wait_activate, VT_WAIT_ACTIVE);

fn print_info(info: &PanicInfo, tty: bool) {
    if tty {
        print!("{}", BLUE_BG);
        print!("{}", WHITE_TEXT);

        print!("{}", CLEAR_SCREEN);
    }

    println!(
        "Kanit Panic\n===========\nKanit experienced an unrecoverable error and cannot continue."
    );
    println!("Consider checking the system journal or the following backtrace.\n");

    if let Some(s) = info.payload().downcast_ref::<&str>() {
        println!("Message: {}", s);
    }

    if let Some(loc) = info.location() {
        println!("File: {}:{}:{}", loc.file(), loc.line(), loc.column());
    }

    let mut junk = String::new();

    if tty {
        println!("Press enter to view backtrace...");

        let _ = stdout().flush();

        let _ = stdin().read_line(&mut junk);
    }

    let backtrace = Backtrace::force_capture();

    println!("-- backtrace --\n{}", backtrace);

    if tty {
        println!("Init is now considered unstable, press enter to enter into emergency shell...");

        let _ = stdout().flush();
        let _ = stdin().read_line(&mut junk);
    }

    Command::new("sh").exec();

    if tty {
        println!("failed to enter emergency shell, press enter to restart...");

        let _ = stdout().flush();
        let _ = stdin().read_line(&mut junk);
    }

    let _ = reboot(RebootMode::RB_AUTOBOOT);
}

pub fn bsod(info: &PanicInfo) {
    if !isatty(RawFd::from(1)).unwrap_or(false) {
        return print_info(info, false);
    }

    // change virtual terminal
    if let Ok(fd) = open("/dev/console", OFlag::O_RDONLY, Mode::empty()) {
        // SAFETY: we ensure `fd` is valid
        unsafe {
            let _ = vt_activate(fd, 1);
            let _ = vt_wait_activate(fd, 1);
        }
    }

    print_info(info, false);
}
