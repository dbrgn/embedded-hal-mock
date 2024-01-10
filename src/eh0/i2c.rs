//! I²C mock implementations.
//!
//! ## Usage
//!
//! ```
//! # use eh0 as embedded_hal;
//! use embedded_hal::{
//!     blocking::i2c::{Read, Write, WriteRead},
//!     prelude::*,
//! };
//! use embedded_hal_mock::eh0::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
//!
//! // Configure expectations
//! let expectations = [
//!     I2cTransaction::write(0xaa, vec![1, 2]),
//!     I2cTransaction::read(0xbb, vec![3, 4]),
//! ];
//! let mut i2c = I2cMock::new(&expectations);
//!
//! // Writing
//! i2c.write(0xaa, &vec![1, 2]).unwrap();
//!
//! // Reading
//! let mut buf = vec![0; 2];
//! i2c.read(0xbb, &mut buf).unwrap();
//! assert_eq!(buf, vec![3, 4]);
//!
//! // Finalise expectations
//! i2c.done();
//! ```
//!
//! ## Transactions
//!
//! There are currently three transaction types:
//!
//! - `Read`: This expects an I²C `read` command and will return the wrapped bytes.
//! - `Write`: This expects an I²C `write` command with the wrapped bytes.
//! - `WriteRead`: This expects an I²C `write_read` command where the
//!   `expected` bytes are written and the `response` bytes are returned.
//!
//! ## Testing Error Handling
//!
//! If you want to test error handling of your code, you can attach an error to
//! a transaction. When the transaction is executed, an error is returned.
//!
//! ```
//! # use eh0 as embedded_hal;
//! # use embedded_hal::prelude::*;
//! # use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
//! # use embedded_hal_mock::eh0::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
//! use std::io::ErrorKind;
//!
//! use embedded_hal_mock::eh0::MockError;
//!
//! // Configure expectations
//! let expectations = [
//!     I2cTransaction::write(0xaa, vec![1, 2]),
//!     I2cTransaction::read(0xbb, vec![3, 4]).with_error(MockError::Io(ErrorKind::Other)),
//! ];
//! let mut i2c = I2cMock::new(&expectations);
//!
//! // Writing returns without an error
//! i2c.write(0xaa, &vec![1, 2]).unwrap();
//!
//! // Reading returns an error
//! let mut buf = vec![0; 2];
//! let err = i2c.read(0xbb, &mut buf).unwrap_err();
//! assert_eq!(err, MockError::Io(ErrorKind::Other));
//!
//! // Finalise expectations
//! i2c.done();
//! ```

use eh0 as embedded_hal;
use embedded_hal::blocking::i2c;

use super::error::MockError;
use crate::common::Generic;

/// I2C Transaction modes
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Write transaction
    Write,
    /// Read transaction
    Read,
    /// Write and read transaction
    WriteRead,
}

/// I2C Transaction type
///
/// Models an I2C read or write
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    expected_mode: Mode,
    expected_addr: u8,
    expected_data: Vec<u8>,
    response_data: Vec<u8>,
    /// An optional error return for a transaction.
    ///
    /// This is in addition to the mode to allow validation that the
    /// transaction mode is correct prior to returning the error.
    expected_err: Option<MockError>,
}

impl Transaction {
    /// Create a Write transaction
    pub fn write(addr: u8, expected: Vec<u8>) -> Transaction {
        Transaction {
            expected_mode: Mode::Write,
            expected_addr: addr,
            expected_data: expected,
            response_data: Vec::new(),
            expected_err: None,
        }
    }

    /// Create a Read transaction
    pub fn read(addr: u8, response: Vec<u8>) -> Transaction {
        Transaction {
            expected_mode: Mode::Read,
            expected_addr: addr,
            expected_data: Vec::new(),
            response_data: response,
            expected_err: None,
        }
    }

    /// Create a WriteRead transaction
    pub fn write_read(addr: u8, expected: Vec<u8>, response: Vec<u8>) -> Transaction {
        Transaction {
            expected_mode: Mode::WriteRead,
            expected_addr: addr,
            expected_data: expected,
            response_data: response,
            expected_err: None,
        }
    }

    /// Add an error return to a transaction
    ///
    /// This is used to mock failure behaviours.
    ///
    /// Note: When attaching this to a read transaction, the response in the
    /// expectation will not actually be written to the buffer.
    pub fn with_error(mut self, error: MockError) -> Self {
        self.expected_err = Some(error);
        self
    }
}

/// Mock I2C implementation
///
/// This supports the specification and evaluation of expectations to allow automated testing of I2C based drivers.
/// Mismatches between expectations will cause runtime assertions to assist in locating the source of the fault.
pub type Mock = Generic<Transaction>;

