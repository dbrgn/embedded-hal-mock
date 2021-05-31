//! Serial mock implementations.
//!
//! You can set expectations for serial read and write transactions on a mock
//! Serial device. Creating error transactions is supported as well.
//!
//! Note that the `embedded_hal` crate provides both non-blocking and blocking
//! serial traits. You can use the same mock for both interfaces.
//!
//! ## Usage: Non-blocking serial traits
//!
//! ```
//! // Note that we're using the non-blocking serial traits
//! use embedded_hal::serial::{Read, Write};
//! use embedded_hal_mock::serial::{
//!     Mock as SerialMock,
//!     Transaction as SerialTransaction,
//! };
//!
//! // Configure expectations
//! let expectations = [
//!     SerialTransaction::read(0x0A),
//!     SerialTransaction::read_many(b"xy"),
//!     SerialTransaction::write_many([1, 2]), // (1)
//!     SerialTransaction::flush(),
//! ];
//!
//! let mut serial = SerialMock::new(&expectations);
//!
//! // Expect three reads
//! assert_eq!(serial.read().unwrap(), 0x0A);
//! assert_eq!(serial.read().unwrap(), b'x');
//! assert_eq!(serial.read().unwrap(), b'y');
//!
//! // When designing against the non-blocking serial
//! // trait, we expect two separate writes. These could be
//! // expressed as two separate transactions, too. See (1) above.
//! serial.write(1).unwrap();
//! serial.write(2).unwrap();
//!
//! // Finally, we expect a flush
//! serial.flush().unwrap();
//!
//! // When you believe there are no more calls on the mock,
//! // call done() to assert there are no pending transactions.
//! serial.done();
//! ```
//!
//! ## Usage: Blocking serial trait
//!
//! ```
//! // Note that we're using the blocking serial write trait
//! use embedded_hal::blocking::serial::Write;
//! use embedded_hal::serial::Read;
//! use embedded_hal_mock::serial::{
//!     Mock as SerialMock,
//!     Transaction as SerialTransaction,
//! };
//!
//! // Configure expectations
//! let expectations = [
//!     SerialTransaction::read(0x0A),
//!     SerialTransaction::read_many(b"xy"),
//!     SerialTransaction::write_many([1, 2]), // (2)
//!     SerialTransaction::flush(),
//! ];
//!
//! let mut serial = SerialMock::new(&expectations);
//!
//! // Expect three reads
//! assert_eq!(serial.read().unwrap(), 0x0A);
//! assert_eq!(serial.read().unwrap(), b'x');
//! assert_eq!(serial.read().unwrap(), b'y');
//!
//! // We use the blocking write here, and we assert that
//! // two words are written. See (2) above.
//! serial.bwrite_all(&[1, 2]).unwrap();
//!
//! // Finally, we expect a flush. Note that this is
//! // a *blocking* flush from the blocking serial trait.
//! serial.bflush().unwrap();
//!
//! // When you believe there are no more calls on the mock,
//! // call done() to assert there are no pending transactions.
//! serial.done();
//! ```
//!
//! ## Testing Error Handling
//!
//! If you want to test error handling of your code, you can also add error
//! transactions. When the transaction is executed, an error is returned.
//!
//! ```
//! # use embedded_hal::prelude::*;
//! # use embedded_hal_mock::serial::{
//! #     Mock as SerialMock,
//! #     Transaction as SerialTransaction,
//! # };
//! use std::io::ErrorKind;
//! use embedded_hal_mock::MockError;
//!
//! // Configure expectations
//! let expectations = [
//!     SerialTransaction::read(42),
//!     SerialTransaction::read_error(nb::Error::WouldBlock),
//!     SerialTransaction::write_error(23, nb::Error::Other(MockError::Io(ErrorKind::Other))),
//!     SerialTransaction::flush_error(nb::Error::Other(MockError::Io(ErrorKind::Interrupted))),
//! ];
//! let mut serial = SerialMock::new(&expectations);
//!
//! // The first read will succeed
//! assert_eq!(serial.read().unwrap(), 42);
//!
//! // The second read will return an error
//! assert_eq!(serial.read().unwrap_err(), nb::Error::WouldBlock);
//!
//! // The following write/flush calls will return errors as well
//! assert_eq!(
//!     serial.write(23).unwrap_err(),
//!     nb::Error::Other(MockError::Io(ErrorKind::Other))
//! );
//! assert_eq!(
//!     serial.flush().unwrap_err(),
//!     nb::Error::Other(MockError::Io(ErrorKind::Interrupted))
//! );
//!
//! // When you believe there are no more calls on the mock,
//! // call done() to assert there are no pending transactions.
//! serial.done();
//! ```

