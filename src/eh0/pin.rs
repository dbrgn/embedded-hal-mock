//! Mock digital [`InputPin`] and [`OutputPin`] v2 implementations
//!
//! [`InputPin`]: https://docs.rs/embedded-hal/0.2/embedded_hal/digital/v2/trait.InputPin.html
//! [`OutputPin`]: https://docs.rs/embedded-hal/0.2/embedded_hal/digital/v2/trait.OutputPin.html
//!
//! ```
//! # use eh0 as embedded_hal;
//! use std::io::ErrorKind;
//!
//! use embedded_hal::digital::v2::{InputPin, OutputPin};
//! use embedded_hal_mock::eh0::{
//!     pin::{Mock as PinMock, State as PinState, Transaction as PinTransaction},
//!     MockError,
//! };
//!
//! let err = MockError::Io(ErrorKind::NotConnected);
//!
//! // Configure expectations
//! let expectations = [
//!     PinTransaction::get(PinState::High),
//!     PinTransaction::get(PinState::High),
//!     PinTransaction::set(PinState::Low),
//!     PinTransaction::set(PinState::High).with_error(err.clone()),
//! ];
//!
//! // Create pin
//! let mut pin = PinMock::new(&expectations);
//!
//! // Run and test
//! assert_eq!(pin.is_high().unwrap(), true);
//! assert_eq!(pin.is_low().unwrap(), false);
//!
//! pin.set_low().unwrap();
//! pin.set_high().expect_err("expected error return");
//!
//! pin.done();
//!
//! // Update expectations
//! pin.update_expectations(&[]);
//! // ...
//! pin.done();
//! ```

use eh0 as embedded_hal;
use embedded_hal::{
    digital::v2::{InputPin, OutputPin},
    PwmPin,
};

use super::error::MockError;
use crate::common::Generic;

/// The type used for the duty of the [`PwmPin`] mock.
pub type PwmDuty = u16;

