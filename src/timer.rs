//! Timer mock implementations.
//!
//! ## Usage
//!
//! TODO
//!

use void::Void;

use embedded_hal::timer::{CountDown, Cancel, Periodic};

#[derive(Debug, PartialEq, Eq)]
enum ClockState {
    Idle,
    Counting,
    Canceled,
}

/// A `Timer` implementation
pub struct MockTimer<Unit> {
    tick: Unit,
    state: ClockState,
}

impl<Unit: Default> MockTimer<Unit> {
    /// Create a new `MockTimer` instance.
    pub fn new() -> Self {
        MockTimer {
            tick: Unit::default(),
            state: ClockState::Idle,
        }
    }
}

impl<Unit> CountDown for MockTimer<Unit> {
    type Time = Unit;

    fn start<T>(&mut self, count: T)
    where
        T: Into<Self::Time> {
            self.state = ClockState::Counting;
            self.tick = count.into();
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        /* if self.state != ClockState::Counting {
            return Err(_)
        } */
        self.state = ClockState::Idle;
        Ok(())
    }
}

impl<Unit> Periodic for MockTimer<Unit> {}

impl<Unit> Cancel for MockTimer<Unit> {
    type Error = ();

    fn cancel(&mut self) -> Result<(), Self::Error> {
        if self.state != ClockState::Counting {
            return Err(())
        }
        self.state = ClockState::Canceled;
        Ok(())
    }
}
