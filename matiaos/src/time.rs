//! Abstraction interface for time-related operations
//! Currently supported architectures: aarch64

#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/time.rs"]
mod arch_time;

use core::time::Duration;

/// A generic time manager
pub struct TimeManager;

static TIME_MANAGER: TimeManager = TimeManager::new();

/// Return a reference to the global TimeManager.
pub fn time_manager() -> &'static TimeManager {
    &TIME_MANAGER
}

impl TimeManager {
    pub const fn new() -> Self {
        Self
    }

    /// The uptime of the device since power-on
    pub fn uptime(&self) -> Duration {
        arch_time::uptime()
    }

    /// Spin for duration (i.e sleep/block the current task)
    pub fn spin_for_duration(&self, duration: Duration) {
        arch_time::spin_for_duration(duration);
    }
}
