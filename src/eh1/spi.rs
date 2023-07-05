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
//! # use eh1 as embedded_hal;
//! use embedded_hal::spi::{SpiBus, SpiBusWrite};
//! use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
//! use embedded_hal_nb::spi::FullDuplex;
//!
//! // Configure expectations
//! let expectations = [
//!     SpiTransaction::write(0x09),
//!     SpiTransaction::read(0x0A),
//!     SpiTransaction::write(0xFE),
//!     SpiTransaction::read(0xFF),
//!     SpiTransaction::write_vec(vec![1, 2]),
//!     SpiTransaction::transfer_in_place(vec![3, 4], vec![5, 6]),
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
//! SpiBusWrite::write(&mut spi, &vec![1, 2]).unwrap();
//!
//! // Transferring
//! let mut buf = vec![3, 4];
//! spi.transfer_in_place(&mut buf).unwrap();
//! assert_eq!(buf, vec![5, 6]);
//!
//! // Finalise expectations
//! spi.done();
//! ```
use eh1 as embedded_hal;
use embedded_hal::spi;
use embedded_hal::spi::{Operation, SpiBusRead, SpiBusWrite, SpiDeviceRead, SpiDeviceWrite};
use embedded_hal_nb::nb;
use embedded_hal_nb::spi::FullDuplex;

use crate::common::Generic;

/// SPI Transaction mode
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Write transaction
    Write,
    /// Write and read transaction
    Transfer,
    /// Write and read in-place transaction
    TransferInplace,
    /// After a write transaction in real HW a Read is available
    Read,
    /// Flush transaction
    Flush,
    /// Mark the start of a group of transactions
    TransactionStart,
    /// Mark the end of a group of transactions
    TransactionEnd,
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
    pub fn transfer(expected: Vec<u8>, response: Vec<u8>) -> Transaction {
        Transaction {
            expected_mode: Mode::Transfer,
            expected_data: expected,
            response,
        }
    }

    /// Create a transfer in-place transaction
    pub fn transfer_in_place(expected: Vec<u8>, response: Vec<u8>) -> Transaction {
        Transaction {
            expected_mode: Mode::TransferInplace,
            expected_data: expected,
            response,
        }
    }

    /// Create a write transaction
    pub fn write(expected: u8) -> Transaction {
        Transaction {
            expected_mode: Mode::Write,
            expected_data: [expected].to_vec(),
            response: Vec::new(),
        }
    }

    /// Create a read transaction
    pub fn read(response: u8) -> Transaction {
        Transaction {
            expected_mode: Mode::Read,
            expected_data: Vec::new(),
            response: [response].to_vec(),
        }
    }

    /// Create a read transaction
    pub fn read_vec(response: Vec<u8>) -> Transaction {
        Transaction {
            expected_mode: Mode::Read,
            expected_data: Vec::new(),
            response,
        }
    }

    /// Create flush transaction
    pub fn flush() -> Transaction {
        Transaction {
            expected_mode: Mode::Flush,
            expected_data: Vec::new(),
            response: Vec::new(),
        }
    }

    /// Create nested transactions
    pub fn transaction_start() -> Transaction {
        Transaction {
            expected_mode: Mode::TransactionStart,
            expected_data: Vec::new(),
            response: Vec::new(),
        }
    }

    /// Create nested transactions
    pub fn transaction_end() -> Transaction {
        Transaction {
            expected_mode: Mode::TransactionEnd,
            expected_data: Vec::new(),
            response: Vec::new(),
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

impl embedded_hal::spi::ErrorType for Mock {
    type Error = embedded_hal::spi::ErrorKind;
}

impl embedded_hal::spi::SpiBusFlush for Mock {
    fn flush(&mut self) -> Result<(), Self::Error> {
        let w = self.next().expect("no expectation for spi::flush call");
        assert_eq!(w.expected_mode, Mode::Flush, "spi::flush unexpected mode");
        Ok(())
    }
}

#[derive(Default)]
struct SpiBusFlushFuture {
    awaited: bool,
}

impl std::future::Future for SpiBusFlushFuture {
    type Output = Result<(), embedded_hal::spi::ErrorKind>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.awaited = true;
        std::task::Poll::Ready(Ok(()))
    }
}

impl Drop for SpiBusFlushFuture {
    fn drop(&mut self) {
        assert!(self.awaited, "spi::flush call was not awaited");
    }
}

#[cfg(feature = "embedded-hal-async")]
impl embedded_hal_async::spi::SpiBusFlush for Mock {
    async fn flush(&mut self) -> Result<(), Self::Error> {
        embedded_hal::spi::SpiBusFlush::flush(self)
    }
}

impl embedded_hal::spi::SpiBus<u8> for Mock {
    /// spi::TransferInplace implementation for Mock
    ///
    /// This writes the provided response to the buffer and will cause an assertion if the written data does not match the next expectation
    fn transfer_in_place(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let w = self.next().expect("no expectation for spi::transfer call");
        assert_eq!(
            w.expected_mode,
            Mode::TransferInplace,
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

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        let w = self.next().expect("no expectation for spi::transfer call");
        assert_eq!(
            w.expected_mode,
            Mode::Transfer,
            "spi::transfer unexpected mode"
        );
        assert_eq!(
            &w.expected_data, &write,
            "spi::write data does not match expectation"
        );
        assert_eq!(
            read.len(),
            w.response.len(),
            "mismatched response length for spi::transfer"
        );
        read.copy_from_slice(&w.response);
        Ok(())
    }
}

#[cfg(feature = "embedded-hal-async")]
impl embedded_hal_async::spi::SpiBus<u8> for Mock {
    /// spi::TransferInplace implementation for Mock
    ///
    /// This writes the provided response to the buffer and will cause an assertion if the written data does not match the next expectation
    async fn transfer_in_place<'a>(&'a mut self, buffer: &'a mut [u8]) -> Result<(), Self::Error> {
        embedded_hal::spi::SpiBus::<u8>::transfer_in_place(self, buffer)
    }

    async fn transfer<'a>(
        &'a mut self,
        read: &'a mut [u8],
        write: &'a [u8],
    ) -> Result<(), Self::Error> {
        embedded_hal::spi::SpiBus::<u8>::transfer(self, read, write)
    }
}

