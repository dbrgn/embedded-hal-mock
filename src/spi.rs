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
//! use embedded_hal::spi::blocking::{TransferInplace, Write};
//! use embedded_hal_mock::spi::{Mock as SpiMock, Transaction as SpiTransaction};
//! use embedded_hal::spi::nb::FullDuplex;
//!
//! // Configure expectations
//! let expectations = [
//!     SpiTransaction::write(0x09),
//!     SpiTransaction::read(0x0A),
//!     SpiTransaction::write(0xFE),
//!     SpiTransaction::read(0xFF),
//!     SpiTransaction::write_vec(vec![1, 2]),
//!     SpiTransaction::transfer_inplace(vec![3, 4], vec![5, 6]),
//! ];
//!
//! let mut spi = SpiMock::new(&expectations);
//! // FullDuplex transfers
//! FullDuplex::write(&mut spi, 0x09);
//! assert_eq!(spi.read().unwrap(), 0x0A);
//! FullDuplex::write(&mut spi, 0xFE);
//! assert_eq!(spi.read().unwrap(), 0xFF);
//!
//! // Writing
//! Write::write(&mut spi, &vec![1, 2]).unwrap();
//!
//! // Transferring
//! let mut buf = vec![3, 4];
//! spi.transfer_inplace(&mut buf).unwrap();
//! assert_eq!(buf, vec![5, 6]);
//!
//! // Finalise expectations
//! spi.done();
//! ```
use embedded_hal::spi::blocking as spi;
use embedded_hal::spi::nb::FullDuplex;
use embedded_hal::spi::ErrorKind;
use nb;

use crate::common::Generic;

/// SPI Transaction mode
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Write transaction
    Write,
    /// Write and read transaction
    Transfer,
    /// After a write transaction in real HW a Read is available
    Read,
}

/// SPI transaction type
///
/// Models an SPI write or transfer (with response)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    expected_mode: Mode,
    expected_data: Vec<u8>,
    response: Vec<u8>,
}

impl Transaction {
    /// Create a write transaction
    pub fn write_vec(expected: Vec<u8>) -> Transaction {
        Transaction {
            expected_mode: Mode::Write,
            expected_data: expected,
            response: Vec::new(),
        }
    }

    /// Create a transfer transaction
    pub fn transfer_inplace(expected: Vec<u8>, response: Vec<u8>) -> Transaction {
        Transaction {
            expected_mode: Mode::Transfer,
            expected_data: expected,
            response,
        }
    }

    /// Create a transfer transaction
    pub fn write(expected: u8) -> Transaction {
        Transaction {
            expected_mode: Mode::Write,
            expected_data: [expected].to_vec(),
            response: Vec::new(),
        }
    }

    /// Create a transfer transaction
    pub fn read(response: u8) -> Transaction {
        Transaction {
            expected_mode: Mode::Read,
            expected_data: Vec::new(),
            response: [response].to_vec(),
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
pub type Mock = Generic<Transaction>;

impl spi::Write<u8> for Mock {
    type Error = ErrorKind;

    /// spi::Write implementation for Mock
    ///
    /// This will cause an assertion if the write call does not match the next expectation
    fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        let w = self.next().expect("no expectation for spi::write call");
        assert_eq!(w.expected_mode, Mode::Write, "spi::write unexpected mode");
        assert_eq!(
            &w.expected_data, &buffer,
            "spi::write data does not match expectation"
        );
        Ok(())
    }
}

impl FullDuplex<u8> for Mock {
    type Error = ErrorKind;

    /// spi::FullDuplex implementeation for Mock
    ///
    /// This will call the nonblocking read/write primitives.
    fn write(&mut self, buffer: u8) -> nb::Result<(), Self::Error> {
        let data = self.next().expect("no expectation for spi::write call");
        assert_eq!(
            data.expected_mode,
            Mode::Write,
            "spi::write unexpected mode"
        );
        assert_eq!(
            data.expected_data[0], buffer,
            "spi::write data does not match expectation"
        );
        Ok(())
    }

    /// spi::FullDuplex implementeation for Mock
    ///
    /// This will call the nonblocking read/write primitives.
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let w = self.next().expect("no expectation for spi::read call");
        assert_eq!(w.expected_mode, Mode::Read, "spi::Read unexpected mode");
        assert_eq!(
            1,
            w.response.len(),
            "mismatched response length for spi::read"
        );
        let buffer: u8 = w.response[0];
        Ok(buffer)
    }
}

impl spi::TransferInplace<u8> for Mock {
    type Error = ErrorKind;

    /// spi::TransferInplace implementation for Mock
    ///
    /// This writes the provided response to the buffer and will cause an assertion if the written data does not match the next expectation
    fn transfer_inplace<'w>(&mut self, buffer: &'w mut [u8]) -> Result<(), Self::Error> {
        let w = self.next().expect("no expectation for spi::transfer call");
        assert_eq!(
            w.expected_mode,
            Mode::Transfer,
            "spi::transfer unexpected mode"
        );
        assert_eq!(
            &w.expected_data, &buffer,
            "spi::write data does not match expectation"
        );
        assert_eq!(
            buffer.len(),
            w.response.len(),
            "mismatched response length for spi::transfer"
        );
        buffer.copy_from_slice(&w.response);
        Ok(())
    }
}