impl i2c::Read for Mock {
    type Error = MockError;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let e = self
            .next()
            .expect("no pending expectation for i2c::read call");

        assert_eq!(e.expected_mode, Mode::Read, "i2c::read unexpected mode");
        assert_eq!(e.expected_addr, address, "i2c::read address mismatch");

        assert_eq!(
            buffer.len(),
            e.response_data.len(),
            "i2c:read mismatched response length"
        );

        match e.expected_err {
            Some(err) => Err(err),
            None => {
                buffer.copy_from_slice(&e.response_data);
                Ok(())
            }
        }
    }
}

impl i2c::Write for Mock {
    type Error = MockError;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        let e = self
            .next()
            .expect("no pending expectation for i2c::write call");

        assert_eq!(e.expected_mode, Mode::Write, "i2c::write unexpected mode");
        assert_eq!(e.expected_addr, address, "i2c::write address mismatch");
        assert_eq!(
            e.expected_data, bytes,
            "i2c::write data does not match expectation"
        );

        match e.expected_err {
            Some(err) => Err(err),
            None => Ok(()),
        }
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
        let e = self
            .next()
            .expect("no pending expectation for i2c::write_read call");

        assert_eq!(
            e.expected_mode,
            Mode::WriteRead,
            "i2c::write_read unexpected mode"
        );
        assert_eq!(e.expected_addr, address, "i2c::write_read address mismatch");
        assert_eq!(
            e.expected_data, bytes,
            "i2c::write_read write data does not match expectation"
        );

        assert_eq!(
            buffer.len(),
            e.response_data.len(),
            "i2c::write_read mismatched response length"
        );

        match e.expected_err {
            Some(err) => Err(err),
            None => {
                buffer.copy_from_slice(&e.response_data);
                Ok(())
            }
        }
    }
}

impl i2c::WriteIterRead for Mock {
    type Error = MockError;

    fn write_iter_read<B>(
        &mut self,
        address: u8,
        bytes: B,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error>
    where
        B: IntoIterator<Item = u8>,
    {
        // Just collect the bytes and pass them on to the WriteRead::write_read implementation
        use embedded_hal::blocking::i2c::WriteRead;
        let bytes: Vec<_> = bytes.into_iter().collect();
        self.write_read(address, bytes.as_slice(), buffer)
    }
}

impl i2c::WriteIter for Mock {
    type Error = MockError;

    fn write<B>(&mut self, address: u8, bytes: B) -> Result<(), Self::Error>
    where
        B: IntoIterator<Item = u8>,
    {
        // Just collect the bytes and pass them on to the Write::write implementation
        use embedded_hal::blocking::i2c::Write;
        let bytes: Vec<_> = bytes.into_iter().collect();
        Write::write(self, address, bytes.as_slice())
    }
}

#[cfg(test)]
mod test {
    use std::{io::ErrorKind as IoErrorKind, time::SystemTime};

    use eh0 as embedded_hal;
    use embedded_hal::blocking::i2c::{Read, Write, WriteRead};

    use super::{super::error::MockError, *};

    #[test]
    fn write() {
        let expectations = [Transaction::write(0xaa, vec![10, 12])];
        let mut i2c = Mock::new(&expectations);

        i2c.write(0xaa, &vec![10, 12]).unwrap();

        i2c.done();
    }

    #[test]
    fn read() {
        let expectations = [Transaction::read(0xaa, vec![1, 2])];
        let mut i2c = Mock::new(&expectations);

        let mut buff = vec![0; 2];
        i2c.read(0xaa, &mut buff).unwrap();
        assert_eq!(vec![1, 2], buff);

        i2c.done();
    }

    #[test]
    fn write_read() {
        let expectations = [Transaction::write_read(0xaa, vec![1, 2], vec![3, 4])];
        let mut i2c = Mock::new(&expectations);

        let v = vec![1, 2];
        let mut buff = vec![0; 2];
        i2c.write_read(0xaa, &v, &mut buff).unwrap();
        assert_eq!(vec![3, 4], buff);

        i2c.done();
    }

    #[test]
    fn multiple_transactions() {
        let expectations = [
            Transaction::write(0xaa, vec![1, 2]),
            Transaction::read(0xbb, vec![3, 4]),
        ];
        let mut i2c = Mock::new(&expectations);

        i2c.write(0xaa, &vec![1, 2]).unwrap();

        let mut v = vec![0; 2];
        i2c.read(0xbb, &mut v).unwrap();
        assert_eq!(v, vec![3, 4]);

        i2c.done();
    }

    #[test]
    #[should_panic(expected = "i2c::write data does not match expectation")]
    fn write_data_mismatch() {
        let expectations = [Transaction::write(0xaa, vec![1, 2])];
        let mut i2c = Mock::new(&expectations);

        let _ = i2c.write(0xaa, &vec![1, 3]);
    }

