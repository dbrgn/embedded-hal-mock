//! Mock [`pwm`] implementation
//!
//! [`pwm`]: https://docs.rs/embedded-hal/0.2.7/embedded_hal/trait.Pwm.html
//!
//! ```
//! # use eh0 as embedded_hal;
//!
//! use embedded_hal_mock::eh0::pwm::{Transaction as PwmTransaction, TransactionKind as PwmTransactionKind, Mock as PwmMock, Channel as MockChannel};
//! use embedded_hal::Pwm;
//!
//!
//! let channel = MockChannel {};
//! let expected_duty = 10_000;
//! // Configure expectations
//! let expectations = [
//!     PwmTransaction::enable(),
//!     PwmTransaction::new(PwmTransactionKind::SetDuty(expected_duty)),
//!     PwmTransaction::new(PwmTransactionKind::GetDuty(expected_duty)),
//!     PwmTransaction::disable(),
//! ];
//!
//! // Create pwn
//! let mut pwm = PwmMock::new(&expectations);
//!
//! // Run and test
//!
//! pwm.enable(channel);
//! pwm.set_duty(channel, expected_duty);
//! assert_eq!(pwm.get_duty(channel), expected_duty);
//! pwm.disable(channel);
//!
//! pwm.done();
//!
//! // Update expectations
//! pwm.expect(&[]);
//! // ...
//! pwm.done();
//!
//! ```

use crate::common::Generic;

use eh0 as embedded_hal;
use embedded_hal::Pwm;

/// The type used for the time of the [`Pwm`] mock.
pub type PwmTime = u32;
/// The type used for the duty of the [`Pwm`] mock.
pub type PwmDuty = u16;

/// MockPwm transaction
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Transaction {
    /// Kind is the transaction kind (and data) expected
    kind: TransactionKind,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
/// Pwm mock channel
pub struct Channel {}

impl Transaction {
    /// Create a new pwm transaction
    pub fn new(kind: TransactionKind) -> Transaction {
        Transaction { kind }
    }

    /// Create a new disable transaction
    pub fn disable() -> Transaction {
        Transaction::new(TransactionKind::Disable)
    }

    /// Create a new enable transaction
    pub fn enable() -> Transaction {
        Transaction::new(TransactionKind::Enable)
    }

    /// Create a new get_period transaction
    pub fn get_period(time: PwmTime) -> Transaction {
        Transaction::new(TransactionKind::GetPeriod(time))
    }

    /// Create a new get_period transaction
    pub fn set_period(time: PwmTime) -> Transaction {
        Transaction::new(TransactionKind::SetPeriod(time))
    }

    /// Create a new get_duty transaction
    pub fn get_duty(duty: PwmDuty) -> Transaction {
        Transaction::new(TransactionKind::GetDuty(duty))
    }

    /// Create a new get_max_duty transaction
    pub fn get_max_duty(max_duty: PwmDuty) -> Transaction {
        Transaction::new(TransactionKind::GetMaxDuty(max_duty))
    }

    /// Create a new set_duty transaction
    pub fn set_duty(expected_duty: PwmDuty) -> Transaction {
        Transaction::new(TransactionKind::SetDuty(expected_duty))
    }
}

/// MockPwm transaction kind.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum TransactionKind {
    /// Disable a [`Pwm`] using [`Pwm::disable`]
    Disable,
    /// Enable a [`Pwm`] using [`Pwm::enable`]
    Enable,
    /// Query the duty of a [`Pwm`] using [`Pwm::get_period`], returning the specified value
    GetPeriod(PwmTime),
    /// Query the duty of a [`Pwm`] using [`Pwm::set_period`], returning the specified value
    SetPeriod(PwmTime),
    /// Query the duty of a [`Pwm`] using [`Pwm::get_duty`], returning the specified value
    GetDuty(PwmDuty),
    /// Query the max. duty of a [`Pwm`] using [`Pwm::get_max_duty`], returning the specified value
    GetMaxDuty(PwmDuty),
    /// Set the duty of a [`Pwm`] using [`Pwm::set_duty`], expecting the specified value
    SetDuty(PwmDuty),
}