impl spi::WriteIter<u8> for Mock {
    type Error = ErrorKind;

    fn write_iter<WI>(&mut self, words: WI) -> Result<(), Self::Error>
    where
        WI: IntoIterator<Item = u8>,
    {
        let w = self
            .next()
            .expect("no expectation for spi::write_iter call");
        let buffer = words.into_iter().collect::<Vec<_>>();
        assert_eq!(
            w.expected_mode,
            Mode::Write,
            "spi::write_iter unexpected mode"
        );
        assert_eq!(
            &w.expected_data, &buffer,
            "spi::write_iter data does not match expectation"
        );
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use embedded_hal::spi::blocking::{TransferInplace, Write, WriteIter};

    #[test]
    fn test_spi_mock_write() {
        let mut spi = Mock::new(&[Transaction::write(10)]);

        let _ = Write::write(&mut spi, &[10]).unwrap();

        spi.done();
    }

    #[test]
    fn test_spi_mock_read() {
        let mut spi = Mock::new(&[Transaction::read(10)]);

        let ans = spi.read().unwrap();

        assert_eq!(ans, 10);

        spi.done();
    }

    #[test]
    fn test_spi_mock_multiple1() {
        let expectations = [
            Transaction::write_vec(vec![1, 2]),
            Transaction::write(9),
            Transaction::read(10),
            Transaction::write(0xFE),
            Transaction::read(0xFF),
            Transaction::transfer_inplace(vec![3, 4], vec![5, 6]),
        ];
        let mut spi = Mock::new(&expectations);

        Write::write(&mut spi, &vec![1, 2]).unwrap();

        let _ = Write::write(&mut spi, &[0x09]);
        assert_eq!(spi.read().unwrap(), 0x0a);
        let _ = Write::write(&mut spi, &[0xfe]);
        assert_eq!(spi.read().unwrap(), 0xFF);
        let mut v = vec![3, 4];
        spi.transfer_inplace(&mut v).unwrap();

        assert_eq!(v, vec![5, 6]);

        spi.done();
    }

    #[test]
    fn test_spi_mock_write_vec() {
        let expectations = [Transaction::write_vec(vec![10, 12])];
        let mut spi = Mock::new(&expectations);

        Write::write(&mut spi, &vec![10, 12]).unwrap();

        spi.done();
    }

    #[test]
    fn test_spi_mock_write_iter() {
        let expectations = [Transaction::write_vec(vec![10, 12])];
        let mut spi = Mock::new(&expectations);

        spi.write_iter(vec![10, 12u8]).unwrap();

        spi.done();
    }

    #[test]
    fn test_spi_mock_transfer_inplace() {
        let expectations = [Transaction::transfer_inplace(vec![10, 12], vec![12, 13])];
        let mut spi = Mock::new(&expectations);

        let mut v = vec![10, 12];
        spi.transfer_inplace(&mut v).unwrap();

        assert_eq!(v, vec![12, 13]);

        spi.done();
    }

    #[test]
    fn test_spi_mock_multiple() {
        let expectations = [
            Transaction::write_vec(vec![1, 2]),
            Transaction::transfer_inplace(vec![3, 4], vec![5, 6]),
        ];
        let mut spi = Mock::new(&expectations);

        Write::write(&mut spi, &vec![1, 2]).unwrap();

        let mut v = vec![3, 4];
        spi.transfer_inplace(&mut v).unwrap();

        assert_eq!(v, vec![5, 6]);

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_write_err() {
        let expectations = [Transaction::write_vec(vec![10, 12])];
        let mut spi = Mock::new(&expectations);

        Write::write(&mut spi, &vec![10, 12, 12]).unwrap();

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_write_iter_err() {
        let expectations = [Transaction::write_vec(vec![10, 12])];
        let mut spi = Mock::new(&expectations);

        spi.write_iter(vec![10, 12, 12u8]).unwrap();

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_transfer_err() {
        let expectations = [Transaction::transfer_inplace(vec![10, 12], vec![12, 15])];
        let mut spi = Mock::new(&expectations);

        let mut v = vec![10, 12];
        spi.transfer_inplace(&mut v).unwrap();

        assert_eq!(v, vec![12, 13]);

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_transfer_response_err() {
        let expectations = [Transaction::transfer_inplace(vec![1, 2], vec![3, 4, 5])];
        let mut spi = Mock::new(&expectations);

        let mut v = vec![10, 12];
        spi.transfer_inplace(&mut v).unwrap();

        assert_eq!(v, vec![12, 13]);

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_mode_err() {
        let expectations = [Transaction::transfer_inplace(vec![10, 12], vec![])];
        let mut spi = Mock::new(&expectations);

        Write::write(&mut spi, &vec![10, 12, 12]).unwrap();

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_multiple_transaction_err() {
        let expectations = [
            Transaction::write_vec(vec![10, 12]),
            Transaction::write_vec(vec![10, 12]),
        ];
        let mut spi = Mock::new(&expectations);

        Write::write(&mut spi, &vec![10, 12, 12]).unwrap();

        spi.done();
    }
}