    #[test]
    #[should_panic(expected = "i2c::write unexpected mode")]
    fn transaction_type_mismatch() {
        let expectations = [Transaction::read(0xaa, vec![10, 12])];
        let mut i2c = Mock::new(&expectations);

        let mut buff = vec![0; 2];
        let _ = i2c.write(0xaa, &mut buff);
    }

    #[test]
    #[should_panic(expected = "i2c::write_read address mismatch")]
    fn address_mismatch() {
        let expectations = [Transaction::write_read(0xbb, vec![1, 2], vec![3, 4])];
        let mut i2c = Mock::new(&expectations);

        let v = vec![1, 2];
        let mut buff = vec![0; 2];
        let _ = i2c.write_read(0xaa, &v, &mut buff);
    }

    #[test]
    #[should_panic(expected = "Not all expectations consumed")]
    fn unconsumed_expectations() {
        let expectations = [
            Transaction::write(0xaa, vec![10, 12]),
            Transaction::write(0xaa, vec![10, 12]),
        ];
        let mut i2c = Mock::new(&expectations);

        i2c.write(0xaa, &vec![10, 12]).unwrap();

        i2c.done();
    }

    #[test]
    fn clone_linked_to_original() {
        let expectations = [
            Transaction::read(0xaa, vec![1, 2]),
            Transaction::write(0xbb, vec![3, 4]),
        ];
        let mut i2c = Mock::new(&expectations);

        // Clone mock. The clone should be linked to the same data as the original.
        let mut i2c_clone = i2c.clone();

        // Read on the original mock
        let mut buff = vec![0; 2];
        i2c.read(0xaa, &mut buff).unwrap();
        assert_eq!(vec![1, 2], buff);

        // Write on the clone
        i2c_clone.write(0xbb, &[3, 4]).unwrap();

        // Randomly call `.done()` on the original mock, or on the clone.
        // Use "system time % 2" as poor man's `rand()`.
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        if now.as_millis() % 2 == 0 {
            i2c.done();
        } else {
            i2c_clone.done();
        }
    }

    mod with_error {
        use super::*;

        #[test]
        fn write() {
            let expected_err = MockError::Io(IoErrorKind::Other);
            let mut i2c = Mock::new(&[
                Transaction::write(0xaa, vec![10, 12]).with_error(expected_err.clone())
            ]);
            let err = i2c.write(0xaa, &vec![10, 12]).unwrap_err();
            assert_eq!(err, expected_err);
            i2c.done();
        }

        /// The transaction mode should still be validated.
        #[test]
        #[should_panic(expected = "i2c::read unexpected mode")]
        fn write_wrong_mode() {
            let mut i2c = Mock::new(&[Transaction::write(0xaa, vec![10, 12])
                .with_error(MockError::Io(IoErrorKind::Other))]);
            let mut buf = vec![0; 2];
            let _ = i2c.read(0xaa, &mut buf);
        }

        /// The transaction bytes should still be validated.
        #[test]
        #[should_panic(expected = "i2c::write data does not match expectation")]
        fn write_wrong_data() {
            let mut i2c = Mock::new(&[Transaction::write(0xaa, vec![10, 12])
                .with_error(MockError::Io(IoErrorKind::Other))]);
            let _ = i2c.write(0xaa, &vec![10, 13]);
        }

        #[test]
        fn read() {
            let expected_err = MockError::Io(IoErrorKind::Other);
            let mut i2c =
                Mock::new(
                    &[Transaction::read(0xaa, vec![10, 12]).with_error(expected_err.clone())],
                );
            let mut buf = vec![0; 2];
            let err = i2c.read(0xaa, &mut buf).unwrap_err();
            assert_eq!(err, expected_err);
            i2c.done();
        }

        #[test]
        fn write_read() {
            let expected_err = MockError::Io(IoErrorKind::Other);
            let mut i2c = Mock::new(&[Transaction::write_read(0xaa, vec![10, 12], vec![13, 14])
                .with_error(expected_err.clone())]);
            let mut buf = vec![0; 2];
            let err = i2c.write_read(0xaa, &[10, 12], &mut buf).unwrap_err();
            assert_eq!(err, expected_err);
            i2c.done();
        }

        /// The transaction bytes should still be validated.
        #[test]
        #[should_panic(expected = "i2c::write_read write data does not match expectation")]
        fn write_read_wrong_data() {
            let mut i2c = Mock::new(&[Transaction::write_read(0xaa, vec![10, 12], vec![13, 14])
                .with_error(MockError::Io(IoErrorKind::Other))]);
            let mut buf = vec![0; 2];
            let _ = i2c.write_read(0xaa, &vec![10, 13], &mut buf);
        }
    }
}