// This module is implemented a little differently than
// the spi and i2c modules. We'll note that, unlike the
// spi and i2c modules which share the foundational Generic
// transaction queue, we provide our own implementation.
// We found that, in keeping with the established API design
// and the unique features of the embedded_hal serial traits
// (described in the note below), this was a necessary trade-
// off. We welcome any other ideas that allow us to take
// advantage of the common components.
//
// We also generalize over a trait's `Word`, rather than requiring
// consumers to use traits that operate on `u8`s. This does not
// make the public API any more confusing for users, and it permits
// maximal flexibility.

use embedded_hal::blocking::serial::write;
use embedded_hal::serial;

use crate::error::MockError;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

// Note that mode is private
//
// Although it is public in both the spi
// and i2c modules, the variants are not
// required to be in the public interface.
// We chose to not supply them publicly to
// consumers because there is no public API
// that readily uses them.

/// Serial communication mode
#[derive(Debug, Clone)]
enum Mode<Word> {
    /// A serial read that returns a word
    Read(Word),
    /// A serial read that returns an error
    ReadError(nb::Error<MockError>),
    /// A serial write that transmits a word
    Write(Word),
    /// A serial write that returns an error
    WriteError(Word, nb::Error<MockError>),
    /// A flush call
    Flush,
    /// A flush call that returns an error
    FlushError(nb::Error<MockError>),
}

/// A serial transaction
///
/// Transactions can either be reads, writes, or flushes. A
/// collection of transactions represent the expected operations
/// that are performed on your serial device.
///
/// # Example
///
/// ```
/// use embedded_hal_mock::serial::Transaction;
/// use embedded_hal_mock::serial::Mock;
///
/// // We expect, in order,
/// // 1. A read that returns 0x23,
/// // 2. A write of [0x55, 0xAA]
/// // 3. A flush
/// let transactions = [
///     Transaction::read(0x23),
///     Transaction::write_many([0x55, 0xAA]),
///     Transaction::flush()
/// ];
///
/// let serial = Mock::new(&transactions);
/// ```
pub struct Transaction<Word> {
    /// A collection of modes
    ///
    /// Since we need to express a blocking write in terms of
    /// multiple writes, we aggregate all of them into this
    /// member. Then, they are handed-off to the mock on
    /// construction.
    mode: Vec<Mode<Word>>,
}

