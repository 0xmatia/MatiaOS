/*
 * File: console.rs
 * Project: RpiOS
 * File Created: Saturday, 6th November 2021 5:17:59 pm
 * Author: Elad Matia (elad.matia@gmail.com)
 */

mod null_console;
use crate::synchronization::{NullLock, interface::Mutex};

pub mod interface {
    pub use core::fmt;

    /// Console write functions
    pub trait Write {
        /// Write a single character
        fn write_char(&self, c: char);
        /// write format string trait
        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
        /// block until TX FIFO is not busy anymore
        fn flush(&self);
    }

    /// Console read functions
    pub trait Read {
        /// Read one character
        fn read_char(&self) -> char {
            ' '
        }
        /// Clear RX buffers
        fn clear_rx(&self);
    }
    /// console statistics
    pub trait Statistics {
        /// returns the number of characters written
        fn chars_written(&self) -> usize {
            0
        }
        /// returns the number of characters read
        fn chars_read(&self) -> usize {
            0
        }
    }

    /// trait alias: All the stuff a fully functional console needs
    pub trait All: Read + Write + Statistics {}
}

//--------------------------------------------------------------------------------------------------
// Public definitions
//--------------------------------------------------------------------------------------------------

static CUR_CONSOLE: NullLock<&'static (dyn interface::All + Sync)> = NullLock::new(&null_console::NULL_CONSOLE);

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Register a new console.
pub fn register_console(new_console: &'static (dyn interface::All + Sync)) {
    CUR_CONSOLE.lock(|con| *con = new_console);
}

/// Return a reference to the currently registered console.
///
/// This is the global console used by all printing macros.
pub fn console() -> &'static dyn interface::All {
    CUR_CONSOLE.lock(|con| *con)
} 
