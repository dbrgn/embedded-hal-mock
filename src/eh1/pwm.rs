//! Mock implementations for
//! [`embedded_hal::pwm`](https://docs.rs/embedded-hal/1/embedded_hal/pwm/index.html).
//!
//! Usage example:
//! ```
//! use std::io::ErrorKind;
//!
//! # use eh1 as embedded_hal;
//! use embedded_hal::pwm::SetDutyCycle;
//! use embedded_hal_mock::eh1::{
//!     pwm::{Mock as PwmMock, Transaction as PwmTransaction},
//!     MockError,
//! };
//!
//! // Configure expectations
//! let expectations = [
//!     PwmTransaction::max_duty_cycle(100),
//!     PwmTransaction::set_duty_cycle(50),
//!     PwmTransaction::set_duty_cycle(101).with_error(MockError::Io(ErrorKind::NotConnected)),
//! ];
//!
//! // Create pin
//! let mut pwm = PwmMock::new(&expectations);
//!
//! // Run and test
//! pwm.set_duty_cycle_percent(50).unwrap();
//! pwm.set_duty_cycle(101).expect_err("expected error return");
//!
//! // Finalise expectations
//! pwm.done();
//! ```

use eh1::pwm::{ErrorKind, ErrorType, SetDutyCycle};

use crate::{common::Generic, eh1::MockError};

/// MockPwm transaction
#[derive(PartialEq, Clone, Debug)]
pub struct Transaction {
    /// Kind is the transaction kind (and data) expected
    kind: TransactionKind,
    /// Err is an optional error return for a transaction.
    /// This is in addition to kind to allow validation that the transaction kind
    /// is correct prior to returning the error.
    err: Option<MockError>,
}

impl Transaction {
    /// Create a new PWM transaction
    pub fn new(kind: TransactionKind) -> Transaction {
        Transaction { kind, err: None }
    }

    /// Create a new [`TransactionKind::GetMaxDutyCycle`] transaction for [`SetDutyCycle::max_duty_cycle`].
    pub fn max_duty_cycle(duty: u16) -> Transaction {
        Transaction::new(TransactionKind::GetMaxDutyCycle(duty))
    }

    /// Create a new [`TransactionKind::SetDutyCycle`] transaction for [`SetDutyCycle::set_duty_cycle`].
    pub fn set_duty_cycle(duty: u16) -> Transaction {
        Transaction::new(TransactionKind::SetDutyCycle(duty))
    }

    /// Add an error return to a transaction
    ///
    /// This is used to mock failure behaviours.
    pub fn with_error(mut self, error: MockError) -> Self {
        self.err = Some(error);
        self
    }
}

/// MockPwm transaction kind
#[derive(PartialEq, Clone, Debug)]
pub enum TransactionKind {
    /// [`SetDutyCycle::max_duty_cycle`] which will return the defined duty.
    GetMaxDutyCycle(u16),
    /// [`SetDutyCycle::set_duty_cycle`] with the expected duty.
    SetDutyCycle(u16),
}

/// Mock PWM `SetDutyCycle` implementation
pub type Mock = Generic<Transaction>;

impl eh1::pwm::Error for MockError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

impl ErrorType for Mock {
    type Error = MockError;
}

impl SetDutyCycle for Mock {
    fn max_duty_cycle(&self) -> u16 {
        let mut s = self.clone();

        let Transaction { kind, err } = s.next().expect("no expectation for max_duty_cycle call");

        assert_eq!(err, None, "error not supported by max_duty_cycle!");

        match kind {
            TransactionKind::GetMaxDutyCycle(duty) => duty,
            other => panic!("expected max_duty_cycle, got {:?}", other),
        }
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        let Transaction { kind, err } =
            self.next().expect("no expectation for set_duty_cycle call");

        assert_eq!(
            kind,
            TransactionKind::SetDutyCycle(duty),
            "expected set_duty_cycle"
        );

        if let Some(e) = err {
            Err(e)
        } else {
            Ok(())
        }
    }
}
