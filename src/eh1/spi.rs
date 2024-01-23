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
//! use embedded_hal::spi::SpiBus;
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
//! FullDuplex::write(&mut spi, 0x09).unwrap();
//! assert_eq!(FullDuplex::read(&mut spi).unwrap(), 0x0A);
//! FullDuplex::write(&mut spi, 0xFE).unwrap();
//! assert_eq!(FullDuplex::read(&mut spi).unwrap(), 0xFF);
//!
//! // Writing
//! SpiBus::write(&mut spi, &vec![1, 2]).unwrap();
//!
//! // Transferring
//! let mut buf = vec![3, 4];
//! spi.transfer_in_place(&mut buf).unwrap();
//! assert_eq!(buf, vec![5, 6]);
//!
//! // Finalise expectations
//! spi.done();
//! ```
use core::fmt::Debug;

use eh1::spi::{self, Operation, SpiBus, SpiDevice};
use embedded_hal_nb::{nb, spi::FullDuplex};

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
    /// A delay in the SPI transaction with the specified delay in microseconds
    Delay(u32),
}

/// SPI transaction type
///
/// Models an SPI write or transfer (with response)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transaction<W> {
    pub expected_mode: Mode,
    pub expected_data: Vec<W>,
    pub response: Vec<W>,
}

