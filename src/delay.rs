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

use core::convert::Infallible;
use std::thread;
use std::time::Duration;

use embedded_hal::delay;

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

impl delay::DelayUs for NoopDelay {
    type Error = Infallible;

    /// A no-op delay implementation.
    fn delay_us(&mut self, _n: u32) -> Result<(), Self::Error> {
        Ok(())
    }
}

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

impl delay::DelayUs for StdSleep {
    type Error = Infallible;

    /// A `Delay` implementation that uses `std::thread::sleep`.
    fn delay_us(&mut self, n: u32) -> Result<(), Self::Error> {
        thread::sleep(Duration::from_micros(n as u64));
        Ok(())
    }
}