impl<Word> Transaction<Word>
where
    Word: Clone,
{
    /// Expect a serial read that returns the expected word
    pub fn read(word: Word) -> Self {
        Transaction {
            mode: vec![Mode::Read(word)],
        }
    }

    /// Expect a serial read that returns the expected words
    pub fn read_many<Ws>(words: Ws) -> Self
    where
        Ws: AsRef<[Word]>,
    {
        Transaction {
            mode: words.as_ref().iter().cloned().map(Mode::Read).collect(),
        }
    }

    /// Expect a serial read that returns an error
    pub fn read_error(error: nb::Error<MockError>) -> Self {
        Transaction {
            mode: vec![Mode::ReadError(error)],
        }
    }

    /// Expect a serial write that transmits the specified word
    pub fn write(word: Word) -> Self {
        Transaction {
            mode: vec![Mode::Write(word)],
        }
    }

    /// Expect a serial write that transmits the specified words
    pub fn write_many<Ws>(words: Ws) -> Self
    where
        Ws: AsRef<[Word]>,
    {
        Transaction {
            mode: words.as_ref().iter().cloned().map(Mode::Write).collect(),
        }
    }

    /// Expect a serial write that returns an error after transmitting the
    /// specified word
    pub fn write_error(word: Word, error: nb::Error<MockError>) -> Self {
        Transaction {
            mode: vec![Mode::WriteError(word, error)],
        }
    }

    /// Expect a caller to flush the serial buffers
    pub fn flush() -> Self {
        Transaction {
            mode: vec![Mode::Flush],
        }
    }

    /// Expect a serial flush that returns an error
    pub fn flush_error(error: nb::Error<MockError>) -> Self {
        Transaction {
            mode: vec![Mode::FlushError(error)],
        }
    }
}

/// Mock serial device
///
/// The mock serial device can be loaded with expected transactions, then
/// passed-on into a serial device user. If the expectations were not met
/// in the specified order, the type causes a panic and describes what
/// expectation wasn't met.
///
/// The type is clonable so that it may be shared with a serial
/// device user. Under the hood, both cloned mocks will share
/// the same state, allowing your handle to eventually call `done()`,
/// if desired.
#[derive(Clone)]
pub struct Mock<Word> {
    /// The expected operations upon the mock
    ///
    /// It's in an arc to maintain shared state, and in a mutex
    /// to make it thread safe. It's then wrapped in an `Option`
    /// so that we can take it in the call to `done()`.
    expected_modes: Arc<Mutex<Option<VecDeque<Mode<Word>>>>>,
}

impl<Word: Clone> Mock<Word> {
    /// Create a serial mock that will expect the provided transactions
    pub fn new(transactions: &[Transaction<Word>]) -> Self {
        let mut ser = Mock {
            expected_modes: Arc::new(Mutex::new(None)),
        };
        ser.expect(transactions);
        ser
    }

    /// Set expectations on the interface
    ///
    /// This is a list of transactions to be executed in order.
    /// Note that setting this will overwrite any existing expectations
    pub fn expect(&mut self, transactions: &[Transaction<Word>]) {
        let mut lock = self
            .expected_modes
            .lock()
            .expect("unable to lock serial mock in call to expect");
        *lock = Some(
            transactions
                .iter()
                .fold(VecDeque::new(), |mut modes, transaction| {
                    modes.extend(transaction.mode.clone());
                    modes
                }),
        );
    }

    /// Asserts that all expectations up to this point were satisfied.
    /// Panics if there are unsatisfied expectations.
    pub fn done(&mut self) {
        let mut lock = self
            .expected_modes
            .lock()
            .expect("unable to lock serial mock in call to done");
        let modes = lock.take().expect("attempted to take None from Optional");
        assert!(
            modes.is_empty(),
            "serial mock has unsatisfied expectations after call to done"
        );
    }

    /// Pop the next transaction out of the queue
    fn pop(&mut self) -> Option<Mode<Word>> {
        let mut lock = self
            .expected_modes
            .lock()
            .expect("unable to lock serial mock in call to pop");
        let queue = lock
            .as_mut()
            .expect("attempt to get queue reference from a None");
        queue.pop_front()
    }
}

impl<Word> serial::Read<Word> for Mock<Word>
where
    Word: Clone + std::fmt::Debug,
{
    type Error = MockError;

    fn read(&mut self) -> nb::Result<Word, Self::Error> {
        let t = self.pop().expect("called serial::read with no expectation");
        match t {
            Mode::Read(word) => Ok(word),
            Mode::ReadError(error) => Err(error),
            other => panic!(
                "expected to perform a serial transaction '{:?}', but instead did a read",
                other
            ),
        }
    }
}

