//! ADC mock implementation.
//!
//! ## Usage
//!
//! ```
//! # use eh0 as embedded_hal;
//! use embedded_hal::adc::OneShot;
//! use embedded_hal_mock::eh0::adc::{Mock, MockChan0, MockChan1, Transaction};
//!
//! // Configure expectations: expected input channel numbers and values returned by read operations
//! let expectations = [
//!     Transaction::read(0, 0xab),
//!     Transaction::read(1, 0xabcd)
//! ];
//! let mut adc = Mock::new(&expectations);
//!
//! // Reading
//! assert_eq!(0xab, adc.read(&mut MockChan0 {}).unwrap());
//! assert_eq!(0xabcd, adc.read(&mut MockChan1 {}).unwrap());
//!
//! // Finalise expectations
//! adc.done();
//! ```
//!
//! ## Testing Error Handling
//!
//! Attach an error to test error handling. An error is returned when such a transaction is executed.
//!
//! ```
//! # use eh0 as embedded_hal;
//! use std::io::ErrorKind;
//!
//! use embedded_hal::adc::OneShot;
//! use embedded_hal_mock::eh0::{
//!     adc::{Mock, MockChan1, Transaction},
//!     MockError,
//! };
//!
//! // Configure expectations
//! let expectations = [
//!     Transaction::read(1, 0xabba).with_error(MockError::Io(ErrorKind::InvalidData))
//! ];
//! let mut adc = Mock::new(&expectations);
//!
//! // Reading returns an error
//! adc.read(&mut MockChan1 {})
//!     .expect_err("expected error return");
//!
//! // Finalise expectations
//! adc.done();
//! ```

use std::fmt::Debug;

use eh0 as embedded_hal;
use embedded_hal::adc::{Channel, OneShot};
use nb;

use super::error::MockError;
use crate::common::Generic;

/// ADC transaction type
///
/// Models an ADC read
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transaction<T> {
    expected_chan: u8,
    response: T,
    /// An optional error return for a transaction.
    err: Option<MockError>,
}

impl<T> Transaction<T> {
    /// Create a read transaction
    pub fn read(chan: u8, resp: T) -> Transaction<T> {
        Transaction {
            expected_chan: chan,
            response: resp,
            err: None,
        }
    }

    /// Add an error return to a transaction.
    ///
    /// This is used to mock failure behaviour.
    pub fn with_error(mut self, error: MockError) -> Self {
        self.err = Some(error);
        self
    }
}

/// Mock ADC implementation
pub struct MockAdc;

macro_rules! mock_channel {
    ($ADC:ident, $($pin:ident => $chan:expr),+ $(,)*) => {
        $(
            /// Mock ADC channel implementation
            #[derive(Clone, Debug, PartialEq, Eq)]
            pub struct $pin;

            impl Channel<$ADC> for $pin {
                type ID = u8;

                fn channel() -> u8 { $chan }
            }
        )+
    };
}

mock_channel!(MockAdc,
    MockChan0 => 0_u8,
    MockChan1 => 1_u8,
    MockChan2 => 2_u8,
);

/// Mock ADC implementation
///
/// Mock ADC implements OneShot trait reading operation. Returned type can be either derived from
/// definition of expectations or specified explicitly. Explicit ADC read return type can be used
/// to mock specific ADC accuracy.
pub type Mock<T> = Generic<Transaction<T>>;

impl<Pin, T> OneShot<MockAdc, T, Pin> for Mock<T>
where
    Pin: Channel<MockAdc, ID = u8>,
    T: Clone + Debug + PartialEq,
{
    type Error = MockError;

    fn read(&mut self, _pin: &mut Pin) -> nb::Result<T, Self::Error> {
        let w = self.next().expect("unexpected read call");
        assert_eq!(w.expected_chan, Pin::channel(), "unexpected channel");
        match w.err {
            Some(e) => Err(nb::Error::Other(e)),
            None => Ok(w.response),
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::ErrorKind;

    use eh0 as embedded_hal;
    use embedded_hal::adc::OneShot;

    use super::{super::error::MockError, *};

    #[test]
    fn test_adc_single_read16() {
        let expectations = [Transaction::read(0, 0xabcdu16)];
        let mut adc = Mock::new(&expectations);

        assert_eq!(0xabcdu16, adc.read(&mut MockChan0 {}).unwrap());

        adc.done();
    }

    #[test]
    fn test_adc_single_read32() {
        let expectations = [Transaction::read(0, 0xabcdabcdu32)];
        let mut adc = Mock::new(&expectations);

        assert_eq!(0xabcdabcdu32, adc.read(&mut MockChan0 {}).unwrap());

        adc.done();
    }

    #[test]
    fn test_adc_mult_read() {
        let expectations = [
            Transaction::read(0, 0xabcd),
            Transaction::read(1, 0xabba),
            Transaction::read(2, 0xbaab),
        ];
        let mut adc = Mock::new(&expectations);

        assert_eq!(0xabcd, adc.read(&mut MockChan0 {}).unwrap());
        assert_eq!(0xabba, adc.read(&mut MockChan1 {}).unwrap());
        assert_eq!(0xbaab, adc.read(&mut MockChan2 {}).unwrap());

        adc.done();
    }

    #[test]
    fn test_adc_err_read() {
        let expectations = [
            Transaction::read(0, 0xabcd),
            Transaction::read(1, 0xabba).with_error(MockError::Io(ErrorKind::InvalidData)),
        ];
        let mut adc = Mock::new(&expectations);

        assert_eq!(0xabcd, adc.read(&mut MockChan0 {}).unwrap());
        adc.read(&mut MockChan1 {})
            .expect_err("expected error return");

        adc.done();
    }
}
