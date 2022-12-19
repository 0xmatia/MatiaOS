#![doc(html_logo_url = "https://git.io/JeGIp")]

//! Enter point of, well, everything
//! Well, not really, more general metadata, module definitions etc...
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![no_main]
#![no_std]


mod cpu;
mod bsp;
mod console;
mod print;
mod synchronization;
mod driver;
mod panic_handler;

/// Early init code.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
unsafe fn kernel_init() -> ! {
    use crate::driver::interface::DeviceManager;

    for i in bsp::driver::driver_manager().all_device_drivers().iter() {
        if let Err(e) = i.init() {
            panic!("Error initializing {} driver: {}", i.compatible(), e);
        }
    }
    bsp::driver::driver_manager().post_device_driver_init();

    kernel_main();
}

const OS_LOGO: &str = r#"
      ___                                      ___           ___     
     /  /\          ___            ___        /  /\         /  /\    
    /  /::\        /  /\          /__/\      /  /::\       /  /::\   
   /  /:/\:\      /  /::\         \__\:\    /  /:/\:\     /__/:/\:\  
  /  /::\ \:\    /  /:/\:\        /  /::\  /  /:/  \:\   _\_ \:\ \:\ 
 /__/:/\:\_\:\  /  /::\ \:\    __/  /:/\/ /__/:/ \__\:\ /__/\ \:\ \:\
 \__\/~|::\/:/ /__/:/\:\_\:\  /__/\/:/E.M \  \:\ /  /:/ \  \:\ \:\_\/
    |  |:|::/  \__\/  \:\/:/  \  \::/      \  \:\  /:/   \  \:\_\:\  
    |  |:|\/        \  \::/    \  \:\       \  \:\/:/     \  \:\/:/  
    |__|:|~          \__\/      \__\/        \  \::/       \  \::/   
     \__\|                                    \__\/         \__\/    
"#;

fn kernel_main() -> ! {
    use bsp::console::console;
    use console::interface::All;

    println!("{}", OS_LOGO);
    println!("Running on: {}", bsp::board_name());
    println!();
    println!("RpiOS version 0.1 online");
    println!("Echo mode on");
    console().flush();
    console().clear_rx();

    loop {
        let chr = console().read_char();
        console().write_char(chr);
    }
}
