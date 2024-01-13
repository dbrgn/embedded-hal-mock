//! Delay mock implementations, implementing both sync and async
//! [`BlockingDelay`](https://docs.rs/embedded-hal/latest/embedded_hal/delay/trait.BlockingDelay.html)
//! traits.
//!
//! ## Choosing a Delay Implementation
//!
//! There are three implementations available depending on your use case:
//!
//! - If you want **no actual delay**, create a
//!   [`NoopDelay`](struct.NoopDelay.html) stub. It will always return
//!   immediately, without a delay. This is useful for fast tests, where you
//!   don't actually need to wait for the hardware.
//! - If you do want the **real delay behavior** when running your tests, use
//!   [`StdSleep`](struct.StdSleep.html) stub implementation, which uses
//!   [`std::thread::sleep`](https://doc.rust-lang.org/std/thread/fn.sleep.html)
//!   to implement the delay.
//! - For a **configurable delay** implementation that supports expectations,
//!   use the [`CheckedDelay`](type.CheckedDelay.html) mock. By default it
//!   doesn't perform an actual delay, but allows the user to enable them
//!   individually for each expected call.
//!
//! ## Usage
//!
//! ```
//! # use eh1 as embedded_hal;
//! use std::time::Duration;
//!
//! use embedded_hal::delay::DelayNs;
//! use embedded_hal_mock::eh1::delay::{CheckedDelay, NoopDelay, StdSleep, Transaction};
//!
//! // No actual delay
//!
//! let mut delay = NoopDelay::new();
//! delay.delay_ms(50); // Returns immediately
//!
//! // Real delay
//!
//! let mut delay = StdSleep::new();
//! delay.delay_ms(50); // Will sleep for 50 ms
//!
//! // Configurable mock
//!
//! let transactions = vec![
//!     Transaction::delay_ns(50_000_000),
//!     Transaction::delay_us(60_000),
//!     Transaction::delay_ms(70).wait(),
//! ];
//!
//! let mut delay = CheckedDelay::new(&transactions);
//!
//! delay.delay_ms(50); // Conversion to nanoseconds
//! delay.delay_ms(60); // Conversion to microseconds
//! delay.delay_ms(70); // This will actually delay
//! delay.done();
//!
//! let mut delay = NoopDelay::new();
//! delay.delay_ms(50); // No checks are performed
//! ```

use std::thread;
use std::time::Duration;

use eh1 as embedded_hal;
use embedded_hal::delay;

use crate::common::Generic;

/// Delay transaction
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Transaction {
    /// Kind is the transaction kind (and data) expected
    kind: TransactionKind,
    real_delay: bool,
}

/// Nanoseconds per microsecond
const NANOS_PER_MICRO: u64 = 1_000;
/// Nanoseconds per millisecond
const NANOS_PER_MILLI: u64 = 1_000_000;

impl Transaction {
    /// Create a new delay transaction
    pub fn new(kind: TransactionKind) -> Transaction {
        Transaction {
            kind,
            real_delay: false,
        }
    }

    /// Crete a new delay_ns transaction
    pub fn delay_ns(ns: u32) -> Transaction {
        Transaction::new(TransactionKind::Delay(ns.into()))
    }

    /// Crete a new delay_us transaction
    pub fn delay_us(us: u32) -> Transaction {
        Transaction::new(TransactionKind::Delay(us as u64 * NANOS_PER_MICRO))
    }

    /// Crete a new delay_ms transaction
    pub fn delay_ms(ms: u32) -> Transaction {
        Transaction::new(TransactionKind::Delay(ms as u64 * NANOS_PER_MILLI))
    }

    /// Create a new blocking delay_ns transaction
    pub fn blocking_delay_ns(ns: u32) -> Transaction {
        Transaction::new(TransactionKind::BlockingDelay(ns.into()))
    }

    /// Crete a new blocking delay_us transaction
    pub fn blocking_delay_us(us: u32) -> Transaction {
        Transaction::new(TransactionKind::BlockingDelay(us as u64 * NANOS_PER_MICRO))
    }

    /// Create new blocking delay_ms transaction
    pub fn blocking_delay_ms(ms: u32) -> Transaction {
        Transaction::new(TransactionKind::BlockingDelay(ms as u64 * NANOS_PER_MILLI))
    }

    /// Crete a new async delay_ns transaction
    #[cfg(feature = "embedded-hal-async")]
    pub fn async_delay_ns(ns: u32) -> Transaction {
        Transaction::new(TransactionKind::AsyncDelay(ns.into()))
    }

