use std::fmt;

use log::{set_boxed_logger, set_max_level, LevelFilter, Metadata, Record};

use kanit_common::error::{Context, Result};

const SYMBOLS: [char; 5] = ['!', '=', '*', '&', '#'];

const COLORS: [Colors; 5] = [
    Colors::BrightRed,
    Colors::BrightYellow,
    Colors::BrightGreen,
    Colors::BrightCyan,
    Colors::BrightMagenta,
];

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Colors {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
    BrightBlack = 8,
    BrightRed = 9,
    BrightGreen = 10,
    BrightYellow = 11,
    BrightBlue = 12,
    BrightMagenta = 13,
    BrightCyan = 14,
    BrightWhite = 15,
}

impl Colors {
    pub fn reset() -> &'static str {
        "\x1b[0m"
    }
}

impl fmt::Display for Colors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1b[38;5;{}m", *self as u16)
    }
}

// temporary solution
// logger should switch to a journal once initialized
pub struct InitializationLogger {
    level: LevelFilter,
}

impl InitializationLogger {
    pub fn init(level: LevelFilter) -> Result<()> {
        let logger = Self { level };

        set_max_level(level);
        set_boxed_logger(Box::new(logger)).context("failed to initialize logger")?;

        Ok(())
    }
}

impl log::Log for InitializationLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let idx = (record.level() as usize) - 1;
            let sym = SYMBOLS[idx];
            let color = COLORS[idx];

            println!("{}{}{} {}", color, sym, Colors::reset(), record.args())
        }
    }

    fn flush(&self) {}
}
