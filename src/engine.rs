//! Mock Engine Implementation
//! 
//! This allows expectations to be created over multiple peripherals
//! to support testing of more complex driver behaviors.
//! 
//! ```
//! use embedded_hal::digital::v2::{OutputPin as _, InputPin as _};
//! use embedded_hal::blocking::spi::Write as _;
//! use embedded_hal::blocking::i2c::Write as _;
//! use embedded_hal::blocking::delay::DelayMs as _;
//! 
//! use embedded_hal_mock::engine::*;
//! 
//! let mut engine = Engine::new();
//! let mut spi1 = engine.spi();
//! let mut spi2 = engine.spi();
//! let mut i2c1 = engine.i2c();
//! let mut pin1 = engine.pin();
//! let mut delay1 = engine.delay();
//!
//! pin1.expect(PinTransaction::set(PinState::High));
//! spi1.inner().expect(SpiTransaction::write(vec![1, 2]));
//! spi2.inner().expect(SpiTransaction::write(vec![3, 4]));
//! i2c1.inner().expect(I2cTransaction::write(0xaa, vec![5, 6]));
//! delay1.expect(100);
//!
//! pin1.set_high();
//! spi1.write(&vec![1, 2]);
//! spi2.write(&vec![3, 4]);
//! i2c1.write(0xaa, &vec![5, 6]);
//! delay1.delay_ms(100);
//!
//! engine.done();
//! ```
//! 

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::fmt::Debug;

// Re-exports for user convenience
pub use crate::spi::{Transaction as SpiTransaction, Mock as SpiMock};
pub use crate::i2c::{Transaction as I2cTransaction, Mock as I2cMock};
pub use crate::pin::{Transaction as PinTransaction, Mock as PinMock, State as PinState};

// Unfortunately pin must be reimplemented due to &self InputPin methods
use embedded_hal::digital::v2::{InputPin, OutputPin};
use crate::pin::{TransactionKind as PinTransactionKind};
use crate::error::MockError;

/// Transactions supported by the engine
#[derive(Debug, PartialEq, Clone)]
pub enum Transaction {
    /// Spi transactions
    Spi(SpiTransaction),
    /// I2c transactions
    I2c(I2cTransaction),
    /// Pin transactions
    Pin(PinTransaction),
    /// DelayMs transactions
    DelayMs(u32),
}

pub struct Engine {
    peripheral_count: u32,
    expected: Arc<Mutex<VecDeque<(u32, Transaction)>>>,
}

impl Engine {

    /// Create a new engine instance with no expectations loaded
    pub fn new() -> Self {
        Engine{peripheral_count: 0, expected: Arc::new(Mutex::new(VecDeque::new()))}
    }

    /// Create an SPI peripheral
    pub fn spi(&mut self) -> SpiMock<Peripheral<Spi>> {
        let p = Peripheral::new(self.peripheral_count, self.expected.clone());
        self.peripheral_count += 1;
        p.into()
    }

    /// Create an I2C peripheral
    pub fn i2c(&mut self) -> I2cMock<Peripheral<I2c>> {
        let p = Peripheral::new(self.peripheral_count, self.expected.clone());
        self.peripheral_count += 1;
        p.into()
    }

    /// Create a new Pin
    pub fn pin(&mut self) -> Peripheral<Pin> {
        let p = Peripheral::new(self.peripheral_count, self.expected.clone());
        self.peripheral_count += 1;
        p
    }

    /// Create a new Delay
    pub fn delay(&mut self) -> Peripheral<Delay> {
        let p = Peripheral::new(self.peripheral_count, self.expected.clone());
        self.peripheral_count += 1;
        p
    }

    /// Push an expectation to the internal buffer
    /// This provides an alternative mechanism to peripheral.expect();
    pub fn expect<T>(&mut self, p: &Peripheral<T>, t: Transaction) {
        let mut expected = self.expected.lock().unwrap();
        expected.push_back((p.id, t));
    }

