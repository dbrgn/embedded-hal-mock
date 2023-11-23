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
//! # use eh1 as embedded_hal;
//! // Note that we're using the non-blocking serial traits
//! use embedded_hal_nb::serial::{Read, Write};
//! use embedded_hal_mock::eh1::serial::{
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
//! ## Testing Error Handling
//!
//! If you want to test error handling of your code, you can also add error
//! transactions. When the transaction is executed, an error is returned.
//!
//! ```
//! # use eh1 as embedded_hal;
//! # use embedded_hal_mock::eh1::serial::{
//! #     Mock as SerialMock,
//! #     Transaction as SerialTransaction,
//! # };
//! use embedded_hal_nb::nb;
//! use embedded_hal_nb::serial::{Read, Write, ErrorKind};
//!
//! // Configure expectations
//! let expectations = [
//!     SerialTransaction::read(42),
//!     SerialTransaction::read_error(nb::Error::WouldBlock),
//!     SerialTransaction::write_error(23, nb::Error::Other(ErrorKind::Other)),
//!     SerialTransaction::flush_error(nb::Error::Other(ErrorKind::Parity)),
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
//!     nb::Error::Other(ErrorKind::Other)
//! );
//! assert_eq!(
//!     serial.flush().unwrap_err(),
//!     nb::Error::Other(ErrorKind::Parity)
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

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use embedded_hal_nb::nb;
use embedded_hal_nb::serial;
use embedded_hal_nb::serial::ErrorKind;
use embedded_hal_nb::serial::ErrorType;

use crate::common::DoneCallDetector;

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
    ReadError(nb::Error<ErrorKind>),
    /// A serial write that transmits a word
    Write(Word),
    /// A serial write that returns an error
    WriteError(Word, nb::Error<ErrorKind>),
    /// A flush call
    Flush,
    /// A flush call that returns an error
    FlushError(nb::Error<ErrorKind>),
}

/// A serial transaction
///
/// Transactions can either be reads, writes, or flushes. A
/// collection of transactions represent the expected operations
/// that are performed on your serial device.
///
/// # Example
///
/// ```no_run
/// use embedded_hal_mock::eh1::serial::Transaction;
/// use embedded_hal_mock::eh1::serial::Mock;
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
/// let mut serial = Mock::new(&transactions);
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
    pub fn read_error(error: nb::Error<ErrorKind>) -> Self {
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
    pub fn write_error(word: Word, error: nb::Error<ErrorKind>) -> Self {
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
    pub fn flush_error(error: nb::Error<ErrorKind>) -> Self {
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
    expected_modes: Arc<Mutex<VecDeque<Mode<Word>>>>,
    done_called: Arc<Mutex<DoneCallDetector>>,
}

impl<Word: Clone> Mock<Word> {
    /// Create a serial mock that will expect the provided transactions
    pub fn new(transactions: &[Transaction<Word>]) -> Self {
        let mut ser = Mock {
            expected_modes: Arc::new(Mutex::new(VecDeque::new())),
            done_called: Arc::new(Mutex::new(DoneCallDetector::new())),
        };
        ser.update_expectations(transactions);
        ser
    }

    /// Update expectations on the interface
    ///
    /// When this method is called, first it is ensured that existing
    /// expectations are all consumed by calling [`done()`](#method.done)
    /// internally (if not called already). Afterwards, the new expectations
    /// are set.
    pub fn update_expectations(&mut self, transactions: &[Transaction<Word>]) {
        // Ensure that existing expectations are consumed
        self.done_impl(false);

        // Lock internal state
        let mut expected = self.expected_modes.lock().unwrap();
        let mut done_called = self.done_called.lock().unwrap();

        // Update expectations
        *expected = transactions
            .iter()
            .fold(VecDeque::new(), |mut modes, transaction| {
                modes.extend(transaction.mode.clone());
                modes
            });

        // Reset done call detector
        done_called.reset();
    }

    /// Deprecated alias of `update_expectations`.
    #[deprecated(
        since = "0.10.0",
        note = "The method 'expect' was renamed to 'update_expectations'"
    )]
    pub fn expect<E>(&mut self, transactions: &[Transaction<Word>]) {
        self.update_expectations(transactions)
    }

    /// Asserts that all expectations up to this point were satisfied.
    /// Panics if there are unsatisfied expectations.
    pub fn done(&mut self) {
        self.done_impl(true);
    }

    fn done_impl(&mut self, panic_if_already_done: bool) {
        self.done_called
            .lock()
            .unwrap()
            .mark_as_called(panic_if_already_done);

        let modes = self
            .expected_modes
            .lock()
            .expect("unable to lock serial mock in call to done");
        assert!(
            modes.is_empty(),
            "serial mock has unsatisfied expectations after call to done"
        );
    }

    /// Pop the next transaction out of the queue
    fn pop(&mut self) -> Option<Mode<Word>> {
        self.expected_modes
            .lock()
            .expect("unable to lock serial mock in call to pop")
            .pop_front()
    }
}

