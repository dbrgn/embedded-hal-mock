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

use core::convert::Infallible;
use std::thread;
use std::time::Duration;

use embedded_hal::delay::blocking as delay;

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

impl delay::DelayUs for MockNoop {
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