    /// Finalize expectations
    /// This checks for any remaining expectations and drains the internal buffer
    pub fn done(&mut self) {
        let remaining: Vec<_> = self.expected.lock().unwrap().drain(..).collect();
        assert_eq!(remaining, vec![]);
    }
}

pub struct Peripheral<Type> {
    id: u32,
    expected: Arc<Mutex<VecDeque<(u32, Transaction)>>>,
    _type: PhantomData<Type>,
}

impl <Type>Peripheral<Type> {
    fn new(id: u32, expected: Arc<Mutex<VecDeque<(u32, Transaction)>>>) -> Self {
        Self{id, expected, _type: PhantomData }
    }

    fn pop_front(&mut self) -> Option<(u32, Transaction)> {
        let mut e = self.expected.lock().unwrap();
        e.pop_front()
    }

    fn push_back(&mut self, t: Transaction) {
        let mut e = self.expected.lock().unwrap();
        e.push_back((self.id, t));
    }
}

/// Empty struct for type-state programming
pub struct Spi {}

impl Iterator for Peripheral<Spi>
{
    type Item = SpiTransaction;
    fn next(&mut self) -> Option<Self::Item> {
        let (id, transaction) = self.pop_front().expect("no transaction found");
        assert_eq!(id, self.id);

        match transaction {
            Transaction::Spi(t) => Some(t),
            _ => {
                assert!(true, "expected SPI transaction, found: {:?}", transaction);
                None
            },
        }     
    }
}

impl Peripheral<Spi> {
    pub fn expect(&mut self, t: SpiTransaction) {
        self.push_back(Transaction::Spi(t))
    }
}

/// Empty struct for type-state programming
pub struct I2c {}

impl Iterator for Peripheral<I2c>
{
    type Item = I2cTransaction;
    fn next(&mut self) -> Option<Self::Item> {
        let (id, transaction) = self.pop_front().expect("no transaction found");
        assert_eq!(id, self.id);

        match transaction {
            Transaction::I2c(t) => Some(t),
            _ => {
                assert!(true, "expected I2C transaction, found: {:?}", transaction);
                None
            },
        }     
    }
}

impl Peripheral<I2c> {
    pub fn expect(&mut self, t: I2cTransaction) {
        self.push_back(Transaction::I2c(t))
    }
}


/// Empty struct for type-state programming
pub struct Pin {}

impl Peripheral<Pin>
{
    fn next(&self) -> Option<PinTransaction> {
        let mut e = self.expected.lock().unwrap();
        let (id, transaction) = e.pop_front().expect("no transaction found");
        assert_eq!(id, self.id);

        match transaction {
            Transaction::Pin(t) => Some(t),
            _ => {
                assert!(true, "expected Pin transaction, found: {:?}", transaction);
                None
            },
        }
    }

    pub fn expect(&mut self, t: PinTransaction) {
        self.push_back(Transaction::Pin(t))
    }
}

/// Single digital push-pull output pin
impl OutputPin for Peripheral<Pin> {
    /// Error type
    type Error = MockError;

    /// Drives the pin low
    fn set_low(&mut self) -> Result<(), Self::Error> {
        let PinTransaction{kind, err} = self.next().expect("no expectation for pin::set_low call");

        assert_eq!(kind, PinTransactionKind::Set(PinState::Low), "expected pin::set_low");
        
        match err {
            Some(e) => Err(e.clone()),
            None => Ok(()),
        }
    }

    /// Drives the pin high
    fn set_high(&mut self) -> Result<(), Self::Error> {
        let PinTransaction{kind, err} = self.next().expect("no expectation for pin::set_high call");

        assert_eq!(kind, PinTransactionKind::Set(PinState::High), "expected pin::set_high");
        
        match err {
            Some(e) => Err(e.clone()),
            None => Ok(()),
        }
    }
}

impl InputPin for Peripheral<Pin> {
    /// Error type
    type Error = MockError;

