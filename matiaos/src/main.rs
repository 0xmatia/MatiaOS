#![doc(html_logo_url = "https://git.io/JeGIp")]

//! Enter point of, well, everything
//! Well, not really, more general metadata, module definitions etc...
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![feature(unchecked_math)]
#![feature(const_option)]
#![no_main]
#![no_std]

mod bsp;
mod console;
mod cpu;
mod driver;
mod panic_handler;
mod print;
mod synchronization;
mod time;

/// Early init code.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
unsafe fn kernel_init() -> ! {
    if let Err(e) = bsp::driver::init() {
        panic!("Error initializing the driver subsystem !! {}", e)
    }

    // Initialize driver
    driver::driver_manager().init_drivers();
    // Console should now be registered

    kernel_main();
}

const OS_LOGO: &str = r#"
  __  __       _   _        ____   _____ 
 |  \/  |     | | (_)      / __ \ / ____|
 | \  / | __ _| |_ _  __ _| |  | | (___  
 | |\/| |/ _` | __| |/ _` | |  | |\___ \ 
 | |  | | (_| | |_| | (_| | |__| |____) |
 |_|  |_|\__,_|\__|_|\__,_|\____/|_____/ 
"#;

fn kernel_main() -> ! {
    println!("{OS_LOGO}");
    info!("Booting on: {}", bsp::board_name());
    info!("UART Console registered!");

    info!("Loaded drivers:");
    driver::driver_manager().enumerate();
    info!(
        "uptime: {} seconds",
        time::time_manager().uptime().as_secs()
    );

    info!("MatiaOS version {} is online", env!("CARGO_PKG_VERSION"));
    info!("Echo mode is on");
    loop {
        let chr = console::console().read_char();
        console::console().write_char(chr);
    }
}
