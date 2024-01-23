use core::fmt::Debug;
use crate::eh1::pin::Transaction as PinTransaction;
use crate::eh1::spi::Transaction as SpiTransaction;
use crate::eh1::delay::Transaction as DelayTransaction;

#[derive(Debug, PartialEq, Clone)]
pub enum Expectation {
    Digital(PinTransaction),
    Delay(DelayTransaction),
    Spi(SpiTransaction<u8>)
}

pub type Hal = super::super::common::Generic<Expectation>;
use std::sync::{Arc, Mutex};

impl Hal {
    pub fn pin(self) -> crate::eh1::pin::Mock {
        crate::eh1::pin::Mock::with_hal(
            &[],
            Arc::new(
                Mutex::new(self)
            )
        )
    }

    pub fn delay(self) -> crate::eh1::delay::Mock {
        crate::eh1::delay::Mock::with_hal(
            &[],
            Arc::new(
                Mutex::new(self)
            )
        )
    }

    pub fn spi(self) -> crate::eh1::spi::Mock<u8> {
        crate::eh1::spi::Mock::with_hal(
            &[],
            Arc::new(
                Mutex::new(self)
            )
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use eh1::delay::DelayNs;
    use crate::eh1::pin::State;
    use eh1::{
        digital::OutputPin,
        spi::SpiDevice,
    };

    #[test]
    fn test_hal() {
        let mut hal = Hal::new(&vec![]);

        let mut zero = hal.clone().pin();
        let mut one = hal.clone().pin();
        let mut delay = hal.clone().delay();
        let mut two = hal.clone().pin();
        let mut three = hal.clone().pin();
        let mut spi = hal.clone().spi();

        hal.update_expectations(&vec![
            Expectation::Digital(PinTransaction::set(State::High)),
            Expectation::Digital(PinTransaction::set(State::High)),
            Expectation::Delay(10),
            Expectation::Digital(PinTransaction::set(State::Low)),
            Expectation::Digital(PinTransaction::set(State::High)),
            Expectation::Spi(SpiTransaction::transaction_start()),
            Expectation::Spi(SpiTransaction::write(0x05)),
            Expectation::Spi(SpiTransaction::transaction_end()),
        ]);

        zero.set_high().unwrap();
        one.set_high().unwrap();
        delay.delay_ns(10);
        two.set_low().unwrap();
        three.set_high().unwrap();
        spi.write(&[0x05]).unwrap();

        hal.done();
    }
}
