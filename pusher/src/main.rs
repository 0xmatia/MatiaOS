//! # Pusher by Elad Matia
//!
//! ## The problem
//! The purpose of this simple crate is to make the life of a kernel / embedded developer easier.
//! The main issue I encounterd while developing a simple kernel for the rpi3 was the repetetive
//! action of inserting the sdcard to my computer everytime I wanted to update the kernel.
//!
//! ## The solution
//! The solution was to write a very simple PIC that sits on the rpi and sends a signal over UART
//! signaling when it is ready to receive the kernel. The PIC relocated after loading so that the
//! kernel it receives can be written to the load address of the rpi. On the other side of the UART
//! this binary waits for the signal and sends the binary. Then, the PIC jumps to the newly pushed
//! kernel. This process will make your life much simpler when developing.

mod stdio;
mod session;

use anyhow::{anyhow, Result};
use libc::{
    close, // read-write
    isatty,
    open,
    O_NOCTTY,   // open as non-controlling terminal
    O_NONBLOCK, // non blocking
    O_RDWR,
};
use std::path::{Path, PathBuf};
use std::{env, io};

use session::SerialSession;

const PUSHER_LOGO: &str = r#"
__________             .__                  
\______   \__ __  _____|  |__   ___________ 
 |     ___/  |  \/  ___/  |  \_/ __ \_  __ \
 |    |   |  |  /\___ \|   Y  \  ___/|  | \/
 |____|   |____//____  >___|  /\___  >__|   
                     \/     \/     \/       
"#;

fn main() -> Result<()> {
    println!("{PUSHER_LOGO}\n[PUSHER] Pusher is waiting...");
    let (serial_path, baudrate, kernel_path) = parse_input()?;
    println!("Baudrate: {baudrate}");
    let mut pusher_session = SerialSession::init(serial_path, baudrate, kernel_path)?;
    pusher_session.start_pusher()?;
    Ok(())
}



/// Parse command line arguments.
/// Checks if the serial device exists and is a tty and if the kernel image exists
///
/// # Usage:
/// pusher <tty_device> <baudrate> <kernel_to_push>
///
/// # Return
/// The tty device path and a path to the kernel image
fn parse_input() -> Result<(String, u32, PathBuf)> {
    let supplied_arguments: Vec<String> = env::args().collect();
    if supplied_arguments.len() != 4 {
        return Err(anyhow!("Usage: pusher <device> <baudrate> <kernel>"));
    }
    // check if the supplied device exists
    if !Path::new(&supplied_arguments[1]).exists() {
        return Err(anyhow!("Device doesn't exists"));
    }
    // check if device is a tty
    let fd;
    unsafe {
        fd = open(
            supplied_arguments[1].as_ptr() as *const i8,
            O_RDWR | O_NOCTTY | O_NONBLOCK,
        );
        if -1 == fd {
            return Err(anyhow!(format!(
                "Couldn't open device: {}",
                io::Error::last_os_error()
            )));
        }
        if 0 == isatty(fd) {
            return Err(anyhow!("Supplied device is not a tty!"));
        }

        if -1 == close(fd) {
            return Err(anyhow!("Failed closing serial device fd"));
        }
    }
    // check the the binary to push exists
    if !Path::new(&supplied_arguments[3]).exists() {
        return Err(anyhow!(format!("{} doesn't exist", supplied_arguments[3])));
    }
    Ok((
        supplied_arguments[1].clone(),
        supplied_arguments[2].parse::<u32>()?,
        PathBuf::from(&supplied_arguments[3]),
    ))
}
