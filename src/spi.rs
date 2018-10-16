//! SPI mock implementations.
//!
//! This mock supports the specification and checking of expectations to allow
//! automated testing of SPI based drivers. Mismatches between expected and
//! real SPI transactions will cause runtime assertions to assist with locating
//! faults.
//!
//! ## Usage
//!
//! ```
//! extern crate embedded_hal;
//! extern crate embedded_hal_mock;
//!
//! use embedded_hal::blocking::spi::{Transfer, Write};
//! use embedded_hal_mock::spi::{Mock as SpiMock, Transaction as SpiTransaction};
//!
//! // Configure expectations
//! let expectations = [
//!     SpiTransaction::write(vec![1, 2]),
//!     SpiTransaction::transfer(vec![3, 4], vec![5, 6]),
//! ];
//!
//! let mut spi = SpiMock::new(&expectations);
//!
//! // Writing
//! spi.write(&vec![1, 2]).unwrap();
//!
//! // Transferring
//! let mut buf = vec![3, 4];
//! spi.transfer(&mut buf).unwrap();
//! assert_eq!(buf, vec![5, 6]);
//!
//! // Finalise expectations
//! spi.done();
//! ```

use hal::blocking::spi;

use common::Generic;
use error::MockError;

/// SPI Transaction mode
#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Write,
    Transfer,
}

/// SPI transaction type
///
/// Models an SPI write or transfer (with response)
#[derive(Clone, Debug, PartialEq)]
pub struct Transaction {
    expected_mode: Mode,
    expected_data: Vec<u8>,
    response: Vec<u8>,
}

impl Transaction {
    /// Create a write transaction
    pub fn write(expected: Vec<u8>) -> Transaction {
        Transaction {
            expected_mode: Mode::Write,
            expected_data: expected,
            response: Vec::new(),
        }
    }

    /// Create a transfer transaction
    pub fn transfer(expected: Vec<u8>, response: Vec<u8>) -> Transaction {
        Transaction {
            expected_mode: Mode::Transfer,
            expected_data: expected,
            response,
        }
    }
}

/// Mock SPI implementation
///
/// This supports the specification and checking of expectations to allow
/// automated testing of SPI based drivers. Mismatches between expected and
/// real SPI transactions will cause runtime assertions to assist with locating
/// faults.
///
/// See the usage section in the module level docs for an example.
pub type Mock<'a> = Generic<'a, Transaction>;

impl <'a>spi::Write<u8> for Mock<'a> {
    type Error = MockError;

    /// spi::Write implementation for Mock
    ///
    /// This will cause an assertion if the write call does not match the next expectation
    fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        let w = self.next().expect("no expectation for spi::write call");
        assert_eq!(w.expected_mode, Mode::Write, "spi::write unexpected mode");
        assert_eq!(&w.expected_data, &buffer, "spi::write data does not match expectation");
        Ok(())
    }
}

impl <'a>spi::Transfer<u8> for Mock<'a> {
    type Error = MockError;

    /// spi::Transfer implementation for Mock
    ///
    /// This writes the provided response to the buffer and will cause an assertion if the written data does not match the next expectation
    fn transfer<'w>(&mut self, buffer: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        let w = self.next().expect("no expectation for spi::transfer call");
        assert_eq!(w.expected_mode, Mode::Transfer, "spi::transfer unexpected mode");
        assert_eq!(&w.expected_data, &buffer, "spi::write data does not match expectation");
        assert_eq!(buffer.len(), w.response.len(), "mismatched response length for spi::transfer");
        buffer.copy_from_slice(&w.response);
        Ok(buffer)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use hal::blocking::spi::{Transfer, Write};

    #[test]
    fn test_spi_mock_write() {
        let expectations = [Transaction::write(vec![10, 12])];
        let mut spi = Mock::new(&expectations);

        spi.write(&vec![10, 12]).unwrap();

        spi.done();
    }

    #[test]
    fn test_spi_mock_transfer() {
        let expectations = [Transaction::transfer(
            vec![10, 12],
            vec![12, 13],
        )];
        let mut spi = Mock::new(&expectations);

        let mut v = vec![10, 12];
        spi.transfer(&mut v).unwrap();

        assert_eq!(v, vec![12, 13]);

        spi.done();
    }

    #[test]
    fn test_spi_mock_multiple() {
        let expectations = [
            Transaction::write(vec![1, 2]),
            Transaction::transfer(vec![3, 4], vec![5, 6]),
        ];
        let mut spi = Mock::new(&expectations);

        spi.write(&vec![1, 2]).unwrap();

        let mut v = vec![3, 4];
        spi.transfer(&mut v).unwrap();

        assert_eq!(v, vec![5, 6]);

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_write_err() {
        let expectations = [Transaction::write(vec![10, 12])];
        let mut spi = Mock::new(&expectations);

        spi.write(&vec![10, 12, 12]).unwrap();

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_transfer_err() {
        let expectations = [Transaction::transfer(
            vec![10, 12],
            vec![12, 15],
        )];
        let mut spi = Mock::new(&expectations);

        let mut v = vec![10, 12];
        spi.transfer(&mut v).unwrap();

        assert_eq!(v, vec![12, 13]);

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_transfer_response_err() {
        let expectations = [Transaction::transfer(
            vec![1, 2],
            vec![3, 4, 5],
        )];
        let mut spi = Mock::new(&expectations);

        let mut v = vec![10, 12];
        spi.transfer(&mut v).unwrap();

        assert_eq!(v, vec![12, 13]);

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_mode_err() {
        let expectations = [Transaction::transfer(vec![10, 12], vec![])];
let mut spi = Mock::new(&expectations);

        spi.write(&vec![10, 12, 12]).unwrap();

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_multiple_transaction_err() {
        let expectations = [
            Transaction::write(vec![10, 12]),
            Transaction::write(vec![10, 12]),
        ];
        let mut spi = Mock::new(&expectations);


        spi.write(&vec![10, 12, 12]).unwrap();

        spi.done();
    }
}
