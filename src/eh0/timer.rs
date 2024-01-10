//! Provides a mocked [embedded_time::Clock] that can be used for host-side testing
//! crates that use [embedded_hal::timer].
//!
//! The provided [embedded_time::Clock] implementation is thread safe and can be freely
//! skipped forward with nanosecond precision.
//!
//! # Usage
//!
//! ```rust
//! # use eh0 as embedded_hal;
//! use embedded_hal::timer::CountDown;
//! use embedded_hal_mock::eh0::timer::MockClock;
//! use embedded_time::duration::*;
//!
//! let mut clock = MockClock::new();
//! let mut timer = clock.get_timer();
//! timer.start(100.nanoseconds());
//! // hand over timer to embedded-hal based driver
//! // continue to tick clock
//! clock.tick(50.nanoseconds());
//! assert_eq!(timer.wait(), Err(nb::Error::WouldBlock));
//! clock.tick(50.nanoseconds());
//! assert_eq!(timer.wait(), Ok(()));
//! clock.tick(50.nanoseconds());
//! assert_eq!(timer.wait(), Err(nb::Error::WouldBlock));
//! clock.tick(50.nanoseconds());
//! assert_eq!(timer.wait(), Ok(()));
//! ```

use std::{
    convert::Infallible,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use eh0 as embedded_hal;
use embedded_hal::timer::{Cancel, CountDown, Periodic};
pub use embedded_time::Clock;
use embedded_time::{clock, duration::*, fraction::Fraction, Instant};
use void::Void;

/// A simulated clock that can be used in tests.
#[derive(Clone, Debug)]
pub struct MockClock {
    ticks: Arc<AtomicU64>,
}

impl Clock for MockClock {
    type T = u64;
    const SCALING_FACTOR: Fraction = Fraction::new(1, 1_000_000_000);

    fn try_now(&self) -> Result<Instant<Self>, clock::Error> {
        let ticks: u64 = self.ticks.load(Ordering::Relaxed);
        Ok(Instant::<Self>::new(ticks))
    }
}

impl Default for MockClock {
    fn default() -> Self {
        MockClock {
            ticks: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl MockClock {
    /// Creates a new simulated clock.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of elapsed nanoseconds.
    pub fn elapsed(&self) -> Nanoseconds<u64> {
        Nanoseconds(self.ticks.load(Ordering::Relaxed))
    }

    /// Forward the clock by `ticks` amount.
    pub fn tick<T>(&mut self, ticks: T)
    where
        T: Into<Nanoseconds<u64>>,
    {
        self.ticks.fetch_add(ticks.into().0, Ordering::Relaxed);
    }

    /// Get a new timer based on the clock.
    pub fn get_timer(&self) -> MockTimer {
        let clock = self.clone();
        let duration = Nanoseconds(1);
        let expiration = clock.try_now().unwrap();
        MockTimer {
            clock: self.clone(),
            duration,
            expiration,
            started: false,
        }
    }
}

/// A simulated timer that can be used in tests.
pub struct MockTimer {
    clock: MockClock,
    duration: Nanoseconds<u64>,
    expiration: Instant<MockClock>,
    started: bool,
}

impl CountDown for MockTimer {
    type Time = Nanoseconds<u64>;

    fn start<T>(&mut self, count: T)
    where
        T: Into<Self::Time>,
    {
        let now = self.clock.try_now().unwrap();
        self.duration = count.into();
        self.expiration = now + self.duration;
        self.started = true;
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        let now = self.clock.try_now().unwrap();
        if self.started && now >= self.expiration {
            self.expiration = now + self.duration;
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl Periodic for MockTimer {}

impl Cancel for MockTimer {
    type Error = Infallible;

    fn cancel(&mut self) -> Result<(), Self::Error> {
        self.started = false;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn count_down() {
        let mut clock = MockClock::new();
        let mut timer = clock.get_timer();
        timer.start(100.nanoseconds());
        clock.tick(50.nanoseconds());
        assert_eq!(timer.wait(), Err(nb::Error::WouldBlock));
        clock.tick(50.nanoseconds());
        assert_eq!(timer.wait(), Ok(()));
        clock.tick(50.nanoseconds());
        assert_eq!(timer.wait(), Err(nb::Error::WouldBlock));
        clock.tick(50.nanoseconds());
        assert_eq!(timer.wait(), Ok(()));
    }
}
