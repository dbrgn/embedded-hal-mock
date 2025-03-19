//! IO mock implementations.
//!
//! This mock supports the specification and checking of expectations to allow
//! automated testing of `embedded-io` based drivers. Mismatches between expected
//! and real IO transactions will cause runtime assertions to assist with locating
//! faults.
//!
//! ## Usage
//!
//! ```
//! use embedded_io::{Read, Write};
//! use embedded_hal_mock::eh1::io::{Mock as IoMock, Transaction as IoTransaction};
//!
//! // Configure expectations
//! let expectations = [
//!     IoTransaction::write(vec![10, 20]),
//!     IoTransaction::read(vec![20, 30]),
//!     IoTransaction::flush(),
//! ];
//!
//! let mut io = IoMock::new(&expectations);
//!
//! // Writing
//! let bytes_written = io.write(&[10, 20]).unwrap();
//! assert_eq!(bytes_written, 2);
//!
//! // Reading
//! let mut buffer = [0; 2];
//! let bytes_read = io.read(&mut buffer).unwrap();
//! assert_eq!(buffer, [20, 30]);
//! assert_eq!(bytes_read, 2);
//!
//! // Flushing
//! io.flush().unwrap();
//!
//! // Finalizing expectations
//! io.done();
//!
//! // Async is supported with the optional `embedded-hal-async` feature.
//! #[cfg(feature = "embedded-hal-async")]
//! async {
//!     use embedded_io_async;
//!     let mut io = IoMock::new(&[IoTransaction::write(vec![10, 20])]);
//!     let bytes_written = embedded_io_async::Write::write(&mut io, &[10, 20]).await.unwrap();
//!     assert_eq!(bytes_written, 2);
//!     io.done();
//! };
//!
//! ```

use embedded_io::{
    self, BufRead, ErrorKind, ErrorType, Read, ReadReady, Seek, SeekFrom, Write, WriteReady,
};

use crate::common::Generic;

/// IO Transaction mode
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Write transaction
    Write,
    /// Read transaction
    Read,
    /// Flush transaction
    Flush,
    /// Seek transaction
    Seek(SeekFrom),
    /// WriteReady transaction
    WriteReady(bool),
    /// ReadReady transaction
    ReadReady(bool),
    /// FillBuff transaction
    FillBuff,
    /// Consume transaction
    Consume(usize),
}

/// IO transaction type
///
/// Models an IO write or transfer (with response)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    expected_mode: Mode,
    expected_data: Vec<u8>,
    response: Vec<u8>,
    /// An optional error return for a transaction.
    ///
    /// This is in addition to the mode to allow validation that the
    /// transaction mode is correct prior to returning the error.
    expected_err: Option<ErrorKind>,
}

impl Transaction {
    /// Create a write transaction
    pub fn write(expected_data: Vec<u8>) -> Transaction {
        Transaction {
            expected_mode: Mode::Write,
            expected_data,
            response: Vec::new(),
            expected_err: None,
        }
    }

    /// Create a read transaction
    pub fn read(response: Vec<u8>) -> Transaction {
        Transaction {
            expected_mode: Mode::Read,
            expected_data: Vec::new(),
            response,
            expected_err: None,
        }
    }

    /// Create a flush transaction
    pub fn flush() -> Transaction {
        Transaction {
            expected_mode: Mode::Flush,
            expected_data: Vec::new(),
            response: Vec::new(),
            expected_err: None,
        }
    }

    /// Create a seek transaction
    pub fn seek(pos: SeekFrom, ret_offset: u64) -> Transaction {
        Transaction {
            expected_mode: Mode::Seek(pos),
            expected_data: Vec::new(),
            response: ret_offset.to_be_bytes().to_vec(),
            expected_err: None,
        }
    }

    /// Create a write ready transaction
    pub fn write_ready(ready: bool) -> Transaction {
        Transaction {
            expected_mode: Mode::WriteReady(ready),
            expected_data: Vec::new(),
            response: Vec::new(),
            expected_err: None,
        }
    }

    /// Create a read ready transaction
    pub fn read_ready(ready: bool) -> Transaction {
        Transaction {
            expected_mode: Mode::ReadReady(ready),
            expected_data: Vec::new(),
            response: Vec::new(),
            expected_err: None,
        }
    }

    /// Create a fill buffer transaction
    pub fn fill_buf(response: Vec<u8>) -> Transaction {
        Transaction {
            expected_mode: Mode::FillBuff,
            expected_data: Vec::new(),
            response,
            expected_err: None,
        }
    }

