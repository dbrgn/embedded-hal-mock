//! I²C mock implementations.
//!
//! ## Usage
//!
//! ```
//! extern crate embedded_hal;
//! extern crate embedded_hal_mock;
//!
//! use embedded_hal::prelude::*;
//! use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
//! use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
//!
//! let mut i2c = I2cMock::new();
//!
//! // Configure expectations
//! i2c.expect(vec![
//!     I2cTransaction::write(0xaa, vec![1, 2]),
//!     I2cTransaction::read(0xbb, vec![3, 4]),
//! ]);
//!
//! // Writing
//! i2c.write(0xaa, &vec![1, 2]).unwrap();
//!
//! // Reading
//! let mut buf = vec![0u8; 2];
//! i2c.read(0xbb, &mut buf).unwrap();
//! assert_eq!(buf, vec![3, 4]);
//!
//! // Finalise expectations
//! i2c.done();
//! ```

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use hal::blocking::i2c;

use error::MockError;

/// I2C Transaction modes
#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Write,
    Read,
    WriteRead,
}

/// I2C Transaction type
/// 
/// Models an I2C read or write
#[derive(Clone, Debug, PartialEq)]
pub struct Transaction {
    expected_mode: Mode,
    expected_addr: u8,
    expected_data: Vec<u8>,
    response_data: Vec<u8>,
}

impl Transaction {
    /// Create a Write transaction
    pub fn write(addr: u8, expected: Vec<u8>) -> Transaction {
        Transaction {
            expected_mode: Mode::Write,
            expected_addr: addr,
            expected_data: expected,
            response_data: Vec::new(),
        }
    }

    /// Create a Read transaction
    pub fn read(addr: u8, response: Vec<u8>) -> Transaction {
        Transaction {
            expected_mode: Mode::Read,
            expected_addr: addr,
            expected_data: Vec::new(),
            response_data: response,
        }
    }

    /// Create a WriteRead transaction
    pub fn write_read(addr: u8, expected: Vec<u8>, response: Vec<u8>) -> Transaction {
        Transaction {
            expected_mode: Mode::WriteRead,
            expected_addr: addr,
            expected_data: expected,
            response_data: response,
        }
    }
}

/// Mock I2C implementation
/// 
/// This supports the specification and evaluation of expectations to allow automated testing of I2C based drivers.
/// Mismatches between expectations will cause runtime assertions to assist in locating the source of the fault.
pub struct Mock {
    expected: Arc<Mutex<VecDeque<Transaction>>>,
}

impl Mock {
    pub fn new() -> Self {
        Mock {
            expected: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Set expectations on the I2C interface
    /// 
    /// This is a list of I2C transactions to be executed in order
    /// Note that setting this will overwrite any existing expectations
    pub fn expect(&mut self, expected: Vec<Transaction>) {
        let mut e = self.expected.lock().unwrap();
        *e = expected.into();
    }

    /// Assert that all expectations on a given Mock have been met
    pub fn done(&mut self) {
        let expected = self.expected.lock().unwrap();
        assert_eq!(expected.len(), 0);
    }
}

impl Clone for Mock {
    fn clone(&self) -> Mock {
        Mock{ expected: self.expected.clone() }
    }
}

impl i2c::Read for Mock {
    type Error = MockError;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let w = self
            .expected
            .lock()
            .unwrap()
            .pop_front()
            .expect("no pending expectation for i2c::read call");

        assert_eq!(w.expected_mode, Mode::Read, "i2c::read unexpected mode");
        assert_eq!(w.expected_addr, address, "i2c::read address mismatch");

        assert_eq!(buffer.len(), w.response_data.len(), "i2c:read mismatched response length");
        buffer.copy_from_slice(&w.response_data);

        Ok(())
    }
}

impl i2c::Write for Mock {
    type Error = MockError;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        let w = self
            .expected
            .lock()
            .unwrap()
            .pop_front()
            .expect("no pending expectation for i2c::read call");
            
        assert_eq!(w.expected_mode, Mode::Write, "i2c::write unexpected mode");
        assert_eq!(w.expected_addr, address, "i2c::write address mismatch");
        assert_eq!(w.expected_data, bytes, "i2c::write data does not match expectation");

