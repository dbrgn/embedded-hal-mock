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

use std::thread;
use std::time::Duration;

use eh1 as embedded_hal;
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

impl delay::DelayNs for NoopDelay {
    fn delay_ns(&mut self, _ns: u32) {
        // no-op
    }
}

#[cfg(feature = "embedded-hal-async")]
impl embedded_hal_async::delay::DelayNs for NoopDelay {
    async fn delay_ns(&mut self, _ns: u32) {
        // no-op
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

impl delay::DelayNs for StdSleep {
    fn delay_ns(&mut self, ns: u32) {
        thread::sleep(Duration::from_nanos(ns as u64));
    }
}

#[cfg(feature = "embedded-hal-async")]
impl embedded_hal_async::delay::DelayNs for StdSleep {
    async fn delay_ns(&mut self, _ns: u32) {
        // no-op
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_noop_sleep() {
        use embedded_hal::delay::DelayNs;

        let mut delay = NoopDelay::new();
        delay.delay_ms(10);
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_noop_sleep_async() {
        use embedded_hal_async::delay::DelayNs;

        let mut delay = NoopDelay::new();
        delay.delay_ms(10).await;
    }

    #[test]
    fn test_std_sleep() {
        use embedded_hal::delay::DelayNs;

        let mut delay = StdSleep::new();
        delay.delay_ms(10);
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_std_sleep_async() {
        use embedded_hal_async::delay::DelayNs;

        let mut delay = StdSleep::new();
        delay.delay_ms(10).await;
    }
}
