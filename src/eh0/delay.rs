//! Delay mock implementations.
//!
//! ## Usage
//!
//! If the actual sleep duration is not important, simply create a
//! [`NoopDelay`](struct.NoopDelay.html) instance. There will be no actual
//! delay. This is useful for fast tests, where you don't actually need to wait
//! for the hardware.
//!
//! If you do want the real delay behavior, use
//! [`StdSleep`](struct.StdSleep.html) which uses
//! [`std::thread::sleep`](https://doc.rust-lang.org/std/thread/fn.sleep.html)
//! to implement the delay.

use std::{thread, time::Duration};

use eh0 as embedded_hal;
use embedded_hal::blocking::delay;

/// A `Delay` implementation that does not actually block.
pub struct NoopDelay;

impl NoopDelay {
    /// Create a new `NoopDelay` instance.
    pub fn new() -> Self {
        NoopDelay
    }
}

impl Default for NoopDelay {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! impl_noop_delay_us {
    ($type:ty) => {
        impl delay::DelayUs<$type> for NoopDelay {
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
        impl delay::DelayMs<$type> for NoopDelay {
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