    /// Crete a new async delay_us transaction
    #[cfg(feature = "embedded-hal-async")]
    pub fn async_delay_us(us: u32) -> Transaction {
        Transaction::new(TransactionKind::AsyncDelay(us as u64 * NANOS_PER_MICRO))
    }

    /// Crete a new async delay_ms transaction
    #[cfg(feature = "embedded-hal-async")]
    pub fn async_delay_ms(ms: u32) -> Transaction {
        Transaction::new(TransactionKind::AsyncDelay(ms as u64 * NANOS_PER_MILLI))
    }

    /// Perform an actual delay for this transaction
    pub fn wait(mut self) -> Transaction {
        self.real_delay = true;
        self
    }
}

/// MockDelay transaction kind.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum TransactionKind {
    /// Any delay in nanoseconds, blocking or async. Should be a default kind for most use cases.
    Delay(u64),
    /// Delay in nanoseconds, must be a blocking delay. Expectation will fail for async delays.
    BlockingDelay(u64),
    /// Delay in nanoseconds, must be an async delay. Expectation will fail for blocking delays.
    AsyncDelay(u64),
}

/// Mock Delay implementation with checked calls
///
/// This supports the specification and checking of expectations to allow
/// automated testing of delay based drivers. Mismatches between expected and
/// real delay transactions will cause runtime assertions to assist with locating
/// faults.
///
/// See the usage section in the module level docs for an example.
pub type CheckedDelay = Generic<Transaction>;

impl delay::DelayNs for CheckedDelay {
    fn delay_ns(&mut self, ns: u32) {
        let transaction = self.next().expect("no expectation for delay call");

        match transaction.kind {
            TransactionKind::BlockingDelay(n) => assert_eq!(n, ns.into(), "wrong delay value"),
            TransactionKind::Delay(n) => assert_eq!(n, ns.into(), "wrong delay value"),
            _ => panic!(
                "Wrong kind of delay. Expected Delay or BlockingDelay got {:?}",
                transaction.kind
            ),
        }

        if transaction.real_delay {
            thread::sleep(Duration::from_nanos(ns as u64));
        }
    }

    fn delay_us(&mut self, us: u32) {
        let transaction = self.next().expect("no expectation for delay call");
        match transaction.kind {
            TransactionKind::BlockingDelay(n) => {
                assert_eq!(n, us as u64 * NANOS_PER_MICRO, "wrong delay value")
            }
            TransactionKind::Delay(n) => {
                assert_eq!(n, us as u64 * NANOS_PER_MICRO, "wrong delay value")
            }
            _ => panic!(
                "Wrong kind of delay. Expected Delay or BlockingDelay got {:?}",
                transaction.kind
            ),
        }
        if transaction.real_delay {
            thread::sleep(Duration::from_micros(us as u64));
        }
    }

    fn delay_ms(&mut self, ms: u32) {
        let transaction = self.next().expect("no expectation for delay call");
        match transaction.kind {
            TransactionKind::BlockingDelay(n) => {
                assert_eq!(n, ms as u64 * NANOS_PER_MILLI, "wrong delay value")
            }
            TransactionKind::Delay(n) => {
                assert_eq!(n, ms as u64 * NANOS_PER_MILLI, "wrong delay value")
            }
            _ => panic!(
                "Wrong kind of delay. Expected Delay or BlockingDelay got {:?}",
                transaction.kind
            ),
        }

        if transaction.real_delay {
            thread::sleep(Duration::from_millis(ms as u64));
        }
    }
}

#[cfg(feature = "embedded-hal-async")]
impl embedded_hal_async::delay::DelayNs for CheckedDelay {
    async fn delay_ns(&mut self, ns: u32) {
        let transaction = self.next().expect("no expectation for delay call");

        match transaction.kind {
            TransactionKind::AsyncDelay(n) => assert_eq!(n, ns.into(), "delay unexpected value"),
            TransactionKind::Delay(n) => assert_eq!(n, ns.into(), "delay unexpected value"),
            _ => panic!(
                "Wrong kind of delay. Expected Delay or AsyncDelay got {:?}",
                transaction.kind
            ),
        }

        if transaction.real_delay {
            thread::sleep(Duration::from_nanos(ns as u64));
        }
    }

