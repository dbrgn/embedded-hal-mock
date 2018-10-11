use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use hal::blocking::spi;

use error::MockError;

/// SPI Transaction mode
#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Write,
    Transfer,
}

/// SPI transaction type
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
/// This supports the specification and checking of expectations to allow automated testing of SPI based drivers.
/// Mismatches between expected and real SPI transactions will cause runtime assertions to assist with locating faults.
pub struct SPIMock {
    expected: Arc<Mutex<VecDeque<Transaction>>>,
}

impl SPIMock {
    /// Create a new mock SPI interface
    pub fn new() -> Self {
        SPIMock {
            expected: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Set expectations on the SPI interface
    /// This is a list of SPI transactions to be executed in order
    /// Note that setting this will overwrite any existing expectations
    pub fn expect(&mut self, expected: Vec<Transaction>) {
        let mut e = self.expected.lock().unwrap();
        *e = expected.into();
    }

    /// Assert that all expectations on a given SPIMock have been met
    pub fn done(&mut self) {
        let expected = self.expected.lock().unwrap();
        assert_eq!(expected.len(), 0);
    }
}

impl spi::Write<u8> for SPIMock {
    type Error = MockError;

    /// spi::Write implementation for SPIMock
    /// This will cause an assertion if the write call does not match the next expectation
    fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        let w = self
            .expected
            .lock()
            .unwrap()
            .pop_front()
            .expect("no expectation for spi::write call");
        assert_eq!(w.expected_mode, Mode::Write);
        assert_eq!(&w.expected_data, &buffer);
        Ok(())
    }
}

impl spi::Transfer<u8> for SPIMock {
    type Error = MockError;

    /// spi::Transfer implementation for SPIMock
    /// This writes the provided response to the buffer and will cause an assertion if the written data does not match the next expectation
    fn transfer<'w>(&mut self, buffer: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        let w = self
            .expected
            .lock()
            .unwrap()
            .pop_front()
            .expect("no expectation for spi::transfer call");
        assert_eq!(w.expected_mode, Mode::Transfer);
        assert_eq!(&w.expected_data, &buffer);
        for i in 0..w.response.len() {
            buffer[i] = w.response[i];
        }
        Ok(buffer)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use hal::blocking::spi::{Transfer, Write};

    #[test]
    fn test_spi_mock_write() {
        let mut spi = SPIMock::new();

        spi.expect(vec![Transaction::write(vec![10u8, 12u8])]);

        spi.write(&vec![10u8, 12u8]).unwrap();

        spi.done();
    }

    #[test]
    fn test_spi_mock_transfer() {
        let mut spi = SPIMock::new();

        spi.expect(vec![Transaction::transfer(
            vec![10u8, 12u8],
            vec![12u8, 13u8],
        )]);

        let mut v = vec![10u8, 12u8];
        spi.transfer(&mut v).unwrap();

        assert_eq!(v, vec![12u8, 13u8]);

        spi.done();
    }

    #[test]
    fn test_spi_mock_multiple() {
        let mut spi = SPIMock::new();

        spi.expect(vec![
            Transaction::write(vec![1u8, 2u8]),
            Transaction::transfer(vec![3u8, 4u8], vec![5u8, 6u8]),
        ]);

        spi.write(&vec![1u8, 2u8]).unwrap();

        let mut v = vec![3u8, 4u8];
        spi.transfer(&mut v).unwrap();

        assert_eq!(v, vec![5u8, 6u8]);

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_write_err() {
        let mut spi = SPIMock::new();

        spi.expect(vec![Transaction::write(vec![10u8, 12u8])]);

        spi.write(&vec![10u8, 12u8, 12u8]).unwrap();

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_transfer_err() {
        let mut spi = SPIMock::new();

        spi.expect(vec![Transaction::transfer(
            vec![10u8, 12u8],
            vec![12u8, 15u8],
        )]);

        let mut v = vec![10u8, 12u8];
        spi.transfer(&mut v).unwrap();

        assert_eq!(v, vec![12u8, 13u8]);

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_mode_err() {
        let mut spi = SPIMock::new();

        spi.expect(vec![Transaction::transfer(vec![10u8, 12u8], vec![])]);

        spi.write(&vec![10u8, 12u8, 12u8]).unwrap();

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_multiple_transaction_err() {
        let mut spi = SPIMock::new();

        spi.expect(vec![
            Transaction::write(vec![10u8, 12u8]),
            Transaction::write(vec![10u8, 12u8]),
        ]);

        spi.write(&vec![10u8, 12u8, 12u8]).unwrap();

        spi.done();
    }
}
