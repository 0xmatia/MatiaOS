//! aarch64 timer primitives
//! Use CNTPCT_EL0 and CNTFRQ_EL0 to implement a simple timer.
//!

use crate::warn;
use aarch64_cpu::{asm::barrier, registers::*};
use core::{
    num::{NonZeroU128, NonZeroU32, NonZeroU64}, // This is for optimization purposes
    ops::{Add, Div},
    time::Duration,
};
use tock_registers::interfaces::Readable;

/// Number of nanoseconds per second
const NANOSEC_PER_SEC: NonZeroU64 = NonZeroU64::new(1_000_000_000).unwrap();

/// Internal counter type (CNTPCT_EL0)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct GenericTimerCounterValue(u64);

// This is a dummy value; the boot code in `boot.s` will overide it with the content of of the
// CNTFRQ_EL0 register (i.e the clock frequency of the CNTPCT_EL0 counter).
#[no_mangle]
static ARCH_TIMER_COUNTER_FREQUENCY: NonZeroU32 = NonZeroU32::MIN;

/// Get the current architecture timer frequency
fn get_arch_timer_frequency() -> NonZeroU32 {
    // Prevent unwanted optimizations of the compiler because it is unware
    // ARCH_TIMER_COUNTER_FREQUENCY can be changed "unded the hood" (boot code)
    // In any case, we know this operation is safe, because only we are in control of its value
    unsafe { core::ptr::read_volatile(&ARCH_TIMER_COUNTER_FREQUENCY) }
}

// Implementations for the internal counter type.
impl GenericTimerCounterValue {
    /// Max counter value
    pub const MAX: Self = GenericTimerCounterValue(u64::MAX);
}

/// Implement add for GenericTimerCounterValue
impl Add for GenericTimerCounterValue {
    type Output = GenericTimerCounterValue;

    fn add(self, rhs: Self) -> Self::Output {
        GenericTimerCounterValue(self.0.wrapping_add(rhs.0))
    }
}

/// Get the max possible value of the duration object
fn max_duration() -> Duration {
    Duration::from(GenericTimerCounterValue::MAX)
}

impl From<GenericTimerCounterValue> for Duration {
    // To construct the duration object we need: seconds, nanoseconds.
    // To calculate seconds: counter_value / frequency
    //     - for example, if the frequency is 20hz, it means the counter is ticking in
    //       20 ticks per seconds. GenericTimerCounterValue is the CNTPCT_EL0 value, so dividing it
    //       will give us the seconds
    // To calculate nanoseconds: get the remainder - counter_value % frequency and multiply it by
    // nanoseconds per seconds to get the value in nanoseconds.
    fn from(counter_value: GenericTimerCounterValue) -> Self {
        if counter_value.0 == 0 {
            return Duration::ZERO;
        }

        // Get the timer frequency (CNTFRQ_EL0)
        let frequency: NonZeroU64 = get_arch_timer_frequency().into();
        let seconds = counter_value.0.div(frequency);

        // The remainder
        let subsecond_counter = counter_value.0 % frequency;
        let nanos = unsafe { subsecond_counter.unchecked_mul(u64::from(NANOSEC_PER_SEC)) }
            .div(frequency) as u32;
        Duration::new(seconds, nanos)
    }
}

impl TryFrom<Duration> for GenericTimerCounterValue {
    type Error = &'static str;

    fn try_from(duration: Duration) -> Result<Self, Self::Error> {
        if duration < resolution() {
            return Ok(GenericTimerCounterValue(0));
        }

        if duration > max_duration() {
            return Err("Convertion error. Duration overflowed max allowed value (u64::max)");
        }

        let frequency = u32::from(get_arch_timer_frequency()) as u128;
        let duration: u128 = duration.as_nanos();
        let counter_value =
            unsafe { duration.unchecked_mul(frequency) }.div(NonZeroU128::from(NANOSEC_PER_SEC));

        Ok(GenericTimerCounterValue(counter_value as u64))
    }
}

/// The timer's resolution.
/// Meaning: Get the smallest possible value (non zero) value possible for the counter.
/// This is how accurate our timer can be.
pub fn resolution() -> Duration {
    Duration::from(GenericTimerCounterValue(1))
}

/// Read the timer value from the register (u64)
#[inline(always)]
fn read_cntpct() -> GenericTimerCounterValue {
    barrier::isb(barrier::SY);
    let cnt = CNTPCT_EL0.get();
    GenericTimerCounterValue(cnt)
}

/// Get the system uptime (basically read the counter divided by the frequency)
pub fn uptime() -> Duration {
    read_cntpct().into()
}

/// Spin for Duration
pub fn spin_for_duration(duration: Duration) {
    let current_timer_value = read_cntpct();

    let counter_value_delta: GenericTimerCounterValue = match duration.try_into() {
        Err(msg) => {
            warn!("spin_for_duration error: {}", msg);
            return;
        }
        Ok(val) => val,
    };

    // The target timer value to spin: current timer value + the request time(r) delta
    let timer_target = current_timer_value + counter_value_delta;

    while (GenericTimerCounterValue(CNTPCT_EL0.get())) < timer_target {}
}