    async fn delay_us(&mut self, us: u32) {
        let transaction = self.next().expect("no expectation for delay call");
        match transaction.kind {
            TransactionKind::AsyncDelay(n) => {
                assert_eq!(n, us as u64 * NANOS_PER_MICRO, "wrong delay value")
            }
            TransactionKind::Delay(n) => {
                assert_eq!(n, us as u64 * NANOS_PER_MICRO, "wrong delay value")
            }
            _ => panic!(
                "Wrong kind of delay. Expected Delay or AsyncDelay got {:?}",
                transaction.kind
            ),
        }

        if transaction.real_delay {
            thread::sleep(Duration::from_micros(us as u64));
        }
    }

    async fn delay_ms(&mut self, ms: u32) {
        let transaction = self.next().expect("no expectation for delay call");
        match transaction.kind {
            TransactionKind::AsyncDelay(n) => {
                assert_eq!(n, ms as u64 * NANOS_PER_MILLI, "wrong delay value")
            }
            TransactionKind::Delay(n) => {
                assert_eq!(n, ms as u64 * NANOS_PER_MILLI, "wrong delay value")
            }
            _ => panic!(
                "Wrong kind of delay. Expected Delay or AsyncDelay got {:?}",
                transaction.kind
            ),
        }

        if transaction.real_delay {
            thread::sleep(Duration::from_millis(ms as u64));
        }
    }
}

/// A `Delay` implementation that does not actually block.
pub struct NoopDelay;

impl NoopDelay {
    /// Create a new `NoopDelay` instance.
    pub fn new() -> Self {
        NoopDelay
    }
}

impl Default for NoopDelay {
    fn default() -> Self {
        Self::new()
    }
}

impl delay::DelayNs for NoopDelay {
    fn delay_ns(&mut self, _ns: u32) {
        // no-op
    }
}

#[cfg(feature = "embedded-hal-async")]
impl embedded_hal_async::delay::DelayNs for NoopDelay {
    async fn delay_ns(&mut self, _ns: u32) {
        // no-op
    }
}

/// A `Delay` implementation that uses `std::thread::sleep`.
pub struct StdSleep;

impl StdSleep {
    /// Create a new `StdSleep` instance.
    pub fn new() -> Self {
        StdSleep
    }
}

impl Default for StdSleep {
    fn default() -> Self {
        Self::new()
    }
}

impl delay::DelayNs for StdSleep {
    fn delay_ns(&mut self, ns: u32) {
        thread::sleep(Duration::from_nanos(ns as u64));
    }
}

