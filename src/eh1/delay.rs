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

use crate::common::Generic;

/// Delay transaction type
///
/// Records a delay
pub type Transaction = u32;

/// A `Delay` implementation that does not actually block.
pub type Mock = Generic<Transaction>;
use crate::eh1::top_level::Expectation;

impl TryFrom<Expectation> for Transaction {
    type Error = ();

    fn try_from(expectation: Expectation) -> Result<Self, <Self as TryFrom<Expectation>>::Error> {
        match expectation {
            Expectation::Delay(transaction) => Ok(transaction),
            _ => Err(())
        }
    }
}

fn next_transaction(mock: &mut Generic<Transaction>) -> Transaction {
    if let Some(hal) = &mock.hal {
        hal.lock().unwrap().next().unwrap().try_into().expect("wrong expectation type")
    } else {
        mock.next().unwrap()
    }
}

impl delay::DelayNs for Mock {
    fn delay_ns(&mut self, ns: u32) {
        let w = next_transaction(self);

        assert_eq!(ns, w, "delaying by the wrong number of nanoseconds");
    }
}

impl Mock {
    pub fn expect_delay_ns(&self, ns: u32) -> Expectation {
        Expectation::Delay(ns)
    }
}