impl embedded_hal::spi::SpiBusWrite<u8> for Mock {
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

#[cfg(feature = "embedded-hal-async")]
impl embedded_hal_async::spi::SpiBusWrite<u8> for Mock {
    async fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        embedded_hal::spi::SpiBusWrite::<u8>::write(self, words)
    }
}

impl embedded_hal::spi::SpiBusRead<u8> for Mock {
    /// spi::Read implementation for Mock
    ///
    /// This will cause an assertion if the read call does not match the next expectation
    fn read(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let w = self.next().expect("no expectation for spi::read call");
        assert_eq!(w.expected_mode, Mode::Read, "spi::read unexpected mode");
        assert_eq!(
            buffer.len(),
            w.response.len(),
            "spi:read mismatched response length"
        );
        buffer.copy_from_slice(&w.response);
        Ok(())
    }
}

#[cfg(feature = "embedded-hal-async")]
impl embedded_hal_async::spi::SpiBusRead<u8> for Mock {
    async fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        embedded_hal::spi::SpiBusRead::<u8>::read(self, words)
    }
}

impl FullDuplex<u8> for Mock {
    /// spi::FullDuplex implementation for Mock
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

    /// spi::FullDuplex implementation for Mock
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

impl SpiDeviceRead<u8> for Mock {
    fn read_transaction(&mut self, operations: &mut [&mut [u8]]) -> Result<(), Self::Error> {
        for op in operations {
            SpiBusRead::read(self, op)?;
        }

        Ok(())
    }
}

#[cfg(feature = "embedded-hal-async")]
impl embedded_hal_async::spi::SpiDeviceRead<u8> for Mock {
    async fn read_transaction(&mut self, operations: &mut [&mut [u8]]) -> Result<(), Self::Error> {
        embedded_hal::spi::SpiDeviceRead::<u8>::read_transaction(self, operations)
    }
}

impl SpiDeviceWrite<u8> for Mock {
    fn write_transaction(&mut self, operations: &[&[u8]]) -> Result<(), Self::Error> {
        for op in operations {
            SpiBusWrite::write(self, op)?;
        }

        Ok(())
    }
}

#[cfg(feature = "embedded-hal-async")]
impl embedded_hal_async::spi::SpiDeviceWrite<u8> for Mock {
    async fn write_transaction(&mut self, operations: &[&[u8]]) -> Result<(), Self::Error> {
        embedded_hal::spi::SpiDeviceWrite::<u8>::write_transaction(self, operations)
    }
}

impl spi::SpiDevice for Mock {
    /// spi::SpiDevice implementation for Mock
    ///
    /// This writes the provided response to the buffer and will cause an assertion if the written data does not match the next expectation
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        let w = self
            .next()
            .expect("no expectation for spi::transaction call");
        assert_eq!(
            w.expected_mode,
            Mode::TransactionStart,
            "spi::transaction unexpected mode"
        );

