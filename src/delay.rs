//! Delay mock implementations.
//!
//! ## Usage
//!
//! If the actual sleep duration is not important, simply create a
//! [`MockNoop`](struct.MockNoop.html) instance. There will be no actual
//! delay. This is useful for fast tests, where you don't actually need to wait
//! for the hardware.
//!
//! If you do want the real delay behavior, use
//! [`StdSleep`](struct.StdSleep.html) which uses
//! [`std::thread::sleep`](https://doc.rust-lang.org/std/thread/fn.sleep.html)
//! to implement the delay.

use std::thread;
use std::time::Duration;

use embedded_hal::blocking::delay;

/// A `Delay` implementation that does not actually block.
pub struct MockNoop;

impl MockNoop {
    /// Create a new `MockNoop` instance.
    pub fn new() -> Self {
        MockNoop
    }
}

impl Default for MockNoop {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! impl_noop_delay_us {
    ($type:ty) => {
        impl delay::DelayUs<$type> for MockNoop {
            /// A no-op delay implementation.
            fn delay_us(&mut self, _n: $type) {}
        }
    };
}

impl_noop_delay_us!(u8);
impl_noop_delay_us!(u16);
impl_noop_delay_us!(u32);
impl_noop_delay_us!(u64);

macro_rules! impl_noop_delay_ms {
    ($type:ty) => {
        impl delay::DelayMs<$type> for MockNoop {
            /// A no-op delay implementation.
            fn delay_ms(&mut self, _n: $type) {}
        }
    };
}

impl_noop_delay_ms!(u8);
impl_noop_delay_ms!(u16);
impl_noop_delay_ms!(u32);
impl_noop_delay_ms!(u64);

/// A `Delay` implementation that uses `std::thread::sleep`.
pub struct StdSleep;

impl StdSleep {
    /// Create a new `StdSleep` instance.
    pub fn new() -> Self {
        StdSleep
    }
}

impl Default for StdSleep {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! impl_stdsleep_delay_us {
    ($type:ty) => {
        impl delay::DelayUs<$type> for StdSleep {
            /// A `Delay` implementation that uses `std::thread::sleep`.
            fn delay_us(&mut self, n: $type) {
                thread::sleep(Duration::from_micros(n as u64));
            }
        }
    };
}

impl_stdsleep_delay_us!(u8);
impl_stdsleep_delay_us!(u16);
impl_stdsleep_delay_us!(u32);
impl_stdsleep_delay_us!(u64);

macro_rules! impl_stdsleep_delay_ms {
    ($type:ty) => {
        impl delay::DelayMs<$type> for StdSleep {
            /// A `Delay` implementation that uses `std::thread::sleep`.
            fn delay_ms(&mut self, n: $type) {
                thread::sleep(Duration::from_millis(n as u64));
            }
        }
    };
}

impl_stdsleep_delay_ms!(u8);
impl_stdsleep_delay_ms!(u16);
impl_stdsleep_delay_ms!(u32);
impl_stdsleep_delay_ms!(u64);
