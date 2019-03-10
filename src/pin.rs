//! Mock digital InputPin and OutputPin implementations
//! 
//! ```
//! use std::io::ErrorKind;
//! 
//! use embedded_hal_mock::MockError;
//! use embedded_hal_mock::pin::{Transaction as PinTransaction, Mock as MockPin};
//! use embedded_hal::digital::v2::{InputPin, OutputPin};
//! 
//! let err = MockError::Io(ErrorKind::NotConnected);
//! 
//! // Configure expectations
//! let expectations = [
//!     PinTransaction::get(true),
//!     PinTransaction::get(true),
//!     PinTransaction::set(false),
//!     PinTransaction::set(true).with_error(err.clone()),
//! ];
//! 
//! // Create pin
//! let mut pin = MockPin::new(&expectations);
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
//! pin.expect(&[]);
//! // ...
//! pin.done();
//! 
//! ```


use crate::common::Generic;
use crate::error::MockError;

use embedded_hal::digital::v2::{OutputPin, InputPin};

/// MockPin transaction
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
    /// Create a new pin transaction
    pub fn new(kind: TransactionKind) -> Transaction {
        Transaction{kind, err: None}
    }

    /// Create a new get transaction
    pub fn get(value: bool) -> Transaction {
        Transaction::new(TransactionKind::Get(value))
    }

    /// Create a new get transaction
    pub fn set(value: bool) -> Transaction {
        Transaction::new(TransactionKind::Set(value))
    }

    /// Add an error return to a transaction
    /// This is used to mock failure behaviours
    pub fn with_error(self, error: MockError) -> Self {
        let mut t = self;
        t.err = Some(error);
        t
    }
}

/// MockPin transaction kind, either Get or Set a bool
#[derive(PartialEq, Clone, Debug)]
pub enum TransactionKind {
    /// Set(true) for set_high or Set(false) for set_low
    Set(bool),
    /// Get(true) for high value or Get(false) for low value
    Get(bool),
}

impl TransactionKind {
    fn is_get(&self) -> bool {
        match self {
            TransactionKind::Get(_) => true,
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
        let Transaction{kind, err} = self.next().expect("no expectation for pin::set_low call");

        assert_eq!(kind, TransactionKind::Set(false), "expected pin::set_low");
        
        match err {
            Some(e) => Err(e.clone()),
            None => Ok(()),
        }
    }

    /// Drives the pin high
    fn set_high(&mut self) -> Result<(), Self::Error> {
        let Transaction{kind, err} = self.next().expect("no expectation for pin::set_high call");

        assert_eq!(kind, TransactionKind::Set(true), "expected pin::set_high");
        
        match err {
            Some(e) => Err(e.clone()),
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

        let Transaction{kind, err} = s.next().expect("no expectation for pin::is_high call");

        assert_eq!(kind.is_get(), true, "expected pin::get");

        if let Some(e) = err { 
            Err(e.clone())
        } else if let TransactionKind::Get(v) = kind {
            Ok(v == true)
        } else {
            unreachable!();
        }
    }

    /// Is the input pin low?
    fn is_low(&self) -> Result<bool, Self::Error> {
        let mut s = self.clone();

        let Transaction{kind, err} = s.next().expect("no expectation for pin::is_low call");

        assert_eq!(kind.is_get(), true, "expected pin::get");

        if let Some(e) = err { 
            Err(e.clone())
        } else if let TransactionKind::Get(v) = kind {
            Ok(v == false)
        } else {
            unreachable!();
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    use std::io::ErrorKind;

    use embedded_hal::digital::v2::{OutputPin, InputPin};
    use TransactionKind::{Get, Set};

    #[test]
    fn test_input_pin() {
        let expectations = [
            Transaction::new(Get(true)),
            Transaction::new(Get(true)),
            Transaction::new(Get(false)),
            Transaction::new(Get(false)),
            Transaction::new(Get(true)).with_error(MockError::Io(ErrorKind::NotConnected)),
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
            Transaction::new(Set(true)),
            Transaction::new(Set(false)),
            Transaction::new(Set(true)).with_error(MockError::Io(ErrorKind::NotConnected)),
        ];
        let mut pin = Mock::new(&expectations);

        pin.set_high().unwrap();
        pin.set_low().unwrap();
        
        pin.set_high().expect_err("expected error return");

        pin.done();
    }

}