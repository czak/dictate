use std::fmt::Arguments;
use std::io::{self, Write};

const RESET: &str = "\x1b[0m";

pub fn emit(color: &str, label: &str, message: Arguments) {
    let mut stderr = io::stderr().lock();
    let _ = writeln!(stderr, "{color}{label:<7}{RESET} {message}");
}

macro_rules! info {
    ($($arg:tt)*) => {
        $crate::log::emit("\x1b[36m", "info", format_args!($($arg)*))
    };
}

macro_rules! warning {
    ($($arg:tt)*) => {
        $crate::log::emit("\x1b[33m", "warning", format_args!($($arg)*))
    };
}

macro_rules! err {
    ($($arg:tt)*) => {
        $crate::log::emit("\x1b[1;31m", "err", format_args!($($arg)*))
    };
}

pub(crate) use {err, info, warning};