impl<Word> serial::Write<Word> for Mock<Word>
where
    Word: PartialEq + std::fmt::Debug + Clone,
{
    type Error = MockError;

    fn write(&mut self, word: Word) -> nb::Result<(), Self::Error> {
        let t = self
            .pop()
            .expect("called serial::write with no expectation");

        let assert_write = |expectation: Word| {
            assert_eq!(
                expectation, word,
                "serial::write expected to write {:?} but actually wrote {:?}",
                expectation, word
            );
        };

        match t {
            Mode::Write(expectation) => {
                assert_write(expectation);
                Ok(())
            }
            Mode::WriteError(expectation, error) => {
                assert_write(expectation);
                Err(error)
            }
            other => panic!(
                "expected to perform a serial transaction '{:?}' but instead did a write of {:?}",
                other, word
            ),
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        let t = self
            .pop()
            .expect("called serial::flush with no expectation");
        match t {
            Mode::Flush => Ok(()),
            Mode::FlushError(error) => Err(error),
            mode => panic!(
                "expected to perform a serial transaction '{:?}' but instead did a flush",
                mode
            ),
        }
    }
}

// Note: We attempted to provide our own implementation of
// embedded_hal::blocking::serial::Write. However, we're unable
// to override it due to the blanket default implementation provided by
// the embedded_hal crate. It comes down to the fact that, if we were
// to provide an embedded_hal::blocking::serial::Write implementation
// here, any user of embedded_hal would be free to implement the *default*
// version for our type. Therefore, we conform to the default implementation,
// knowing that the default is implemented in terms of the non-blocking
// trait, which is defined above.
//
// If you know a way around this, please let us know!
impl<Word> write::Default<Word> for Mock<Word> where Word: PartialEq + std::fmt::Debug + Clone {}

#[cfg(test)]
mod test {
    use std::io;

    use embedded_hal::{
        blocking::serial::Write as BWrite,
        serial::{Read, Write},
    };

    use super::{Mock, MockError, Transaction};

    #[test]
    fn test_serial_mock_read() {
        let ts = [Transaction::read(0x54)];
        let mut ser = Mock::new(&ts);
        let r = ser.read().expect("failed to read");
        assert_eq!(r, 0x54);
        ser.done();
    }

    #[test]
    fn test_serial_mock_write_single_value_nonblocking() {
        let ts = [Transaction::write(0xAB)];
        let mut ser = Mock::new(&ts);
        ser.write(0xAB).unwrap();
        ser.done();
    }

    #[test]
    fn test_serial_mock_write_many_values_nonblocking() {
        let ts = [Transaction::write_many([0xAB, 0xCD, 0xEF])];
        let mut ser = Mock::new(&ts);
        ser.write(0xAB).unwrap();
        ser.write(0xCD).unwrap();
        ser.write(0xEF).unwrap();
        ser.done();
    }

    #[test]
    fn test_serial_mock_read_many_values_nonblocking() {
        let ts = [Transaction::read_many([0xAB, 0xCD, 0xEF])];
        let mut ser = Mock::new(&ts);
        assert_eq!(0xAB, ser.read().unwrap());
        assert_eq!(0xCD, ser.read().unwrap());
        assert_eq!(0xEF, ser.read().unwrap());
        ser.done();
    }

    #[test]
    fn test_serial_mock_blocking_write() {
        let ts = [Transaction::write_many([0xAB, 0xCD, 0xEF])];
        let mut ser = Mock::new(&ts);
        ser.bwrite_all(&[0xAB, 0xCD, 0xEF]).unwrap();
        ser.done();
    }

    #[test]
    #[should_panic(expected = "called serial::write with no expectation")]
    fn test_serial_mock_blocking_write_more_than_expected() {
        let ts = [Transaction::write_many([0xAB, 0xCD])];
        let mut ser = Mock::new(&ts);
        ser.bwrite_all(&[0xAB, 0xCD, 0xEF]).unwrap();
        ser.done();
    }

    #[test]
    #[should_panic(expected = "unsatisfied expectations")]
    fn test_serial_mock_blocking_write_not_enough() {
        let ts = [Transaction::write_many([0xAB, 0xCD, 0xEF, 0x00])];
        let mut ser = Mock::new(&ts);
        ser.bwrite_all(&[0xAB, 0xCD, 0xEF]).unwrap();
        ser.done();
    }

    #[test]
    #[should_panic(expected = "serial::write expected to write")]
    fn test_serial_mock_wrong_write() {
        let ts = [Transaction::write(0x12)];
        let mut ser = Mock::new(&ts);
        ser.write(0x14).unwrap();
    }

    #[test]
    fn test_serial_mock_flush() {
        let ts = [Transaction::flush()];
        let mut ser: Mock<u8> = Mock::new(&ts);
        ser.flush().unwrap();
        ser.done();
    }

    #[test]
    fn test_serial_mock_blocking_flush() {
        let ts = [Transaction::flush()];
        let mut ser: Mock<u8> = Mock::new(&ts);
        ser.bflush().unwrap();
        ser.done();
    }

    #[test]
    #[should_panic(expected = "unsatisfied expectations")]
    fn test_serial_mock_pending_transactions() {
        let ts = [Transaction::read(0x54)];
        let mut ser = Mock::new(&ts);
        ser.done();
    }

    #[test]
    #[should_panic(expected = "unsatisfied expectations")]
    fn test_serial_mock_reuse_pending_transactions() {
        let ts = [Transaction::read(0x54)];
        let mut ser = Mock::new(&ts);
        let r = ser.read().expect("failed to read");
        assert_eq!(r, 0x54);
        ser.done();
        ser.expect(&ts);
        ser.done();
    }

    #[test]
    #[should_panic(expected = "expected to perform a serial transaction 'Read(")]
    fn test_serial_mock_expected_read() {
        let ts = [Transaction::read(0x54)];
        let mut ser = Mock::new(&ts);
        ser.bwrite_all(&[0x77]).unwrap();
    }

    #[test]
    #[should_panic(expected = "expected to perform a serial transaction 'Write(")]
    fn test_serial_mock_expected_write() {
        let ts = [Transaction::write(0x54)];
        let mut ser = Mock::new(&ts);
        ser.flush().unwrap();
    }

    #[test]
    #[should_panic(expected = "expected to perform a serial transaction 'Flush'")]
    fn test_serial_mock_expected_flush() {
        let ts = [Transaction::flush()];
        let mut ser: Mock<u128> = Mock::new(&ts);
        ser.read().unwrap();
    }

    #[test]
    fn test_serial_mock_read_error() {
        let error = nb::Error::WouldBlock;
        let ts = [Transaction::read_error(error.clone())];
        let mut ser: Mock<u8> = Mock::new(&ts);
        assert_eq!(ser.read().unwrap_err(), error);
    }

    #[test]
    fn test_serial_mock_write_error() {
        let error = nb::Error::Other(MockError::Io(io::ErrorKind::NotConnected));
        let ts = [Transaction::write_error(42, error.clone())];
        let mut ser: Mock<u8> = Mock::new(&ts);
        assert_eq!(ser.write(42).unwrap_err(), error);
    }

    #[test]
    #[should_panic(expected = "serial::write expected to write 42 but actually wrote 23")]
    fn test_serial_mock_write_error_wrong_data() {
        let error = nb::Error::Other(MockError::Io(io::ErrorKind::NotConnected));
        let ts = [Transaction::write_error(42, error.clone())];
        let mut ser: Mock<u8> = Mock::new(&ts);
        // The data to be written should still be verified, even if there's an
        // error attached.
        ser.write(23).unwrap();
    }

    #[test]
    fn test_serial_mock_flush_error() {
        let error = nb::Error::Other(MockError::Io(io::ErrorKind::TimedOut));
        let ts = [Transaction::flush_error(error.clone())];
        let mut ser: Mock<u8> = Mock::new(&ts);
        assert_eq!(ser.flush().unwrap_err(), error);
    }
}