/// MockPin transaction
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Transaction {
    /// Kind is the transaction kind (and data) expected
    kind: TransactionKind,
    /// Err is an optional error return for a transaction.
    /// This is in addition to kind to allow validation that the transaction kind
    /// is correct prior to returning the error.
    err: Option<MockError>,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
/// Digital pin value enumeration
pub enum State {
    /// Digital low state
    Low,
    /// Digital high state
    High,
}

impl Transaction {
    /// Create a new pin transaction
    pub fn new(kind: TransactionKind) -> Transaction {
        Transaction { kind, err: None }
    }

    /// Create a new get transaction
    pub fn get(state: State) -> Transaction {
        Transaction::new(TransactionKind::Get(state))
    }

    /// Create a new get transaction
    pub fn set(state: State) -> Transaction {
        Transaction::new(TransactionKind::Set(state))
    }

    /// Create a new disable transaction
    pub fn disable() -> Transaction {
        Transaction::new(TransactionKind::Disable)
    }

    /// Create a new enable transaction
    pub fn enable() -> Transaction {
        Transaction::new(TransactionKind::Enable)
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

    /// Add an error return to a transaction
    ///
    /// This is used to mock failure behaviours.
    ///
    /// Note that this can only be used for methods which actually return a [`Result`];
    /// trying to invoke this for others will lead to an assertion error!
    pub fn with_error(mut self, error: MockError) -> Self {
        assert!(
            self.kind.supports_errors(),
            "the transaction kind supports errors"
        );
        self.err = Some(error);
        self
    }
}

/// MockPin transaction kind.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum TransactionKind {
    /// Set the pin state
    Set(State),
    /// Get the pin state
    Get(State),
    /// Disable a [`PwmPin`] using [`PwmPin::disable`]
    Disable,
    /// Enable a [`PwmPin`] using [`PwmPin::enable`]
    Enable,
    /// Query the duty of a [`PwmPin`] using [`PwmPin::get_duty`], returning the specified value
    GetDuty(PwmDuty),
    /// Query the max. duty of a [`PwmPin`] using [`PwmPin::get_max_duty`], returning the specified value
    GetMaxDuty(PwmDuty),
    /// Set the duty of a [`PwmPin`] using [`PwmPin::set_duty`], expecting the specified value
    SetDuty(PwmDuty),
}

impl TransactionKind {
    fn is_get(&self) -> bool {
        match self {
            TransactionKind::Get(_) => true,
            _ => false,
        }
    }

    /// Specifies whether the actual API returns a [`Result`] (= supports errors) or not.
    fn supports_errors(&self) -> bool {
        match self {
            TransactionKind::Set(_) | TransactionKind::Get(_) => true,
            _ => false,
        }
    }
}

/// Mock Pin implementation
pub type Mock = Generic<Transaction>;

/// Single digital push-pull output pin
impl OutputPin for Mock {
    /// Error type
    type Error = MockError;

    /// Drives the pin low
    fn set_low(&mut self) -> Result<(), Self::Error> {
        let Transaction { kind, err } = self.next().expect("no expectation for pin::set_low call");

        assert_eq!(
            kind,
            TransactionKind::Set(State::Low),
            "expected pin::set_low"
        );

        match err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    /// Drives the pin high
    fn set_high(&mut self) -> Result<(), Self::Error> {
        let Transaction { kind, err } = self.next().expect("no expectation for pin::set_high call");

        assert_eq!(
            kind,
            TransactionKind::Set(State::High),
            "expected pin::set_high"
        );

        match err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}

impl InputPin for Mock {
    /// Error type
    type Error = MockError;

    /// Is the input pin high?
    fn is_high(&self) -> Result<bool, Self::Error> {
        let mut s = self.clone();

        let Transaction { kind, err } = s.next().expect("no expectation for pin::is_high call");

        assert!(kind.is_get(), "expected pin::get");

        if let Some(e) = err {
            Err(e)
        } else if let TransactionKind::Get(v) = kind {
            Ok(v == State::High)
        } else {
            unreachable!();
        }
    }

    /// Is the input pin low?
    fn is_low(&self) -> Result<bool, Self::Error> {
        let mut s = self.clone();

        let Transaction { kind, err } = s.next().expect("no expectation for pin::is_low call");

        assert!(kind.is_get(), "expected pin::get");

        if let Some(e) = err {
            Err(e)
        } else if let TransactionKind::Get(v) = kind {
            Ok(v == State::Low)
        } else {
            unreachable!();
        }
    }
}

impl PwmPin for Mock {
    type Duty = PwmDuty;

    fn disable(&mut self) {
        // Note: Error is being ignored, because method doesn't return a result
        let Transaction { kind, .. } = self.next().expect("no expectation for pin::disable call");

        assert_eq!(kind, TransactionKind::Disable, "expected pin::disable");
    }

    fn enable(&mut self) {
        // Note: Error is being ignored, because method doesn't return a result
        let Transaction { kind, .. } = self.next().expect("no expectation for pin::enable call");

        assert_eq!(kind, TransactionKind::Enable, "expected pin::enable");
    }

    fn get_duty(&self) -> Self::Duty {
        let mut s = self.clone();

        // Note: Error is being ignored, because method doesn't return a result
        let Transaction { kind, .. } = s.next().expect("no expectation for pin::get_duty call");

        if let TransactionKind::GetDuty(duty) = kind {
            duty
        } else {
            panic!("expected pin::get_duty");
        }
    }

    fn get_max_duty(&self) -> Self::Duty {
        let mut s = self.clone();

        // Note: Error is being ignored, because method doesn't return a result
        let Transaction { kind, .. } = s.next().expect("no expectation for pin::get_max_duty call");

        if let TransactionKind::GetMaxDuty(max_duty) = kind {
            max_duty
        } else {
            panic!("expected pin::get_max_duty");
        }
    }

    fn set_duty(&mut self, duty: Self::Duty) {
        // Note: Error is being ignored, because method doesn't return a result
        let Transaction { kind, .. } = self.next().expect("no expectation for pin::set_duty call");

        assert_eq!(
            kind,
            TransactionKind::SetDuty(duty),
            "expected pin::set_duty"
        );
    }
}

#[cfg(test)]
mod test {
    use std::io::ErrorKind;

    use eh0 as embedded_hal;
    use embedded_hal::{
        digital::v2::{InputPin, OutputPin},
        PwmPin,
    };

    use super::{super::error::MockError, TransactionKind::*, *};

    #[test]
    fn test_input_pin() {
        let expectations = [
            Transaction::new(Get(State::High)),
            Transaction::new(Get(State::High)),
            Transaction::new(Get(State::Low)),
            Transaction::new(Get(State::Low)),
            Transaction::new(Get(State::High)).with_error(MockError::Io(ErrorKind::NotConnected)),
        ];
        let mut pin = Mock::new(&expectations);

        assert_eq!(pin.is_high().unwrap(), true);
        assert_eq!(pin.is_low().unwrap(), false);
        assert_eq!(pin.is_high().unwrap(), false);
        assert_eq!(pin.is_low().unwrap(), true);

        pin.is_low().expect_err("expected error return");

        pin.done();
    }

    #[test]
    fn test_output_pin() {
        let expectations = [
            Transaction::new(Set(State::High)),
            Transaction::new(Set(State::Low)),
            Transaction::new(Set(State::High)).with_error(MockError::Io(ErrorKind::NotConnected)),
        ];
        let mut pin = Mock::new(&expectations);

        pin.set_high().unwrap();
        pin.set_low().unwrap();

        pin.set_high().expect_err("expected error return");

        pin.done();
    }

    #[test]
    fn test_pwm_pin() {
        let expected_duty = 10_000;
        let expectations = [
            Transaction::new(Enable),
            Transaction::new(GetMaxDuty(expected_duty)),
            Transaction::new(SetDuty(expected_duty)),
            Transaction::new(GetDuty(expected_duty)),
            Transaction::new(Disable),
        ];
        let mut pin = Mock::new(&expectations);

        pin.enable();
        let max_duty = pin.get_max_duty();
        pin.set_duty(max_duty);
        assert_eq!(pin.get_duty(), expected_duty);
        pin.disable();

        pin.done();
    }
}