/// Mock pwm implementation
pub type Mock = Generic<Transaction>;

impl Pwm for Mock {
    type Channel = Channel;
    type Time = PwmTime;
    type Duty = PwmDuty;

    fn disable(&mut self, _channel: Self::Channel) {
        // Note: Error is being ignored, because method doesn't return a result
        let Transaction { kind, .. } = self.next().expect("no expectation for pwm::disable call");

        assert_eq!(kind, TransactionKind::Disable, "expected pwm::disable");
    }

    fn enable(&mut self, _channel: Self::Channel) {
        // Note: Error is being ignored, because method doesn't return a result
        let Transaction { kind, .. } = self.next().expect("no expectation for pwm::enable call");

        assert_eq!(kind, TransactionKind::Enable, "expected pwm::enable");
    }

    fn get_period(&self) -> Self::Time {
        let mut s = self.clone();

        // Note: Error is being ignored, because method doesn't return a result
        let Transaction { kind, .. } = s.next().expect("no expectation for pwm::get_duty call");

        if let TransactionKind::GetPeriod(time) = kind {
            time
        } else {
            panic!("expected pwm::get_duty");
        }
    }

    fn get_duty(&self, _channel: Self::Channel) -> Self::Duty {
        let mut s = self.clone();

        // Note: Error is being ignored, because method doesn't return a result
        let Transaction { kind, .. } = s.next().expect("no expectation for pwm::get_duty call");

        if let TransactionKind::GetDuty(duty) = kind {
            duty
        } else {
            panic!("expected pwm::get_duty");
        }
    }

    fn get_max_duty(&self) -> Self::Duty {
        let mut s = self.clone();

        // Note: Error is being ignored, because method doesn't return a result
        let Transaction { kind, .. } = s.next().expect("no expectation for pwm::get_max_duty call");

        if let TransactionKind::GetMaxDuty(max_duty) = kind {
            max_duty
        } else {
            panic!("expected pwm::get_max_duty");
        }
    }

    fn set_duty(&mut self, _channel: Self::Channel, duty: Self::Duty) {
        // Note: Error is being ignored, because method doesn't return a result
        let Transaction { kind, .. } = self.next().expect("no expectation for pwm::set_duty call");

        assert_eq!(
            kind,
            TransactionKind::SetDuty(duty),
            "expected pwm::set_duty"
        );
    }

    fn set_period<P>(&mut self, period: P)
    where
        P: Into<Self::Time>,
    {
        // Note: Error is being ignored, because method doesn't return a result
        let Transaction { kind, .. } = self.next().expect("no expectation for pwm::set_duty call");

        assert_eq!(
            kind,
            TransactionKind::SetPeriod(period.into()),
            "expected pwm::set_duty"
        );
    }
}

#[cfg(test)]
mod test {
    use super::TransactionKind::*;
    use super::*;

    use eh0 as embedded_hal;
    use embedded_hal::Pwm;

    #[test]
    fn test_pwm() {
        let channel = Channel {};
        let expected_time = 20_000;
        let expected_duty = 10_000;
        let expectations = [
            Transaction::new(SetPeriod(expected_time)),
            Transaction::new(GetPeriod(expected_time)),
            Transaction::new(Enable),
            Transaction::new(GetMaxDuty(expected_duty)),
            Transaction::new(SetDuty(expected_duty)),
            Transaction::new(GetDuty(expected_duty)),
            Transaction::new(Disable),
        ];
        let mut pwm = Mock::new(&expectations);

        pwm.set_period(expected_time);
        assert_eq!(pwm.get_period(), expected_time);

        pwm.enable(channel);
        let max_duty = pwm.get_max_duty();
        pwm.set_duty(channel, max_duty);
        assert_eq!(pwm.get_duty(channel), expected_duty);
        pwm.disable(channel);

        pwm.done();
    }
}
