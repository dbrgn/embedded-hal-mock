//! Serial mock implementations
//!
//! We may expect serial read and write transactions on a mock
//! Serial device. Note that, in the `embedded_hal` crate, there
//! are the non-blocking serial traits, and there is a blocking
//! `serial::Write` trait that is defined in terms of the non-blocking
//! variant. We may use the same mock to mock both interfaces.
//!
//! ## Usage: non-blocking serial traits
//!
//! ```
//! extern crate embedded_hal;
//! extern crate embedded_hal_mock;
//!
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
//!     SerialTransaction::read(0xFF),
//!     SerialTransaction::write([1, 2]), // (1)
//!     SerialTransaction::flush(),
//! ];
//!
//! let mut serial = SerialMock::new(&expectations);
//!
//! // Expect two reads
//! let word = serial.read().unwrap();
//! assert_eq!(word, 0x0A);
//! let word = serial.read().unwrap();
//! assert_eq!(word, 0xFF);
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
//! // you may call done(), or let the mock drop out without
//! // any other checks
//! serial.done();
//! ```
//!
//! ## Usage: blocking serial trait
//!
//! ```
//! extern crate embedded_hal;
//! extern crate embedded_hal_mock;
//!
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
//!     SerialTransaction::read(0xFF),
//!     SerialTransaction::write([1, 2]), // (2)
//!     SerialTransaction::flush(),
//! ];
//!
//! let mut serial = SerialMock::new(&expectations);
//!
//! // Expect two reads
//! let word = serial.read().unwrap();
//! assert_eq!(word, 0x0A);
//! let word = serial.read().unwrap();
//! assert_eq!(word, 0xFF);
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
//! // you may call done(), or let the mock drop out without
//! // any other checks
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
use std::marker::PhantomData;
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
    /// A serial read that produces a word
    Read(Word),
    /// A serial write that transmits a word
    Write(Word),
    /// A flush call
    Flush,
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
/// // 1. A read that producers 0x23,
/// // 2. A write of [0x55, 0xAA]
/// // 3. A flush
/// let transactions = [
///     Transaction::read(0x23),
///     Transaction::write([0x55, 0xAA]),
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
    pub fn read(expected: Word) -> Self {
        Transaction {
            mode: vec![Mode::Read(expected)],
        }
    }

    /// Expect a serial write that transmits the specified word
    ///
    /// `write()` accepts a collection of words. If you're expecting
    /// a blocking write, you may want to supply a slice of words that
    /// would be used in the write, rather than listing them out one-by-
    /// one. Or, if you're expecting multiple, non-blocking writes, you
    /// may choose to aggregate them into a slice instead of listing them
    /// out one-by-one.
    pub fn write<Ws>(words: Ws) -> Self
    where
        Ws: AsRef<[Word]>,
    {
        Transaction {
            mode: words.as_ref().iter().cloned().map(Mode::Write).collect(),
        }
    }

    /// Expecte a caller to flush the serial buffers
    pub fn flush() -> Self {
        Transaction {
            mode: vec![Mode::Flush],
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
pub struct Mock<'a, Word> {
    /// The expected operations upon the mock
    ///
    /// It's in an arc to maintain shared state, and in a mutex
    /// to make it thread safe. It's then wrapped in an `Option`
    /// so that we can take it in the call to `done()`.
    expected_modes: Arc<Mutex<Option<VecDeque<Mode<Word>>>>>,
    /// We'll note that this phantom type and lifetime is actually
    /// unnecessary. As noted above, we try to keep a consistent
    /// public API with the existing spi and i2c modules. Given that
    /// this lifetime is necessary for them, it is also necessary here.
    ///
    /// Of course, this could quickly be removed if the overall public
    /// API changes.
    marker: PhantomData<&'a Word>,
}

impl<'a, Word: Clone> Mock<'a, Word> {
    /// Create a serial mock that will expect the provided transactions
    pub fn new(transactions: &'a [Transaction<Word>]) -> Self {
        let expected_modes = transactions
            .iter()
            .fold(VecDeque::new(), |mut modes, transaction| {
                modes.extend(transaction.mode.clone());
                modes
            });
        Mock {
            expected_modes: Arc::new(Mutex::new(Some(expected_modes))),
            marker: PhantomData,
        }
    }

    /// Consumes the mock, asserting that all exepctations were met
    pub fn done(self) {
        // Note that we take self by value, unlike the generic versions that
        // take by reference. It wouldn't make sense to call done() twice anyway :P
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

impl<'a, Word> serial::Read<Word> for Mock<'a, Word>
where
    Word: Clone + std::fmt::Debug,
{
    type Error = MockError;

    fn read(&mut self) -> nb::Result<Word, Self::Error> {
        match self.pop().expect("called serial::read with no expectation") {
            Mode::Read(word) => Ok(word.clone()),
            mode => panic!(
                "expected to perform a serial transaction '{:?}', but instead did a read",
                mode
            ),
        }
    }
}

impl<'a, Word> serial::Write<Word> for Mock<'a, Word>
where
    Word: PartialEq + std::fmt::Debug + Clone,
{
    type Error = MockError;

    fn write(&mut self, word: Word) -> nb::Result<(), Self::Error> {
        match self
            .pop()
            .expect("called serial::write with no expectation")
        {
            Mode::Write(expectation) => {
                assert_eq!(
                    expectation, word,
                    "serial::write expected to write {:?} but actually wrote {:?}",
                    expectation, word
                );
                Ok(())
            }
            mode => panic!(
                "expected to perform a serial transaction '{:?}' but instead did a write of {:?}",
                mode, word
            ),
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        match self
            .pop()
            .expect("called serial::flush with no expectation")
        {
            Mode::Flush => Ok(()),
            mode => panic!(
                "expected to perform a serial transaction '{:?}' but instead did a flush",
                mode
            ),
        }
    }
}

// Note: we attempted to provide our own implementation of
// embedded_hal::blocking::serial::Write. However, we're unable
// to override it due to the blanket default implementation provided by
// the embedded_hal crate. It comes down to the fact that, if we were
// to provide an embedded_hal::blocking::serial::Write implementation
// here, any user of embedded_hal would be free to implement the *defuault*
// version for our type. Therefore, we conform to the default implementation,
// knowing that the default is implemented in terms of the non-blocking
// trait, which is defined above.
//
// If you know a way around this, please let us know!
impl<'a, Word> write::Default<Word> for Mock<'a, Word> where
    Word: PartialEq + std::fmt::Debug + Clone
{
}

#[cfg(test)]
mod test {
    use super::Mock;
    use super::Transaction;
    use embedded_hal::blocking::serial::Write as BWrite;
    use embedded_hal::serial::{Read, Write};

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
        let ts = [Transaction::write([0xAB])];
        let mut ser = Mock::new(&ts);
        ser.write(0xAB).unwrap();
        ser.done();
    }

    #[test]
    fn test_serial_mock_write_many_values_nonblocking() {
        let ts = [Transaction::write([0xAB, 0xCD, 0xEF])];
        let mut ser = Mock::new(&ts);
        ser.write(0xAB).unwrap();
        ser.write(0xCD).unwrap();
        ser.write(0xEF).unwrap();
        ser.done();
    }

    #[test]
    fn test_serial_mock_blocking_write() {
        let ts = [Transaction::write([0xAB, 0xCD, 0xEF])];
        let mut ser = Mock::new(&ts);
        ser.bwrite_all(&[0xAB, 0xCD, 0xEF]).unwrap();
        ser.done();
    }

    #[test]
    #[should_panic]
    fn test_serial_mock_blocking_write_more_than_expected() {
        let ts = [Transaction::write([0xAB, 0xCD])];
        let mut ser = Mock::new(&ts);
        ser.bwrite_all(&[0xAB, 0xCD, 0xEF]).unwrap();
        ser.done();
    }

    #[test]
    #[should_panic]
    fn test_serial_mock_blocking_write_not_enough() {
        let ts = [Transaction::write([0xAB, 0xCD, 0xEF, 0x00])];
        let mut ser = Mock::new(&ts);
        ser.bwrite_all(&[0xAB, 0xCD, 0xEF]).unwrap();
        ser.done();
    }

    #[test]
    #[should_panic]
    fn test_serial_mock_wrong_write() {
        let ts = [Transaction::write([0x12])];
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
    #[should_panic]
    fn test_serial_mock_pending_transactions() {
        let ts = [Transaction::read(0x54)];
        let ser = Mock::new(&ts);
        ser.done();
    }

    #[test]
    #[should_panic]
    fn test_serial_mock_expected_read() {
        let ts = [Transaction::read(0x54)];
        let mut ser = Mock::new(&ts);
        ser.bwrite_all(&[0x77]).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_serial_mock_expected_write() {
        let ts = [Transaction::write([0x54])];
        let mut ser = Mock::new(&ts);
        ser.flush().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_serial_mock_expected_flush() {
        let ts = [Transaction::flush()];
        let mut ser: Mock<u128> = Mock::new(&ts);
        ser.read().unwrap();
    }
}