    /// Is the input pin high?
    fn is_high(&self) -> Result<bool, Self::Error> {
        let mut s = self.clone();

        let PinTransaction{kind, err} = s.next().expect("no expectation for pin::is_high call");

        assert_eq!(kind.is_get(), true, "expected pin::get");

        if let Some(e) = err { 
            Err(e.clone())
        } else if let PinTransactionKind::Get(v) = kind {
            Ok(v == PinState::High)
        } else {
            unreachable!();
        }
    }

    /// Is the input pin low?
    fn is_low(&self) -> Result<bool, Self::Error> {
        let mut s = self.clone();

        let PinTransaction{kind, err} = s.next().expect("no expectation for pin::is_low call");

        assert_eq!(kind.is_get(), true, "expected pin::get");

        if let Some(e) = err { 
            Err(e.clone())
        } else if let PinTransactionKind::Get(v) = kind {
            Ok(v == PinState::Low)
        } else {
            unreachable!();
        }
    }
}

/// Empty struct for type-state programming
pub struct Delay {}

impl Iterator for Peripheral<Delay>
{
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        let (id, transaction) = self.pop_front().expect("no transaction found");
        assert_eq!(id, self.id);

        match transaction {
            Transaction::DelayMs(t) => Some(t),
            _ => {
                assert!(true, "expected Delay transaction, found: {:?}", transaction);
                None
            },
        }     
    }
}

impl Peripheral<Delay> {
    pub fn expect(&mut self, t: u32) {
        self.push_back(Transaction::DelayMs(t))
    }
}

use embedded_hal::blocking::delay;

impl delay::DelayMs<u32> for Peripheral<Delay> {
    fn delay_ms(&mut self, v: u32) {
        let e = self.next().unwrap();
        assert_eq!(v, e);
    }
}


#[cfg(test)]
mod test {
    use super::*;

    use std::io::ErrorKind;

    use embedded_hal::digital::v2::{OutputPin as _, InputPin as _};
    use embedded_hal::blocking::spi::Write as _;
    use embedded_hal::blocking::i2c::Write as _;
    use embedded_hal::blocking::delay::DelayMs as _;

    #[test]
    fn test_engine() {
        let mut engine = Engine::new();
        let mut spi1 = engine.spi();
        let mut spi2 = engine.spi();
        let mut i2c1 = engine.i2c();
        let mut pin1 = engine.pin();
        let mut delay1 = engine.delay();

        pin1.expect(PinTransaction::set(PinState::High));
        spi1.inner().expect(SpiTransaction::write(vec![1, 2]));
        spi2.inner().expect(SpiTransaction::write(vec![3, 4]));
        i2c1.inner().expect(I2cTransaction::write(0xaa, vec![5, 6]));
        delay1.expect(100);

        pin1.set_high();
        spi1.write(&vec![1, 2]);
        spi2.write(&vec![3, 4]);
        i2c1.write(0xaa, &vec![5, 6]);
        delay1.delay_ms(100);

        engine.done();
    }

    #[test]
    #[should_panic]
    fn wrong_peripheral_type() {
        let mut engine = Engine::new();
        let mut spi1 = engine.spi();
        let mut i2c1 = engine.i2c();

        spi1.inner().expect(SpiTransaction::write(vec![1, 2]));
        i2c1.inner().expect(I2cTransaction::write(0xaa, vec![5, 6]));

        i2c1.write(0xaa, &vec![5, 6]);
        spi1.write(&vec![1, 2]);

        engine.done();
    }

    #[test]
    #[should_panic]
    fn wrong_peripheral_id() {
        let mut engine = Engine::new();
        let mut spi1 = engine.spi();
        let mut spi2 = engine.spi();


        spi1.inner().expect(SpiTransaction::write(vec![1, 2]));
        spi2.inner().expect(SpiTransaction::write(vec![3, 4]));

        spi2.write(&vec![3, 4]);
        spi1.write(&vec![1, 2]);

        engine.done();
    }
}