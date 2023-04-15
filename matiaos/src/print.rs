/*
 * File: print.rs
 * Project: RpiOS
 * File Created: Saturday, 6th November 2021 5:42:51 pm
 * Author: Elad Matia (elad.matia@gmail.com)
 */

//! Print functions

use crate::console;
use core::fmt;

// private, helper function
pub fn __print(args: fmt::Arguments) {
    // This is just fmt::Write, but more readable (or is it?)
    console::console().write_fmt(args).unwrap();
}

// public usable macros: print, println

/// Regular print, no endline
#[macro_export]
macro_rules! print {
    ($($args:tt)*) => {
        ($crate::print::__print(format_args!($($args)*)));
    };
}

/// Print with newline at the end
#[macro_export]
macro_rules! println {
    () => {
        print!("\n");
    };
    ($($args:tt)*) => {
        ($crate::print::__print(format_args_nl!($($args)*)))
    };
}

// The following macros are taken directly from the github repo of rust-embedded.
// I am not proficient enough with macros to understand them, but they are useful.

/// Prints an info, with a newline.
#[macro_export]
macro_rules! info {
    ($string:expr) => ({
        let timestamp = $crate::time::time_manager().uptime();

        $crate::print::__print(format_args_nl!(
            concat!("[  {:>3}.{:06}] ", $string),
            timestamp.as_secs(),
            timestamp.subsec_micros(),
        ));
    });
    ($format_string:expr, $($arg:tt)*) => ({
        let timestamp = $crate::time::time_manager().uptime();

        $crate::print::__print(format_args_nl!(
            concat!("[  {:>3}.{:06}] ", $format_string),
            timestamp.as_secs(),
            timestamp.subsec_micros(),
            $($arg)*
        ));
    })
}

/// Prints a warning, with a newline.
#[macro_export]
macro_rules! warn {
    ($string:expr) => ({
        let timestamp = $crate::time::time_manager().uptime();

        $crate::print::__print(format_args_nl!(
            concat!("[W {:>3}.{:06}] ", $string),
            timestamp.as_secs(),
            timestamp.subsec_micros(),
        ));
    });
    ($format_string:expr, $($arg:tt)*) => ({
        let timestamp = $crate::time::time_manager().uptime();

        $crate::print::__print(format_args_nl!(
            concat!("[W {:>3}.{:06}] ", $format_string),
            timestamp.as_secs(),
            timestamp.subsec_micros(),
            $($arg)*
        ));
    })
}