    /// Create a consume transaction
    pub fn consume(consumed: usize) -> Transaction {
        Transaction {
            expected_mode: Mode::Consume(consumed),
            expected_data: Vec::new(),
            response: Vec::new(),
            expected_err: None,
        }
    }

    /// Add an error return to a transaction
    ///
    /// This is used to mock failure behaviors.
    ///
    /// Note: When attaching this to a read transaction, the response in the
    /// expectation will not actually be written to the buffer.
    pub fn with_error(mut self, error: ErrorKind) -> Self {
        self.expected_err = Some(error);
        self
    }
}

/// Mock IO implementation
pub type Mock = Generic<Transaction, Vec<u8>>;

impl ErrorType for Mock {
    type Error = ErrorKind;
}

impl Write for Mock {
    fn write(&mut self, buffer: &[u8]) -> Result<usize, Self::Error> {
        let transaction = self.next().expect("no expectation for io::write call");
        assert_eq!(
            transaction.expected_mode,
            Mode::Write,
            "io::write unexpected mode"
        );
        assert_eq!(
            &transaction.expected_data, &buffer,
            "io::write data does not match expectation"
        );

        match transaction.expected_err {
            Some(err) => Err(err),
            None => Ok(buffer.len()),
        }
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        let transaction = self.next().expect("no expectation for io::flush call");
        assert_eq!(
            transaction.expected_mode,
            Mode::Flush,
            "io::flush unexpected mode"
        );

        match transaction.expected_err {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}

impl Read for Mock {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        let transaction = self.next().expect("no expectation for io::read call");
        assert_eq!(
            transaction.expected_mode,
            Mode::Read,
            "io::read unexpected mode"
        );

        if transaction.response.len() > buffer.len() {
            panic!("response longer than read buffer for io::read");
        }

        let len = transaction.response.len();
        buffer[..len].copy_from_slice(&transaction.response[..len]);

        match transaction.expected_err {
            Some(err) => Err(err),
            None => Ok(len),
        }
    }
}

impl Seek for Mock {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let transaction = self.next().expect("no expectation for io::seek call");

        if let Mode::Seek(expected_pos) = transaction.expected_mode {
            assert_eq!(expected_pos, pos, "io::seek unexpected pos");

            let ret_offset: u64 = u64::from_be_bytes(transaction.response.try_into().unwrap());
            match transaction.expected_err {
                Some(err) => Err(err),
                None => Ok(ret_offset),
            }
        } else {
            panic!(
                "expected seek transaction, but instead encountered {:?}. io::seek unexpected mode",
                transaction.expected_mode
            );
        }
    }
}

impl WriteReady for Mock {
    fn write_ready(&mut self) -> Result<bool, Self::Error> {
        let transaction = self
            .next()
            .expect("no expectation for io::write_ready call");

        match transaction.expected_mode {
            Mode::WriteReady(ready) if transaction.expected_err.is_none() => Ok(ready),
            Mode::WriteReady(_) if transaction.expected_err.is_some() => {
                Err(transaction.expected_err.unwrap())
            }
            _ => panic!(
                "expected write_ready transaction, but instead encountered {:?}. io::write_ready unexpected mode",
                transaction.expected_mode
            ),
        }
    }
}

impl ReadReady for Mock {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        let transaction = self.next().expect("no expectation for io::read_ready call");

        match transaction.expected_mode {
            Mode::ReadReady(ready) if transaction.expected_err.is_none() => Ok(ready),
            Mode::ReadReady(_) if transaction.expected_err.is_some() => {
                Err(transaction.expected_err.unwrap())
            }
            _ => panic!(
                "expected read_ready transaction, but instead encountered {:?}. io::read_ready unexpected mode",
                transaction.expected_mode
            )
        }
    }
}

impl BufRead for Mock {
    fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        let transaction = self.next().expect("no expectation for io::fill_buf call");
        assert_eq!(
            transaction.expected_mode,
            Mode::FillBuff,
            "io::fill_buf unexpected mode"
        );

        self.set_mock_data(Some(transaction.response));

