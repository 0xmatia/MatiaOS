/*
* File: driver.rs
* Project: RpiOS
* File Created: Sunday, 26th December 2021 4:16:20 pm
* Author: Elad Matia (elad.matia@gmail.com)
*/

use crate::info;
use crate::synchronization::interface::Mutex;
use crate::synchronization::NullLock;

const MAX_SUPPORTED_DRIVERS: usize = 10;

/// Implementation of a device driver manager
struct DriverManagerInner {
    next_driver_index: usize,
    drivers: [Option<DeviceDriverDescriptor>; MAX_SUPPORTED_DRIVERS],
}

impl DriverManagerInner {
    const fn new() -> Self {
        Self {
            next_driver_index: 0,
            drivers: [None; MAX_SUPPORTED_DRIVERS],
        }
    }
}

/// Driver-related traits (DeviceDriver, Driver manager)
pub mod interface {
    /// Device driver trait - each driver has to implement this
    pub trait DeviceDriver {
        /// Return a string identifying the driver
        fn compatible(&self) -> &'static str;

        /// Called by kernel on startup to initialize the driver.
        /// Devices can only be used after their driver has been initialized
        fn init(&self) -> Result<(), &'static str> {
            Ok(())
        }
    }
}

/// Callback type for device drivers that need a post-init callback
pub type DeviceDriverPostInitCB = unsafe fn() -> Result<(), &'static str>;

/// Describes a device driver
#[derive(Copy, Clone)]
pub struct DeviceDriverDescriptor {
    device_driver: &'static (dyn interface::DeviceDriver + Sync),
    post_init_cb: Option<DeviceDriverPostInitCB>,
}

/// Driver manager
pub struct DriverManager {
    inner: NullLock<DriverManagerInner>,
}

/// Global device_driver instance
pub static DRIVER_MANAGER: DriverManager = DriverManager::new();

/// Get the global driver manager instance
pub fn driver_manager() -> &'static DriverManager {
    &DRIVER_MANAGER
}

impl DeviceDriverDescriptor {
    pub fn new(
        device_driver: &'static (dyn interface::DeviceDriver + Sync),
        post_init_cb: Option<DeviceDriverPostInitCB>,
    ) -> Self {
        Self {
            device_driver,
            post_init_cb,
        }
    }
}

impl DriverManager {
    pub const fn new() -> Self {
        Self {
            inner: NullLock::new(DriverManagerInner::new()),
        }
    }
    /// Register a device descriptor with the kernel's device-driver manager
    pub fn register_driver(&self, device_descriptor: DeviceDriverDescriptor) {
        self.inner.lock(|inner| {
            inner.drivers[inner.next_driver_index] = Some(device_descriptor);
            inner.next_driver_index += 1;
        })
    }

    /// Run a function on all drivers
    pub fn for_each_descriptor(&self, f: impl FnMut(&DeviceDriverDescriptor)) {
        self.inner.lock(|inner| {
            inner
                .drivers
                .iter()
                .filter_map(|x| x.as_ref())
                .for_each(f)
        })
    }

    /// Initialize all registed drivers
    pub unsafe fn init_drivers(&self) {
        // Call init on all drivers
        self.for_each_descriptor(|driver| {
            if let Err(x) = driver.device_driver.init() {
                panic!(
                    "Error initializing device driver: {}: {}",
                    driver.device_driver.compatible(),
                    x
                )
            }

            // call post-init cb if needed
            if let Some(callback) = driver.post_init_cb {
                if let Err(e) = callback() {
                    panic!(
                        "Error executing post init callback for driver: {}: {}",
                        driver.device_driver.compatible(),
                        e
                    )
                }
            }
        })
    }

    /// Enumerate all registered drivers
    pub fn enumerate(&self) {
        let mut index = 0;
        self.for_each_descriptor(|descriptor| {
            info!("[{}] - {}", index, descriptor.device_driver.compatible());
            index += 1;
        })
    }
}
