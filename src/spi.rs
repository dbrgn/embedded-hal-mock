

use std::io::{self, Read};
use std::sync::{Arc, Mutex};

use hal::blocking::spi;

use ::error::MockError;

/// SPI Transaction mode
#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    None,
    Write,
    Transfer,
}

/// SPI transaction type
#[derive(Clone, Debug, PartialEq)]
pub struct Transaction {
    expected_mode: Mode,
    expected_data: Vec<u8>,
    response: Vec<u8>,
}

impl Transaction {
    /// Create a write transaction
    pub fn write(expected: Vec<u8>) -> Transaction {
        Transaction{
            expected_mode: Mode::Write,
            expected_data: expected,
            response: Vec::new(),
        }
    }

    /// Create a transfer transaction
    pub fn transfer(expected: Vec<u8>, response: Vec<u8>) -> Transaction {
        Transaction{
            expected_mode: Mode::Transfer,
            expected_data: expected,
            response,
        }
    }
}

/// Mock SPI implementation
pub struct SPIMock {
    expected: Arc<Mutex<Vec<Transaction>>>,
}

impl SPIMock {
    /// Create a new mock SPI interface
    pub fn new() -> Self {
        SPIMock { expected: Arc::new(Mutex::new(Vec::new())) }
    }

    /// Set expectations on the SPI interface
    /// This is a list of SPI transactions to be executed
    pub fn expect(&mut self, expected: Vec<Transaction>) {
        let mut e = self.expected.lock().unwrap();
        assert_eq!(e.len(), 0);
        *e = expected;
    }

    pub fn check(&mut self) {
        let expected = self.expected.lock().unwrap();
        assert_eq!(expected.len(), 0);
    }
}

impl spi::Write<u8> for SPIMock {
    type Error = MockError;

    fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        let mut w = self.expected.lock().unwrap().pop().unwrap();
        assert_eq!(w.expected_mode, Mode::Write);
        assert_eq!(&w.expected_data, &buffer);
        Ok(())
    }
}

impl spi::Transfer<u8> for SPIMock {
    type Error = MockError;

    fn transfer<'w>(&mut self, buffer: &'w mut[u8]) -> Result<&'w [u8], Self::Error> {
        let mut w = self.expected.lock().unwrap().pop().unwrap();
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

        spi.check();
    }

    #[test]
    fn test_spi_mock_transfer() {
        let mut spi = SPIMock::new();

        spi.expect(vec![Transaction::transfer(vec![10u8, 12u8], vec![12u8, 13u8])]);

        let mut v = vec![10u8, 12u8];
        spi.transfer(&mut v).unwrap();
        
        assert_eq!(v, vec![12u8, 13u8]);

        spi.check();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_write_err() {
        let mut spi = SPIMock::new();

        spi.expect(vec![Transaction::write(vec![10u8, 12u8])]);

        spi.write(&vec![10u8, 12u8, 12u8]).unwrap();

        spi.check();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_transfer_err() {
        let mut spi = SPIMock::new();

        spi.expect(vec![Transaction::transfer(vec![10u8, 12u8], vec![12u8, 15u8])]);

        let mut v = vec![10u8, 12u8];
        spi.transfer(&mut v).unwrap();
        
        assert_eq!(v, vec![12u8, 13u8]);

        spi.check();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_mode_err() {
        let mut spi = SPIMock::new();

        spi.expect(vec![Transaction::transfer(vec![10u8, 12u8], vec![])]);

        spi.write(&vec![10u8, 12u8, 12u8]).unwrap();

        spi.check();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_transaction_err() {
        let mut spi = SPIMock::new();

        spi.expect(vec![
            Transaction::write(vec![10u8, 12u8]),
            Transaction::write(vec![10u8, 12u8]),
        ]);

        spi.write(&vec![10u8, 12u8, 12u8]).unwrap();

        spi.check();
    }
}
