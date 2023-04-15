/*
 * File: raspberrypi.rs
 * Project: RpiOS
 * File Created: Tuesday, 26th October 2021 5:45:10 pm
 * Author: Elad Matia (elad.matia@gmail.com)
 */

//! board specific code for Raspberry Pi (currently 3) 

pub mod cpu;
pub mod driver;
pub mod memory;

/// Returns the board's name (rpi3, rpi4)
pub fn board_name() -> &'static str {
    #[cfg(feature="bsp_rpi3")]
    {
        "Raspberry pi 3"
    }
    
    #[cfg(feature="bsp_rpi4")]
    {

        "Raspberry pi 4"
    }
}