        for op in operations {
            match op {
                Operation::Read(buffer) => {
                    SpiBusRead::read(self, buffer)?;
                }
                Operation::Write(buffer) => {
                    SpiBusWrite::write(self, buffer)?;
                }
                Operation::Transfer(read, write) => {
                    spi::SpiBus::transfer(self, read, write)?;
                }
                Operation::TransferInPlace(buffer) => {
                    spi::SpiBus::transfer_in_place(self, buffer)?;
                }
            }
        }

        let w = self
            .next()
            .expect("no expectation for spi::transaction call");
        assert_eq!(
            w.expected_mode,
            Mode::TransactionEnd,
            "spi::transaction unexpected mode"
        );

        Ok(())
    }
}

#[cfg(feature = "embedded-hal-async")]
impl embedded_hal_async::spi::SpiDevice<u8> for Mock {
    async fn transaction(
        &mut self,
        operations: &mut [Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        let w = self
            .next()
            .expect("no expectation for spi::transaction call");
        assert_eq!(
            w.expected_mode,
            Mode::TransactionStart,
            "spi::transaction unexpected mode"
        );
        for op in operations {
            match op {
                Operation::Read(buffer) => {
                    SpiBusRead::read(self, buffer)?;
                }
                Operation::Write(buffer) => {
                    SpiBusWrite::write(self, buffer)?;
                }
                Operation::Transfer(read, write) => {
                    spi::SpiBus::transfer(self, read, write)?;
                }
                Operation::TransferInPlace(buffer) => {
                    spi::SpiBus::transfer_in_place(self, buffer)?;
                }
            }
        }

        let w = self
            .next()
            .expect("no expectation for spi::transaction call");
        assert_eq!(
            w.expected_mode,
            Mode::TransactionEnd,
            "spi::transaction unexpected mode"
        );

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_spi_mock_write() {
        use embedded_hal::spi::SpiBusWrite;

        let mut spi = Mock::new(&[Transaction::write(10)]);

        let _ = SpiBusWrite::write(&mut spi, &[10]).unwrap();

        spi.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_spi_mock_write() {
        use embedded_hal_async::spi::SpiBusWrite;

        let mut spi = Mock::new(&[Transaction::write(10)]);

        let _ = SpiBusWrite::write(&mut spi, &[10]).await.unwrap();

        spi.done();
    }

    #[test]
    fn test_spi_mock_read_duplex() {
        use embedded_hal_nb::spi::FullDuplex;

        let mut spi = Mock::new(&[Transaction::read(10)]);

        let ans = FullDuplex::read(&mut spi).unwrap();

        assert_eq!(ans, 10);

        spi.done();
    }

    #[test]
    fn test_spi_mock_read_bus() {
        use embedded_hal::spi::SpiBusRead;

        let mut spi = Mock::new(&[Transaction::read(10)]);

        let mut buff = vec![0u8; 1];
        SpiBusRead::read(&mut spi, &mut buff).unwrap();

        assert_eq!(buff, [10]);

        spi.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_spi_mock_read_bus() {
        use embedded_hal_async::spi::SpiBusRead;

        let mut spi = Mock::new(&[Transaction::read(10)]);

        let mut buff = vec![0u8; 1];
        SpiBusRead::read(&mut spi, &mut buff).await.unwrap();

        assert_eq!(buff, [10]);

        spi.done();
    }

    #[test]
    fn test_spi_mock_flush() {
        use embedded_hal::spi::SpiBusFlush;

        let mut spi = Mock::new(&[Transaction::flush()]);
        spi.flush().unwrap();
        spi.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_spi_mock_flush() {
        use embedded_hal_async::spi::SpiBusFlush;

        let mut spi = Mock::new(&[Transaction::flush()]);
        spi.flush().await.unwrap();
        spi.done();
    }

    #[test]
    fn test_spi_mock_multiple1() {
        use embedded_hal::spi::{SpiBus, SpiBusWrite};

        let expectations = [
            Transaction::write_vec(vec![1, 2]),
            Transaction::write(9),
            Transaction::read(10),
            Transaction::write(0xFE),
            Transaction::read(0xFF),
            Transaction::transfer_in_place(vec![3, 4], vec![5, 6]),
        ];
        let mut spi = Mock::new(&expectations);

        SpiBusWrite::write(&mut spi, &[1, 2]).unwrap();

        let _ = SpiBusWrite::write(&mut spi, &[0x09]);
        assert_eq!(FullDuplex::read(&mut spi).unwrap(), 0x0a);
        let _ = SpiBusWrite::write(&mut spi, &[0xfe]);
        assert_eq!(FullDuplex::read(&mut spi).unwrap(), 0xFF);
        let mut v = vec![3, 4];
        SpiBus::transfer_in_place(&mut spi, &mut v).unwrap();

        assert_eq!(v, vec![5, 6]);

        spi.done();
    }

    fn test_spi_mock_multiple_transaction_expectations() -> Mock {
        let expectations = [
            Transaction::transaction_start(),
            Transaction::write_vec(vec![1, 2]),
            Transaction::write(9),
            Transaction::read(10),
            Transaction::transaction_end(),
        ];
        Mock::new(&expectations)
    }

    #[test]
    fn test_spi_mock_multiple_transaction() {
        use embedded_hal::spi::SpiDevice;

        let mut spi = test_spi_mock_multiple_transaction_expectations();
        let mut ans = [0u8; 1];
        spi.transaction(&mut [
            Operation::Write(&[1, 2]),
            Operation::Write(&[0x09]),
            Operation::Read(&mut ans),
        ])
        .unwrap();

        assert_eq!(ans, [10]);

        spi.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_spi_mock_multiple_transaction() {
        use embedded_hal_async::spi::SpiDevice;

        let mut spi = test_spi_mock_multiple_transaction_expectations();
        let mut ans = [0u8; 1];
        spi.transaction(&mut [
            Operation::Write(&[1, 2]),
            Operation::Write(&[0x09]),
            Operation::Read(&mut ans),
        ])
        .await
        .unwrap();

        assert_eq!(ans, [10]);

        spi.done();
    }

    #[test]
    fn test_spi_mock_write_vec() {
        use embedded_hal::spi::SpiBusWrite;

        let expectations = [Transaction::write_vec(vec![10, 12])];
        let mut spi = Mock::new(&expectations);

        SpiBusWrite::write(&mut spi, &[10, 12]).unwrap();

        spi.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_spi_mock_write_vec() {
        use embedded_hal_async::spi::SpiBusWrite;

        let expectations = [Transaction::write_vec(vec![10, 12])];
        let mut spi = Mock::new(&expectations);

        SpiBusWrite::write(&mut spi, &[10, 12]).await.unwrap();

        spi.done();
    }

    #[test]
    fn test_spi_mock_transfer_in_place() {
        use embedded_hal::spi::SpiBus;

        let expectations = [Transaction::transfer_in_place(vec![10, 12], vec![12, 13])];
        let mut spi = Mock::new(&expectations);

        let mut v = vec![10, 12];
        SpiBus::transfer_in_place(&mut spi, &mut v).unwrap();

        assert_eq!(v, vec![12, 13]);

        spi.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_spi_mock_transfer_in_place() {
        use embedded_hal_async::spi::SpiBus;

        let expectations = [Transaction::transfer_in_place(vec![10, 12], vec![12, 13])];
        let mut spi = Mock::new(&expectations);

        let mut v = vec![10, 12];
        SpiBus::transfer_in_place(&mut spi, &mut v).await.unwrap();

        assert_eq!(v, vec![12, 13]);

        spi.done();
    }

    #[test]
    fn test_spi_mock_transfer() {
        use embedded_hal::spi::SpiBus;

        let expectations = [Transaction::transfer(vec![10, 12], vec![12, 13])];
        let mut spi = Mock::new(&expectations);

        let mut v = vec![10, 12];
        SpiBus::transfer(&mut spi, &mut v, &[10, 12]).unwrap();

        assert_eq!(v, vec![12, 13]);

        spi.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_spi_mock_transfer() {
        use embedded_hal_async::spi::SpiBus;

        let expectations = [Transaction::transfer(vec![10, 12], vec![12, 13])];
        let mut spi = Mock::new(&expectations);

        let mut v = vec![10, 12];
        SpiBus::transfer(&mut spi, &mut v, &[10, 12]).await.unwrap();

        assert_eq!(v, vec![12, 13]);

        spi.done();
    }

    #[test]
    fn test_spi_mock_multiple() {
        use embedded_hal::spi::{SpiBus, SpiBusWrite};

        let expectations = [
            Transaction::write_vec(vec![1, 2]),
            Transaction::transfer_in_place(vec![3, 4], vec![5, 6]),
        ];
        let mut spi = Mock::new(&expectations);

        SpiBusWrite::write(&mut spi, &[1, 2]).unwrap();

        let mut v = vec![3, 4];
        SpiBus::transfer_in_place(&mut spi, &mut v).unwrap();

        assert_eq!(v, vec![5, 6]);

        spi.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_spi_mock_multiple() {
        use embedded_hal_async::spi::{SpiBus, SpiBusWrite};

        let expectations = [
            Transaction::write_vec(vec![1, 2]),
            Transaction::transfer_in_place(vec![3, 4], vec![5, 6]),
        ];
        let mut spi = Mock::new(&expectations);

        SpiBusWrite::write(&mut spi, &[1, 2]).await.unwrap();

        let mut v = vec![3, 4];
        SpiBus::transfer_in_place(&mut spi, &mut v).await.unwrap();

        assert_eq!(v, vec![5, 6]);

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_write_err() {
        use embedded_hal::spi::SpiBusWrite;

        let expectations = [Transaction::write_vec(vec![10, 12])];
        let mut spi = Mock::new(&expectations);

        SpiBusWrite::write(&mut spi, &[10, 12, 12]).unwrap();

        spi.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    #[should_panic]
    async fn test_async_spi_mock_write_err() {
        use embedded_hal_async::spi::SpiBusWrite;

        let expectations = [Transaction::write_vec(vec![10, 12])];
        let mut spi = Mock::new(&expectations);

        SpiBusWrite::write(&mut spi, &[10, 12, 12]).await.unwrap();

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_transfer_err() {
        use embedded_hal::spi::SpiBus;

        let expectations = [Transaction::transfer_in_place(vec![10, 12], vec![12, 15])];
        let mut spi = Mock::new(&expectations);

        let mut v = vec![10, 12];
        SpiBus::transfer_in_place(&mut spi, &mut v).unwrap();

        assert_eq!(v, vec![12, 13]);

        spi.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    #[should_panic]
    async fn test_async_spi_mock_transfer_err() {
        use embedded_hal_async::spi::SpiBus;

        let expectations = [Transaction::transfer_in_place(vec![10, 12], vec![12, 15])];
        let mut spi = Mock::new(&expectations);

        let mut v = vec![10, 12];
        SpiBus::transfer_in_place(&mut spi, &mut v).await.unwrap();

        assert_eq!(v, vec![12, 13]);

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_transfer_response_err() {
        use embedded_hal::spi::SpiBus;

        let expectations = [Transaction::transfer_in_place(vec![1, 2], vec![3, 4, 5])];
        let mut spi = Mock::new(&expectations);

        let mut v = vec![10, 12];
        SpiBus::transfer_in_place(&mut spi, &mut v).unwrap();

        assert_eq!(v, vec![12, 13]);

        spi.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    #[should_panic]
    async fn test_async_spi_mock_transfer_response_err() {
        use embedded_hal_async::spi::SpiBus;

        let expectations = [Transaction::transfer_in_place(vec![1, 2], vec![3, 4, 5])];
        let mut spi = Mock::new(&expectations);

        let mut v = vec![10, 12];
        SpiBus::transfer_in_place(&mut spi, &mut v).await.unwrap();

        assert_eq!(v, vec![12, 13]);

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_mode_err() {
        use embedded_hal::spi::SpiBusWrite;

        let expectations = [Transaction::transfer_in_place(vec![10, 12], vec![])];
        let mut spi = Mock::new(&expectations);

        SpiBusWrite::write(&mut spi, &[10, 12, 12]).unwrap();

        spi.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    #[should_panic]
    async fn test_async_spi_mock_mode_err() {
        use embedded_hal_async::spi::SpiBusWrite;

        let expectations = [Transaction::transfer_in_place(vec![10, 12], vec![])];
        let mut spi = Mock::new(&expectations);

        SpiBusWrite::write(&mut spi, &[10, 12, 12]).await.unwrap();

        spi.done();
    }

    #[test]
    #[should_panic]
    fn test_spi_mock_multiple_transaction_err() {
        use embedded_hal::spi::SpiBusWrite;

        let expectations = [
            Transaction::write_vec(vec![10, 12]),
            Transaction::write_vec(vec![10, 12]),
        ];
        let mut spi = Mock::new(&expectations);

        SpiBusWrite::write(&mut spi, &[10, 12, 12]).unwrap();

        spi.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    #[should_panic]
    async fn test_async_spi_mock_multiple_transaction_err() {
        use embedded_hal_async::spi::SpiBusWrite;

        let expectations = [
            Transaction::write_vec(vec![10, 12]),
            Transaction::write_vec(vec![10, 12]),
        ];
        let mut spi = Mock::new(&expectations);

        SpiBusWrite::write(&mut spi, &[10, 12, 12]).await.unwrap();

        spi.done();
    }
}