impl<W> Transaction<W>
where
    W: Copy + Debug + PartialEq,
{
    /// Create a write transaction
    pub fn write_vec(expected: Vec<W>) -> Transaction<W> {
        Transaction {
            expected_mode: Mode::Write,
            expected_data: expected,
            response: Vec::new(),
        }
    }

    /// Create a transfer transaction
    pub fn transfer(expected: Vec<W>, response: Vec<W>) -> Transaction<W> {
        Transaction {
            expected_mode: Mode::Transfer,
            expected_data: expected,
            response,
        }
    }

    /// Create a transfer in-place transaction
    pub fn transfer_in_place(expected: Vec<W>, response: Vec<W>) -> Transaction<W> {
        Transaction {
            expected_mode: Mode::TransferInplace,
            expected_data: expected,
            response,
        }
    }

    /// Create a write transaction
    pub fn write(expected: W) -> Transaction<W> {
        Transaction {
            expected_mode: Mode::Write,
            expected_data: [expected].to_vec(),
            response: Vec::new(),
        }
    }

    /// Create a read transaction
    pub fn read(response: W) -> Transaction<W> {
        Transaction {
            expected_mode: Mode::Read,
            expected_data: Vec::new(),
            response: [response].to_vec(),
        }
    }

    /// Create a read transaction
    pub fn read_vec(response: Vec<W>) -> Transaction<W> {
        Transaction {
            expected_mode: Mode::Read,
            expected_data: Vec::new(),
            response,
        }
    }

    /// Create flush transaction
    pub fn flush() -> Transaction<W> {
        Transaction {
            expected_mode: Mode::Flush,
            expected_data: Vec::new(),
            response: Vec::new(),
        }
    }

    /// Create nested transactions
    pub fn transaction_start() -> Transaction<W> {
        Transaction {
            expected_mode: Mode::TransactionStart,
            expected_data: Vec::new(),
            response: Vec::new(),
        }
    }

    /// Create nested transactions
    pub fn transaction_end() -> Transaction<W> {
        Transaction {
            expected_mode: Mode::TransactionEnd,
            expected_data: Vec::new(),
            response: Vec::new(),
        }
    }

    /// Create a delay transaction
    pub fn delay(delay: u32) -> Transaction<W> {
        Transaction {
            expected_mode: Mode::Delay(delay),
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
pub type Mock<W> = Generic<Transaction<W>>;

impl Mock<u8> {
    pub fn expect_transaction_start(&self) -> Expectation {
        Expectation::Spi(Transaction::transaction_start())
    }

    pub fn expect_transaction_end(&self) -> Expectation {
        Expectation::Spi(Transaction::transaction_end())
    }

    pub fn expect_write(&self, value: u8) -> Expectation {
        Expectation::Spi(Transaction::write(value))
    }

    pub fn expect_write_vec(&self, values: Vec<u8>) -> Expectation {
        Expectation::Spi(Transaction::write_vec(values))
    }
}

impl<W> spi::ErrorType for Mock<W>
where
    W: Copy + Debug + PartialEq,
{
    type Error = spi::ErrorKind;
}

#[derive(Default)]
struct SpiBusFuture {
    awaited: bool,
}

impl std::future::Future for SpiBusFuture {
    type Output = Result<(), spi::ErrorKind>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.awaited = true;
        std::task::Poll::Ready(Ok(()))
    }
}

impl Drop for SpiBusFuture {
    fn drop(&mut self) {
        assert!(self.awaited, "spi::flush call was not awaited");
    }
}

impl SpiBus<u8> for Mock<u8> {
    /// spi::Read implementation for Mock
    ///
    /// This will cause an assertion if the read call does not match the next expectation
    fn read(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let w = next_transaction(self);
        assert_eq!(w.expected_mode, Mode::Read, "spi::read unexpected mode");
        assert_eq!(
            buffer.len(),
            w.response.len(),
            "spi:read mismatched response length"
        );
        buffer.copy_from_slice(&w.response);
        Ok(())
    }

    /// spi::Write implementation for Mock
    ///
    /// This will cause an assertion if the write call does not match the next expectation
    fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        let w = next_transaction(self);
        assert_eq!(w.expected_mode, Mode::Write, "spi::write unexpected mode");
        assert_eq!(
            &w.expected_data, &buffer,
            "spi::write data does not match expectation"
        );
        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        let w = next_transaction(self);
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

    /// spi::TransferInplace implementation for Mock
    ///
    /// This writes the provided response to the buffer and will cause an assertion if the written data does not match the next expectation
    fn transfer_in_place(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let w = next_transaction(self);
        assert_eq!(
            w.expected_mode,
            Mode::TransferInplace,
            "spi::transfer_in_place unexpected mode"
        );
        assert_eq!(
            &w.expected_data, &buffer,
            "spi::transfer_in_place write data does not match expectation"
        );
        assert_eq!(
            buffer.len(),
            w.response.len(),
            "mismatched response length for spi::transfer_in_place"
        );
        buffer.copy_from_slice(&w.response);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        let w = next_transaction(self);
        assert_eq!(w.expected_mode, Mode::Flush, "spi::flush unexpected mode");
        Ok(())
    }
}

#[cfg(feature = "embedded-hal-async")]
impl<W> embedded_hal_async::spi::SpiBus<W> for Mock<W>
where
    W: Copy + 'static + Debug + PartialEq,
{
    async fn read(&mut self, words: &mut [W]) -> Result<(), Self::Error> {
        eh1::spi::SpiBus::<W>::read(self, words)
    }

    async fn write(&mut self, words: &[W]) -> Result<(), Self::Error> {
        eh1::spi::SpiBus::<W>::write(self, words)
    }

    async fn transfer(&mut self, read: &mut [W], write: &[W]) -> Result<(), Self::Error> {
        eh1::spi::SpiBus::<W>::transfer(self, read, write)
    }

    /// spi::TransferInplace implementation for Mock
    ///
    /// This writes the provided response to the buffer and will cause an assertion if the written data does not match the next expectation
    async fn transfer_in_place(&mut self, words: &mut [W]) -> Result<(), Self::Error> {
        eh1::spi::SpiBus::<W>::transfer_in_place(self, words)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        eh1::spi::SpiBus::flush(self)
    }
}

impl<W> FullDuplex<W> for Mock<W>
where
    W: Copy + Debug + PartialEq,
{
    /// spi::FullDuplex implementation for Mock
    ///
    /// This will call the nonblocking read/write primitives.
    fn write(&mut self, buffer: W) -> nb::Result<(), Self::Error> {
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
    fn read(&mut self) -> nb::Result<W, Self::Error> {
        let w = self.next().expect("no expectation for spi::read call");
        assert_eq!(w.expected_mode, Mode::Read, "spi::Read unexpected mode");
        assert_eq!(
            1,
            w.response.len(),
            "mismatched response length for spi::read"
        );
        let buffer: W = w.response[0];
        Ok(buffer)
    }
}

use crate::eh1::top_level::Expectation;

impl TryFrom<Expectation> for Transaction<u8> {
    type Error = ();

    fn try_from(expectation: Expectation) -> Result<Self, <Self as TryFrom<Expectation>>::Error> {
        match expectation {
            Expectation::Spi(transaction) => Ok(transaction),
            _ => Err(())
        }
    }
}

fn next_transaction(mock: &mut Generic<Transaction<u8>>) -> Transaction<u8> {
    if let Some(hal) = &mock.hal {
        hal.lock().unwrap().next().unwrap().try_into().expect("wrong expectation type")
    } else {
        mock.next().unwrap()
    }
}

impl SpiDevice<u8> for Mock<u8> {
    /// spi::SpiDevice implementation for Mock
    ///
    /// This writes the provided response to the buffer and will cause an assertion if the written data does not match the next expectation
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        let w = next_transaction(self);
        assert_eq!(
            w.expected_mode,
            Mode::TransactionStart,
            "spi::transaction unexpected mode"
        );

        for op in operations {
            match op {
                Operation::Read(buffer) => {
                    SpiBus::read(self, buffer)?;
                }
                Operation::Write(buffer) => {
                    SpiBus::write(self, buffer)?;
                }
                Operation::Transfer(read, write) => {
                    SpiBus::transfer(self, read, write)?;
                }
                Operation::TransferInPlace(buffer) => {
                    SpiBus::transfer_in_place(self, buffer)?;
                }
                Operation::DelayNs(delay) => {
                    let w = next_transaction(self);
                    assert_eq!(
                        w.expected_mode,
                        Mode::Delay(*delay),
                        "spi::transaction unexpected mode"
                    );
                }
            }
        }

        let w = next_transaction(self);
        assert_eq!(
            w.expected_mode,
            Mode::TransactionEnd,
            "spi::transaction unexpected mode"
        );

        Ok(())
    }
}

#[cfg(feature = "embedded-hal-async")]
impl<W> embedded_hal_async::spi::SpiDevice<W> for Mock<W>
where
    W: Copy + 'static + Debug + PartialEq,
{
    async fn transaction(
        &mut self,
        operations: &mut [Operation<'_, W>],
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
                    SpiBus::read(self, buffer)?;
                }
                Operation::Write(buffer) => {
                    SpiBus::write(self, buffer)?;
                }
                Operation::Transfer(read, write) => {
                    SpiBus::transfer(self, read, write)?;
                }
                Operation::TransferInPlace(buffer) => {
                    SpiBus::transfer_in_place(self, buffer)?;
                }
                Operation::DelayNs(delay) => {
                    let w = self.next().expect("no expectation for spi::delay call");
                    assert_eq!(
                        w.expected_mode,
                        Mode::Delay(*delay),
                        "spi::transaction unexpected mode"
                    );
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
        use eh1::spi::SpiBus;

        let mut spi = Mock::new(&[Transaction::write(10)]);

        let _ = SpiBus::write(&mut spi, &[10]).unwrap();

        spi.done();
    }

    // #[test]
    // fn test_spi_mock_write_u16() {
    //     let mut spi = Mock::new(&[Transaction::write(0xFFFF_u16)]);

    //     let _ = SpiBus::write(&mut spi, &[0xFFFF_u16]).unwrap();

    //     spi.done();
    // }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_spi_mock_write() {
        use embedded_hal_async::spi::SpiBus;

        let mut spi = Mock::new(&[Transaction::write(10)]);

        let _ = SpiBus::write(&mut spi, &[10]).await.unwrap();

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

    // #[test]
    // fn test_spi_mock_read_duplex_u16() {
    //     use embedded_hal_nb::spi::FullDuplex;

    //     let mut spi = Mock::new(&[Transaction::read(0xFFFF_u16)]);

    //     let ans = FullDuplex::read(&mut spi).unwrap();

    //     assert_eq!(ans, 0xFFFF_u16);

    //     spi.done();
    // }

    #[test]
    fn test_spi_mock_read_bus() {
        use eh1::spi::SpiBus;

        let mut spi = Mock::new(&[Transaction::read(10)]);

        let mut buff = vec![0u8; 1];
        SpiBus::read(&mut spi, &mut buff).unwrap();

        assert_eq!(buff, [10]);

        spi.done();
    }

    // #[test]
    // fn test_spi_mock_read_bus_u16() {
    //     use eh1::spi::SpiBus;

    //     let mut spi = Mock::new(&[Transaction::read(0xFFFF_u16)]);

    //     let mut buff = vec![0u16; 1];
    //     SpiBus::read(&mut spi, &mut buff).unwrap();

    //     assert_eq!(buff, [0xFFFF_u16]);

    //     spi.done();
    // }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_spi_mock_read_bus() {
        use embedded_hal_async::spi::SpiBus;

        let mut spi = Mock::new(&[Transaction::read(10)]);

        let mut buff = vec![0u8; 1];
        SpiBus::read(&mut spi, &mut buff).await.unwrap();

        assert_eq!(buff, [10]);

        spi.done();
    }

    #[test]
    fn test_spi_mock_flush() {
        use eh1::spi::SpiBus;

        let mut spi = Mock::new(&[Transaction::<u8>::flush()]);
        spi.flush().unwrap();
        spi.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_spi_mock_flush() {
        use embedded_hal_async::spi::SpiBus;

        let mut spi = Mock::new(&[Transaction::<u8>::flush()]);
        SpiBus::flush(&mut spi).await.unwrap();
        spi.done();
    }

    #[test]
    fn test_spi_mock_multiple1() {
        use eh1::spi::SpiBus;

        let expectations = [
            Transaction::write_vec(vec![1, 2]),
            Transaction::write(9),
            Transaction::read(10),
            Transaction::write(0xFE),
            Transaction::read(0xFF),
            Transaction::transfer_in_place(vec![3, 4], vec![5, 6]),
        ];
        let mut spi = Mock::new(&expectations);

        SpiBus::write(&mut spi, &[1, 2]).unwrap();

        let _ = SpiBus::write(&mut spi, &[0x09]);
        assert_eq!(FullDuplex::read(&mut spi).unwrap(), 0x0a);
        let _ = SpiBus::write(&mut spi, &[0xfe]);
        assert_eq!(FullDuplex::read(&mut spi).unwrap(), 0xFF);
        let mut v = vec![3, 4];
        SpiBus::transfer_in_place(&mut spi, &mut v).unwrap();

        assert_eq!(v, vec![5, 6]);

        spi.done();
    }

    fn test_spi_mock_multiple_transaction_expectations() -> Mock<u8> {
        let expectations = [
            Transaction::transaction_start(),
            Transaction::write_vec(vec![1, 2]),
            Transaction::write(9),
            Transaction::delay(100),
            Transaction::read(10),
            Transaction::transaction_end(),
        ];
        Mock::new(&expectations)
    }

    #[test]
    fn test_spi_mock_multiple_transaction() {
        use eh1::spi::SpiDevice;

        let mut spi = test_spi_mock_multiple_transaction_expectations();
        let mut ans = [0u8; 1];
        spi.transaction(&mut [
            Operation::Write(&[1, 2]),
            Operation::Write(&[0x09]),
            Operation::DelayNs(100),
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
        SpiDevice::transaction(
            &mut spi,
            &mut [
                Operation::Write(&[1, 2]),
                Operation::Write(&[0x09]),
                Operation::DelayNs(100),
                Operation::Read(&mut ans),
            ],
        )
        .await
        .unwrap();

        assert_eq!(ans, [10]);

        spi.done();
    }

    #[test]
    fn test_spi_mock_write_vec() {
        use eh1::spi::SpiBus;

        let expectations = [Transaction::write_vec(vec![10, 12])];
        let mut spi = Mock::new(&expectations);

        SpiBus::write(&mut spi, &[10, 12]).unwrap();

        spi.done();
    }

    // #[test]
    // fn test_spi_mock_write_vec_u32() {
    //     use eh1::spi::SpiBus;

    //     let expectations = [Transaction::write_vec(vec![0xFFAABBCC_u32, 12])];
    //     let mut spi = Mock::new(&expectations);

    //     SpiBus::write(&mut spi, &[0xFFAABBCC_u32, 12]).unwrap();

    //     spi.done();
    // }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_spi_mock_write_vec() {
        use embedded_hal_async::spi::SpiBus;

        let expectations = [Transaction::write_vec(vec![10, 12])];
        let mut spi = Mock::new(&expectations);

        SpiBus::write(&mut spi, &[10, 12]).await.unwrap();

        spi.done();
    }

    #[test]
    fn test_spi_mock_transfer_in_place() {
        use eh1::spi::SpiBus;

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
        use eh1::spi::SpiBus;

        let expectations = [Transaction::transfer(vec![10, 12], vec![12, 13])];
        let mut spi = Mock::new(&expectations);

        let mut v = vec![10, 12];
        SpiBus::transfer(&mut spi, &mut v, &[10, 12]).unwrap();

        assert_eq!(v, vec![12, 13]);

        spi.done();
    }

    // #[test]
    // fn test_spi_mock_transfer_u32() {
    //     use eh1::spi::SpiBus;

    //     let expectations = [Transaction::transfer(
    //         vec![0xFFAABBCC_u32, 12],
    //         vec![0xAABBCCDD_u32, 13],
    //     )];
    //     let mut spi = Mock::new(&expectations);

    //     let mut v = vec![0xFFAABBCC_u32, 12];
    //     SpiBus::transfer(&mut spi, &mut v, &[0xFFAABBCC_u32, 12]).unwrap();

    //     assert_eq!(v, vec![0xAABBCCDD_u32, 13]);

    //     spi.done();
    // }

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
        use eh1::spi::SpiBus;

        let expectations = [
            Transaction::write_vec(vec![1, 2]),
            Transaction::transfer_in_place(vec![3, 4], vec![5, 6]),
        ];
        let mut spi = Mock::new(&expectations);

        SpiBus::write(&mut spi, &[1, 2]).unwrap();

        let mut v = vec![3, 4];
        SpiBus::transfer_in_place(&mut spi, &mut v).unwrap();

        assert_eq!(v, vec![5, 6]);

        spi.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_async_spi_mock_multiple() {
        use embedded_hal_async::spi::SpiBus;

        let expectations = [
            Transaction::write_vec(vec![1, 2]),
            Transaction::transfer_in_place(vec![3, 4], vec![5, 6]),
        ];
        let mut spi = Mock::new(&expectations);

        SpiBus::write(&mut spi, &[1, 2]).await.unwrap();

        let mut v = vec![3, 4];
        SpiBus::transfer_in_place(&mut spi, &mut v).await.unwrap();

        assert_eq!(v, vec![5, 6]);

        spi.done();
    }

    #[test]
    #[should_panic(expected = "spi::write data does not match expectation")]
    fn test_spi_mock_write_err() {
        use eh1::spi::SpiBus;
        let expectations = [Transaction::write_vec(vec![10, 12])];
        let mut spi = Mock::new(&expectations);
        SpiBus::write(&mut spi, &[10, 12, 12]).unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    #[should_panic(expected = "spi::write data does not match expectation")]
    async fn test_async_spi_mock_write_err() {
        use embedded_hal_async::spi::SpiBus;
        let expectations = [Transaction::write_vec(vec![10, 12])];
        let mut spi = Mock::new(&expectations);
        SpiBus::write(&mut spi, &[10, 12, 12]).await.unwrap();
    }

    #[test]
    #[should_panic(expected = "spi::transfer_in_place write data does not match expectation")]
    fn test_spi_mock_transfer_err() {
        use eh1::spi::SpiBus;
        let expectations = [Transaction::transfer_in_place(vec![10, 12], vec![12, 15])];
        let mut spi = Mock::new(&expectations);
        SpiBus::transfer_in_place(&mut spi, &mut vec![10, 13]).unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    #[should_panic(expected = "spi::transfer_in_place write data does not match expectation")]
    async fn test_async_spi_mock_transfer_err() {
        use embedded_hal_async::spi::SpiBus;
        let expectations = [Transaction::transfer_in_place(vec![10, 12], vec![12, 15])];
        let mut spi = Mock::new(&expectations);
        SpiBus::transfer_in_place(&mut spi, &mut vec![10, 13])
            .await
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "spi::write unexpected mode")]
    fn test_spi_mock_mode_err() {
        use eh1::spi::SpiBus;
        let expectations = [Transaction::transfer_in_place(vec![10, 12], vec![])];
        let mut spi = Mock::new(&expectations);
        SpiBus::write(&mut spi, &[10, 12, 12]).unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    #[should_panic(expected = "spi::write unexpected mode")]
    async fn test_async_spi_mock_mode_err() {
        use embedded_hal_async::spi::SpiBus;
        let expectations = [Transaction::transfer_in_place(vec![10, 12], vec![])];
        let mut spi = Mock::new(&expectations);
        SpiBus::write(&mut spi, &[10, 12, 12]).await.unwrap();
    }

    #[test]
    #[should_panic(expected = "spi::write data does not match expectation")]
    fn test_spi_mock_multiple_transaction_err() {
        use eh1::spi::SpiBus;

        let expectations = [
            Transaction::write_vec(vec![10, 12]),
            Transaction::write_vec(vec![10, 12]),
        ];
        let mut spi = Mock::new(&expectations);
        SpiBus::write(&mut spi, &[10, 12, 10]).unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    #[should_panic(expected = "spi::write data does not match expectation")]
    async fn test_async_spi_mock_multiple_transaction_err() {
        use embedded_hal_async::spi::SpiBus;

        let expectations = [
            Transaction::write_vec(vec![10, 12]),
            Transaction::write_vec(vec![10, 12]),
        ];
        let mut spi = Mock::new(&expectations);
        SpiBus::write(&mut spi, &[10, 12, 12]).await.unwrap();
    }
}
