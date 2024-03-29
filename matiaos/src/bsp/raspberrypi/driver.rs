/*
 * File: driver.rs
 * Project: RpiOS
 * File Created: Thursday, 30th December 2021 2:49:44 pm
 * Author: Elad Matia (elad.matia@gmail.com)
 */

use crate::bsp::device_driver;
use crate::bsp::memory::map;
use crate::console;
use crate::driver as generic_driver;
use core::sync::atomic::{AtomicBool, Ordering};

// Global instances of the drivers, created first at boot (`kernel_init`)
static PL011_UART: device_driver::PL011Uart =
    unsafe { device_driver::PL011Uart::new(map::mmio::PL011_UART_START) };
static GPIO: device_driver::GPIO = unsafe { device_driver::GPIO::new(map::mmio::GPIO_START) };

/// This must be called only after successful init of the UART driver.
fn post_init_uart() -> Result<(), &'static str> {
    console::register_console(&PL011_UART);

    Ok(())
}

/// This must be called only after successful init of the GPIO driver.
fn post_init_gpio() -> Result<(), &'static str> {
    GPIO.init_gpio_uart_pins();
    Ok(())
}

fn driver_uart() -> Result<(), &'static str> {
    let uart_descriptor =
        generic_driver::DeviceDriverDescriptor::new(&PL011_UART, Some(post_init_uart));
    generic_driver::driver_manager().register_driver(uart_descriptor);

    Ok(())
}

fn driver_gpio() -> Result<(), &'static str> {
    let gpio_descriptor = generic_driver::DeviceDriverDescriptor::new(&GPIO, Some(post_init_gpio));
    generic_driver::driver_manager().register_driver(gpio_descriptor);

    Ok(())
}

/// Initialize the driver subsystem.
///
/// # Safety
///
/// See child function calls.
///
/// # Note
///
/// Using atomics here relieves us from needing to use `unsafe` for the static variable.
///
/// On `AArch64`, which is the only implemented architecture at the time of writing this,
/// [`AtomicBool::load`] and [`AtomicBool::store`] are lowered to ordinary load and store
/// instructions. They are therefore safe to use even with MMU + caching deactivated.
///
/// [`AtomicBool::load`]: core::sync::atomic::AtomicBool::load
/// [`AtomicBool::store`]: core::sync::atomic::AtomicBool::store
pub unsafe fn init() -> Result<(), &'static str> {
    static INIT_DONE: AtomicBool = AtomicBool::new(false);
    // Regular atmoic operation, nothing fancy - relxed
    if INIT_DONE.load(Ordering::Relaxed) {
        return Err("Init already done");
    }

    driver_uart()?;
    driver_gpio()?;

    INIT_DONE.store(true, Ordering::Relaxed);
    Ok(())
}