        match transaction.expected_err {
            Some(err) => Err(err),
            None => Ok(self.mock_data().as_ref().unwrap()),
        }
    }

    fn consume(&mut self, amt: usize) {
        let transaction = self.next().expect("no expectation for io::consume call");

        match transaction.expected_mode {
            Mode::Consume(expected_amt) if transaction.expected_err.is_none() => {
                assert_eq!(expected_amt, amt, "io::consume unexpected amount");
            }
            Mode::Consume(_) if transaction.expected_err.is_some() => {
                panic!("io::consume can't expect an error. io::consume unexpected error");
            }
            _ => panic!(
                "expected consume transaction, but instead encountered {:?}. io::consume unexpected mode",
                transaction.expected_mode
            )
        }
    }
}

#[cfg(feature = "embedded-hal-async")]
impl embedded_io_async::Write for Mock {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        embedded_io::Write::write(self, buf)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        embedded_io::Write::flush(self)
    }
}

#[cfg(feature = "embedded-hal-async")]
impl embedded_io_async::Read for Mock {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        embedded_io::Read::read(self, buf)
    }
}

#[cfg(feature = "embedded-hal-async")]
impl embedded_io_async::Seek for Mock {
    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        embedded_io::Seek::seek(self, pos)
    }
}

#[cfg(feature = "embedded-hal-async")]
impl embedded_io_async::BufRead for Mock {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        embedded_io::BufRead::fill_buf(self)
    }

    fn consume(&mut self, amt: usize) {
        embedded_io::BufRead::consume(self, amt)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_io_mock_write() {
        let mut io = Mock::new(&[Transaction::write(vec![10])]);

        let ret = io.write(&[10]).unwrap();
        assert_eq!(ret, 1);

        io.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_spi_mock_write() {
        use embedded_io_async::Write;
        let mut io = Mock::new(&[Transaction::write(vec![10])]);

        let ret = Write::write(&mut io, &[10]).await.unwrap();
        assert_eq!(ret, 1);

        io.done();
    }

    #[test]
    fn test_io_mock_flush() {
        let mut io = Mock::new(&[Transaction::flush()]);

        io.flush().unwrap();

        io.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_io_mock_flush() {
        use embedded_io_async::Write;

        let mut io = Mock::new(&[Transaction::flush()]);

        Write::flush(&mut io).await.unwrap();

        io.done();
    }

    #[test]
    fn test_io_mock_read() {
        let mut io = Mock::new(&[Transaction::read(vec![10])]);

        let mut buffer = [0; 1];
        let ret = io.read(&mut buffer).unwrap();

        assert_eq!(buffer, [10]);
        assert_eq!(ret, 1);

        io.done();
    }

    #[test]
    fn test_io_mock_read_buffer_to_long() {
        let mut io = Mock::new(&[Transaction::read(vec![10])]);

        let mut buffer = [0; 3];
        let ret = io.read(&mut buffer).unwrap();

        assert_eq!(buffer, [10, 0, 0]);
        assert_eq!(ret, 1);

        io.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_io_mock_read() {
        use embedded_io_async::Read;

        let mut io = Mock::new(&[Transaction::read(vec![10])]);

        let mut buffer = [0; 1];
        let ret = Read::read(&mut io, &mut buffer).await.unwrap();

        assert_eq!(buffer, [10]);
        assert_eq!(ret, 1);

        io.done();
    }

    #[test]
    fn test_io_mock_seek() {
        let mut io = Mock::new(&[Transaction::seek(SeekFrom::End(10), 90)]);

        let ret = io.seek(SeekFrom::End(10)).unwrap();

        assert_eq!(ret, 90);

        io.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_io_mock_seek() {
        use embedded_io_async::Seek;

        let mut io = Mock::new(&[Transaction::seek(SeekFrom::End(10), 90)]);

        let ret = Seek::seek(&mut io, SeekFrom::End(10)).await.unwrap();

        assert_eq!(ret, 90);

        io.done();
    }

    #[test]
    fn test_io_mock_write_ready() {
        let mut io = Mock::new(&[Transaction::write_ready(false)]);

        let ret = io.write_ready().unwrap();

        assert!(!ret);

        io.done();
    }

    #[test]
    fn test_io_mock_read_ready() {
        let mut io = Mock::new(&[Transaction::read_ready(true)]);

        let ret = io.read_ready().unwrap();

        assert!(ret);

        io.done();
    }

    #[test]
    fn test_io_mock_fill_buf() {
        let mut io = Mock::new(&[
            Transaction::fill_buf(vec![10]),
            Transaction::fill_buf(vec![10, 20, 30]),
        ]);

        let ret = io.fill_buf().unwrap();
        assert_eq!(ret, &[10]);

        let ret = io.fill_buf().unwrap();
        assert_eq!(ret, &[10, 20, 30]);

        io.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_io_mock_fill_buf() {
        use embedded_io_async::BufRead;

        let mut io = Mock::new(&[Transaction::fill_buf(vec![10])]);

        let ret = BufRead::fill_buf(&mut io).await.unwrap();

        assert_eq!(ret, &[10]);

        io.done();
    }

    #[test]
    fn test_io_mock_consume() {
        let mut io = Mock::new(&[Transaction::consume(10)]);

        io.consume(10);

        io.done();
    }

    #[test]
    fn test_io_mock_multiple() {
        let mut io = Mock::new(&[
            Transaction::write(vec![10, 20]),
            Transaction::read(vec![20, 30]),
            Transaction::flush(),
            Transaction::seek(SeekFrom::End(30), 40),
            Transaction::write_ready(false),
            Transaction::read_ready(true),
            Transaction::fill_buf(vec![50, 100, 150]),
            Transaction::consume(60),
        ]);

        let ret = io.write(&[10, 20]).unwrap();
        assert_eq!(ret, 2);

        let mut buffer = [0; 2];
        let ret = io.read(&mut buffer).unwrap();
        assert_eq!(buffer, [20, 30]);
        assert_eq!(ret, 2);

        io.flush().unwrap();

        let ret = io.seek(SeekFrom::End(30)).unwrap();
        assert_eq!(ret, 40);

        let ret = io.write_ready().unwrap();
        assert!(!ret);

        let ret = io.read_ready().unwrap();
        assert!(ret);

        let ret = io.fill_buf().unwrap();
        assert_eq!(ret, &[50, 100, 150]);

        io.consume(60);

        io.done();
    }

    #[test]
    #[should_panic(expected = "io::write data does not match expectation")]
    fn test_io_mock_write_err() {
        let mut io = Mock::new(&[Transaction::write(vec![10, 12])]);

        io.write(&[10, 12, 12]).unwrap();

        io.done();
    }

    #[test]
    #[should_panic(expected = "response longer than read buffer for io::read")]
    fn test_io_mock_read_buffer_to_short() {
        let mut io = Mock::new(&[Transaction::read(vec![10, 20, 30])]);

        let mut buffer = [0; 1];
        let ret = io.read(&mut buffer).unwrap();

        assert_eq!(buffer, [10]);
        assert_eq!(ret, 1);

        io.done();
    }

    #[test]
    #[should_panic(expected = "io::seek unexpected pos")]
    fn test_io_mock_seek_err() {
        let mut io = Mock::new(&[Transaction::seek(SeekFrom::End(2), 0)]);

        io.seek(SeekFrom::Start(1)).unwrap();

        io.done();
    }

    #[test]
    #[should_panic(expected = "io::consume unexpected amount")]
    fn test_io_mock_consume_err() {
        let mut io = Mock::new(&[Transaction::consume(10)]);

        io.consume(20);

        io.done();
    }

    #[test]
    #[should_panic(expected = "io::write unexpected mode")]
    fn test_io_mock_mode_err() {
        let mut io = Mock::new(&[Transaction::fill_buf(vec![10])]);

        io.write(&[10]).unwrap();

        io.done();
    }

    #[test]
    #[should_panic(expected = "Not all expectations consumed")]
    fn test_io_mock_not_all_expectations() {
        let mut io = Mock::new(&[Transaction::write(vec![10]), Transaction::write(vec![10])]);

        io.write(&[10]).unwrap();

        io.done();
    }

    #[test]
    fn test_io_mock_write_with_error() {
        let mut io =
            Mock::new(&[Transaction::write(vec![10, 12]).with_error(ErrorKind::PermissionDenied)]);

        let err = io.write(&[10, 12]).unwrap_err();
        assert_eq!(err, ErrorKind::PermissionDenied);

        io.done();
    }

    #[test]
    fn test_io_mock_read_write_with_error() {
        let mut io = Mock::new(&[
            Transaction::read(vec![10, 12]).with_error(ErrorKind::ConnectionReset),
            Transaction::write(vec![10, 12]).with_error(ErrorKind::NotConnected),
        ]);

        let err = io.read(&mut [10, 12]).unwrap_err();
        assert_eq!(err, ErrorKind::ConnectionReset);

        let err = io.write(&[10, 12]).unwrap_err();
        assert_eq!(err, ErrorKind::NotConnected);

        io.done();
    }
}