#[cfg(feature = "embedded-hal-async")]
impl embedded_hal_async::delay::DelayNs for StdSleep {
    async fn delay_ns(&mut self, ns: u32) {
        thread::sleep(Duration::from_nanos(ns as u64));
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use super::*;

    #[test]
    fn test_noop_sleep() {
        use embedded_hal::delay::DelayNs;

        let mut delay = NoopDelay::new();
        let now = Instant::now();
        delay.delay_ms(1000);
        assert!(now.elapsed().as_millis() < 100);
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_noop_sleep_async() {
        use embedded_hal_async::delay::DelayNs;

        let mut delay = NoopDelay::new();
        let now = Instant::now();
        delay.delay_ms(1000).await;
        assert!(now.elapsed().as_millis() < 100);
    }

    #[test]
    fn test_std_sleep() {
        use embedded_hal::delay::DelayNs;

        let mut delay = StdSleep::new();
        let now = Instant::now();
        delay.delay_ms(1000);
        assert!(now.elapsed().as_millis() >= 1000);
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_std_sleep_async() {
        use embedded_hal_async::delay::DelayNs;

        let mut delay = StdSleep::new();
        let now = Instant::now();
        delay.delay_ms(1000).await;
        assert!(now.elapsed().as_millis() >= 1000);
    }

    #[test]
    fn test_checked_sleep() {
        use embedded_hal::delay::DelayNs;

        let transactions = vec![
            Transaction::delay_ns(1000),
            Transaction::delay_ns(2000),
            Transaction::delay_ns(3000),
        ];

        let mut delay = CheckedDelay::new(&transactions);
        let now = Instant::now();
        delay.delay_ns(1000);
        delay.delay_ns(2000);
        delay.delay_ns(3000);
        assert!(now.elapsed().as_millis() < 100);
        delay.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_checked_sleep_async() {
        use embedded_hal_async::delay::DelayNs;

        let transactions = vec![
            Transaction::async_delay_ns(1000),
            Transaction::async_delay_ns(2000),
            Transaction::async_delay_ns(3000),
            Transaction::blocking_delay_ns(4000),
            Transaction::delay_ns(5000),
            Transaction::delay_ns(6000),
        ];

        let mut delay = CheckedDelay::new(&transactions);
        let now = Instant::now();
        delay.delay_ns(1000).await;
        delay.delay_ns(2000).await;
        delay.delay_ns(3000).await;
        embedded_hal::delay::DelayNs::delay_ns(&mut delay, 4000);
        embedded_hal::delay::DelayNs::delay_ns(&mut delay, 5000);
        delay.delay_ns(6000).await;

        assert!(now.elapsed().as_millis() < 100);
        delay.done();
    }

    #[test]
    fn test_checked_sleep_conversions() {
        use embedded_hal::delay::DelayNs;

        let transactions = vec![
            Transaction::delay_ns(1000),
            Transaction::delay_us(1000),
            Transaction::delay_ms(1),
        ];

        let mut delay = CheckedDelay::new(&transactions);
        let now = Instant::now();
        delay.delay_us(1);
        delay.delay_ms(1);
        delay.delay_ns(1_000_000);
        assert!(now.elapsed().as_millis() < 100);
        delay.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_checked_sleep_conversions_async() {
        use embedded_hal_async::delay::DelayNs;

        let transactions = vec![
            Transaction::async_delay_ns(1000),
            Transaction::async_delay_us(1000),
            Transaction::async_delay_ms(1),
        ];

        let mut delay = CheckedDelay::new(&transactions);
        let now = Instant::now();
        delay.delay_us(1).await;
        delay.delay_ms(1).await;
        delay.delay_ns(1_000_000).await;
        assert!(now.elapsed().as_millis() < 100);
        delay.done();
    }

    #[test]
    fn test_checked_sleep_real_delay() {
        use embedded_hal::delay::DelayNs;

        let transactions = vec![
            Transaction::delay_ns(50_000_000).wait(),
            Transaction::delay_us(50_000).wait(),
            Transaction::delay_ms(50).wait(),
        ];

        let mut delay = CheckedDelay::new(&transactions);
        let now = Instant::now();
        delay.delay_ms(50);
        delay.delay_ms(50);
        delay.delay_ms(50);

        assert!(now.elapsed().as_millis() >= 150);
        assert!(now.elapsed().as_millis() < 1500);
        delay.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    async fn test_checked_sleep_real_delay_async() {
        use embedded_hal_async::delay::DelayNs;

        let transactions = vec![
            Transaction::async_delay_ns(50_000_000).wait(),
            Transaction::async_delay_us(50_000).wait(),
            Transaction::async_delay_ms(50).wait(),
        ];

        let mut delay = CheckedDelay::new(&transactions);
        let now = Instant::now();
        delay.delay_ms(50).await;
        delay.delay_ms(50).await;
        delay.delay_ms(50).await;

        assert!(now.elapsed().as_millis() >= 150);
        assert!(now.elapsed().as_millis() < 1500);
        delay.done();
    }

    #[test]
    fn test_checked_sleep_overflow() {
        use embedded_hal::delay::DelayNs;

        let transactions = vec![
            Transaction::delay_us(4295000),
            Transaction::delay_ms(4295),
            Transaction::delay_us(4295000 * 100),
            Transaction::delay_ms(4295 * 100),
        ];

        let mut delay = CheckedDelay::new(&transactions);
        let now = Instant::now();
        delay.delay_us(4295000);
        delay.delay_ms(4295);
        delay.delay_us(4295000 * 100);
        delay.delay_ms(4295 * 100);
        assert!(now.elapsed().as_millis() < 100);
        delay.done();
    }

    #[test]
    #[should_panic(expected = "wrong delay value")]
    fn test_checked_sleep_overflow_err() {
        use embedded_hal::delay::DelayNs;

        let transactions = vec![Transaction::delay_us(4295)];

        let mut delay = CheckedDelay::new(&transactions);
        let now = Instant::now();
        delay.delay_ms(4);
        assert!(now.elapsed().as_millis() < 100);
        delay.done();
    }

    #[tokio::test]
    #[cfg(feature = "embedded-hal-async")]
    #[should_panic(expected = "wrong delay value")]
    async fn test_checked_sleep_overflow_async_err() {
        use embedded_hal_async::delay::DelayNs;

        let transactions = vec![Transaction::async_delay_us(4295)];

        let mut delay = CheckedDelay::new(&transactions);
        let now = Instant::now();
        delay.delay_ms(4).await;
        assert!(now.elapsed().as_millis() < 100);
        delay.done();
    }
}