impl<Word> ErrorType for Mock<Word> {
    type Error = ErrorKind;
}

impl<Word> serial::Read<Word> for Mock<Word>
where
    Word: Copy + Clone + std::fmt::Debug,
{
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
    Word: PartialEq + std::fmt::Debug + Copy + Clone,
{
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

#[cfg(test)]
mod test {
    use super::*;

    use embedded_hal_nb::serial::{ErrorKind, Read, Write};

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
    #[should_panic(expected = "serial::write expected to write 18 but actually wrote 20")]
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
    #[should_panic(expected = "serial mock has unsatisfied expectations after call to done")]
    fn test_serial_mock_pending_transactions() {
        let ts = [Transaction::read(0x54)];
        let mut ser = Mock::new(&ts);
        ser.done();
    }

    #[test]
    #[should_panic(expected = "serial mock has unsatisfied expectations after call to done")]
    fn test_serial_mock_reuse_pending_transactions() {
        let ts = [Transaction::read(0x54)];
        let mut ser = Mock::new(&ts);
        let r = ser.read().expect("failed to read");
        assert_eq!(r, 0x54);
        ser.done();
        ser.update_expectations(&ts);
        ser.done();
    }

    #[test]
    #[should_panic(
        expected = "expected to perform a serial transaction 'Write(84)' but instead did a flush"
    )]
    fn test_serial_mock_expected_write() {
        let ts = [Transaction::write(0x54)];
        let mut ser = Mock::new(&ts);
        ser.flush().unwrap();
    }

    #[test]
    #[should_panic(
        expected = "expected to perform a serial transaction 'Flush', but instead did a read"
    )]
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
        ser.done();
    }

    #[test]
    fn test_serial_mock_write_error() {
        let error = nb::Error::Other(ErrorKind::Parity);
        let ts = [Transaction::write_error(42, error.clone())];
        let mut ser: Mock<u8> = Mock::new(&ts);
        assert_eq!(ser.write(42).unwrap_err(), error);
        ser.done();
    }

    #[test]
    #[should_panic(expected = "serial::write expected to write 42 but actually wrote 23")]
    fn test_serial_mock_write_error_wrong_data() {
        let error = nb::Error::Other(ErrorKind::Parity);
        let ts = [Transaction::write_error(42, error.clone())];
        let mut ser: Mock<u8> = Mock::new(&ts);
        // The data to be written should still be verified, even if there's an
        // error attached.
        ser.write(23).unwrap();
    }

    #[test]
    fn test_serial_mock_flush_error() {
        let error = nb::Error::Other(ErrorKind::Overrun);
        let ts = [Transaction::flush_error(error.clone())];
        let mut ser: Mock<u8> = Mock::new(&ts);
        assert_eq!(ser.flush().unwrap_err(), error);
        ser.done();
    }
}