        Ok(())
    }
}

impl i2c::WriteRead for Mock {
    type Error = MockError;

    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        let w = self
            .expected
            .lock()
            .unwrap()
            .pop_front()
            .expect("no pending expectation for i2c::read call");
            
        assert_eq!(w.expected_mode, Mode::WriteRead, "i2c::write_read unexpected mode");
        assert_eq!(w.expected_addr, address, "i2c::write_read address mismatch");
        assert_eq!(w.expected_data, bytes, "i2c::write_read write data does not match expectation");

        assert_eq!(buffer.len(), w.response_data.len(), "i2c::write_read mismatched response length");
        buffer.copy_from_slice(&w.response_data);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use hal::blocking::i2c::{Read, Write, WriteRead};

    #[test]
    fn test_i2c_mock_write() {
        let mut i2c = Mock::new();

        i2c.expect(vec![Transaction::write(0xaa, vec![10, 12])]);

        i2c.write(0xaa, &vec![10, 12]).unwrap();

        i2c.done();
    }

    #[test]
    fn test_i2c_mock_read() {
        let mut i2c = Mock::new();

        i2c.expect(vec![Transaction::read(0xaa, vec![1, 2])]);

        let mut buff = vec![0u8; 2];
        i2c.read(0xaa, &mut buff).unwrap();
        assert_eq!(vec![1, 2], buff);

        i2c.done();
    }

    #[test]
    fn test_i2c_mock_write_read() {
        let mut i2c = Mock::new();

        i2c.expect(vec![Transaction::write_read(0xaa,
            vec![1, 2],
            vec![3, 4],
        )]);

        let v = vec![1, 2];
        let mut buff = vec![0u8; 2];
        i2c.write_read(0xaa, &v, &mut buff).unwrap();
        assert_eq!(vec![3, 4], buff);

        i2c.done();
    }

    #[test]
    fn test_i2c_mock_multiple() {
        let mut i2c = Mock::new();

        i2c.expect(vec![
            Transaction::write(0xaa, vec![1, 2]),
            Transaction::read(0xbb, vec![3, 4]),
        ]);

        i2c.write(0xaa, &vec![1, 2]).unwrap();

        let mut v = vec![0u8; 2];
        i2c.read(0xbb, &mut v).unwrap();

        assert_eq!(v, vec![3, 4]);

        i2c.done();
    }

    #[test]
    #[should_panic]
    fn test_i2c_mock_write_err() {
        let mut i2c = Mock::new();

        i2c.expect(vec![Transaction::write(0xaa, vec![1, 2])]);

        i2c.write(0xaa, &vec![1, 3]).unwrap();

        i2c.done();
    }

    #[test]
    #[should_panic]
    fn test_i2c_mock_read_err() {
        let mut i2c = Mock::new();

        i2c.expect(vec![Transaction::read(0xaa, vec![10, 12])]);

        let mut buff = vec![0u8; 2];
        i2c.write(0xaa, &mut buff).unwrap();
        assert_eq!(vec![10, 12], buff);

        i2c.done();
    }

    #[test]
    #[should_panic]
    fn test_i2c_mock_write_read_err() {
        let mut i2c = Mock::new();

        i2c.expect(vec![Transaction::write_read(0xbb,
            vec![1, 2],
            vec![3, 4],
        )]);

        let v = vec![1, 2];
        let mut buff = vec![0u8; 2];
        i2c.write_read(0xaa, &v, &mut buff).unwrap();
        assert_eq!(vec![3, 4], buff);

        i2c.done();
    }

    #[test]
    #[should_panic]
    fn test_i2c_mock_mode_err() {
        let mut i2c = Mock::new();

        i2c.expect(vec![Transaction::read(0xaa, vec![10, 12])]);

        i2c.write(0xaa, &vec![10, 12]).unwrap();

        i2c.done();
    }

    #[test]
    #[should_panic]
    fn test_i2c_mock_multiple_transaction_err() {
        let mut i2c = Mock::new();

        i2c.expect(vec![
            Transaction::write(0xaa, vec![10, 12]),
            Transaction::write(0xaa, vec![10, 12]),
        ]);

        i2c.write(0xaa, &vec![10, 12]).unwrap();

        i2c.done();
    }
}
