use core::fmt::Debug;
use eh1::{
    digital::{OutputPin, InputPin, ErrorType},
    spi::{SpiDevice, Operation},
    delay
};
use crate::eh1::pin::{
    Transaction as PinTransaction,
    State as PinState,
    TransactionKind
};
use crate::eh1::spi::{
    Mode,
    Transaction as SpiTransaction,
};

impl ErrorType for HalDigital {
    type Error = super::error::MockError;
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expectation {
    Digital(u8, PinTransaction),
    Delay(u64),
    Spi(SpiTransaction<u8>)
}

pub type Hal = super::super::common::Generic<Expectation>;
use std::sync::{Arc, Mutex};

pub struct HalDigital(pub Arc<Mutex<Hal>>, u8);
pub struct HalDelay(pub Arc<Mutex<Hal>>);
pub struct HalSpi(pub Arc<Mutex<Hal>>);

impl delay::DelayNs for HalDelay {
    fn delay_ns(&mut self, ns: u32) {
        match self.0.lock().unwrap().next().expect("no expectation for delay call") {
            Expectation::Delay(expected_ns) => {
                assert_eq!(
                    expected_ns,
                    ns.into(),
                    "wrong timing"
                );
            },
            Expectation::Spi(_) => panic!("wrong peripheral type: spi instead of delay"),
            Expectation::Digital(_, _) => panic!("wrong peripheral type: digital instead of delay")
        }
    }
}

impl Hal {
    pub fn pin(self, id: u8) -> HalDigital {
        HalDigital(Arc::new(Mutex::new(self)), id)
    }

    pub fn delay(self) -> HalDelay {
        HalDelay(Arc::new(Mutex::new(self)))
    }

    pub fn spi(self) -> HalSpi {
        HalSpi(Arc::new(Mutex::new(self)))
    }
}


//base implementation holds an option to an overall hal, which iterator it takes from depends on
//the option?

// Generic iterator is peekable.  peek to do top-level stuff, but then next() to do rest?

// how do either of these supporting existing API 
impl OutputPin for HalDigital {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        match self.0.lock().unwrap().next().expect("no expectation for pin::set_low call") {
            Expectation::Digital(id, PinTransaction { kind, err }) => {
                assert_eq!(
                    id,
                    self.1,
                    "wrong pin"
                );

                assert_eq!(
                    kind,
                    TransactionKind::Set(PinState::Low),
                    "expected pin::set_low"
                );

                match err {
                    Some(e) => Err(e),
                    None => Ok(()),
                }
            },
            _ => panic!("wrong peripheral type")
        }
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        match self.0.lock().unwrap().next().expect("no expectation for pin::set_high call") {
            Expectation::Digital(id, PinTransaction { kind, err }) => {
                assert_eq!(
                    id,
                    self.1,
                    "wrong pin"
                );

                assert_eq!(
                    kind,
                    TransactionKind::Set(PinState::High),
                    "expected pin::set_high"
                );

                match err {
                    Some(e) => Err(e),
                    None => Ok(()),
                }
            },
            _ => panic!("wrong peripheral type")
        }
    }
}

impl InputPin for HalDigital {
    fn is_high(&mut self) -> Result<bool, <Self as eh1::digital::ErrorType>::Error> { 
        match self.0.lock().unwrap().next().expect("no expectation for pin::is_high call") {
            Expectation::Digital(id, PinTransaction { kind, err }) => {
                assert_eq!(
                    id,
                    self.1,
                    "wrong pin"
                );

                assert!(kind.is_get(), "expected pin::get");

                if let Some(e) = err {
                    Err(e)
                } else if let TransactionKind::Get(v) = kind {
                    Ok(v == PinState::High)
                } else {
                    unreachable!();
                }
            },
            _ => panic!("wrong peripheral type")
        }
    }

    fn is_low(&mut self) -> Result<bool, <Self as eh1::digital::ErrorType>::Error> {
        match self.0.lock().unwrap().next().expect("no expectation for pin::is_low call") {
            Expectation::Digital(id, PinTransaction { kind, err }) => {
                assert_eq!(
                    id,
                    self.1,
                    "wrong pin"
                );

                assert!(kind.is_get(), "expected pin::get");

                if let Some(e) = err {
                    Err(e)
                } else if let TransactionKind::Get(v) = kind {
                    Ok(v == PinState::Low)
                } else {
                    unreachable!();
                }
            },
            _ => panic!("wrong peripheral type")
        }
    }
}

impl eh1::spi::ErrorType for HalSpi {
    type Error = eh1::spi::ErrorKind;
}

impl SpiDevice for HalSpi {
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        let mut iterator = self.0.lock().unwrap();
        let transaction_call = iterator.next().expect("no expectation for spi::transaction call");
        match transaction_call {
            Expectation::Spi(w) => {
                assert_eq!(
                    w.expected_mode,
                    Mode::TransactionStart,
                    "spi::transaction unexpected mode"
                );

                for op in operations {
                    match op {
                        Operation::Read(buffer) => {
                            let read_call = iterator.next().expect("no expectation for spi::read call");
                            match read_call {
                                Expectation::Spi(w) => {
                                    assert_eq!(w.expected_mode, Mode::Read, "spi::read unexpected mode");
                                    assert_eq!(
                                        buffer.len(),
                                        w.response.len(),
                                        "spi:read mismatched response length"
                                    );
                                    buffer.copy_from_slice(&w.response);
                                },
                                _ => panic!("wrong type")
                            }
                        },
                        Operation::Write(buffer) => {
                            let write_call = iterator.next().expect("no expectation for spi::write call");
                            match write_call {
                                Expectation::Spi(w) => {
                                    assert_eq!(w.expected_mode, Mode::Write, "spi::write unexpected mode");
                                    assert_eq!(
                                        &w.expected_data, buffer,
                                        "spi::write data does not match expectation"
                                    );
                                },
                                _ => panic!("wrong type")
                            }
                        },
                        Operation::Transfer(read, write) => {
                            match iterator.next().expect("no expectation for spi::transfer call") {
                                Expectation::Spi(w) => {
                                    assert_eq!(
                                        w.expected_mode,
                                        Mode::Transfer,
                                        "spi::transfer unexpected mode"
                                    );
                                    assert_eq!(
                                        &w.expected_data, write,
                                        "spi::write data does not match expectation"
                                    );
                                    assert_eq!(
                                        read.len(),
                                        w.response.len(),
                                        "mismatched response length for spi::transfer"
                                    );
                                    read.copy_from_slice(&w.response);
                                },
                                _ => panic!("wrong type")
                            }
                        }
                        Operation::TransferInPlace(buffer) => {
                            match iterator.next().expect("no expectation for spi::transfer_in_place call") {
                                Expectation::Spi(w) => {
                                    assert_eq!(
                                        w.expected_mode,
                                        Mode::TransferInplace,
                                        "spi::transfer_in_place unexpected mode"
                                    );
                                    assert_eq!(
                                        &w.expected_data, buffer,
                                        "spi::transfer_in_place write data does not match expectation"
                                    );
                                    assert_eq!(
                                        buffer.len(),
                                        w.response.len(),
                                        "mismatched response length for spi::transfer_in_place"
                                    );
                                    buffer.copy_from_slice(&w.response);
                                },
                                _ => panic!("wrong type")
                            }
                        }
                        Operation::DelayNs(delay) => {
                            match iterator.next().expect("no expectation for spi::delay call") {
                                Expectation::Spi(w) => {
                                    assert_eq!(
                                        w.expected_mode,
                                        Mode::Delay(*delay),
                                        "spi::transaction unexpected mode"
                                    );
                                },
                                _ => panic!("wrong expectation type")
                            };
                        }
                    }

                }
            },
            _ => panic!("wrong expectation type")
        }

        let transaction_call = iterator.next().expect("no expectation for spi::transaction call");
        match transaction_call {
            Expectation::Spi(w) => {
                assert_eq!(
                    w.expected_mode,
                    Mode::TransactionEnd,
                    "spi::transaction unexpected mode"
                )
            },
            _ => panic!("wrong expectation type")
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use eh1::delay::DelayNs;

    #[test]
    fn test_hal() {
        let hal = Hal::new(
            &vec![
                Expectation::Digital(0, PinTransaction::set(PinState::High)),
                Expectation::Digital(1, PinTransaction::set(PinState::High)),
                Expectation::Delay(10),
                Expectation::Digital(2, PinTransaction::set(PinState::Low)),
                Expectation::Digital(3, PinTransaction::set(PinState::High)),
            ]
        );

        let mut zero = hal.clone().pin(0);
        let mut one = hal.clone().pin(1);
        let mut delay = hal.clone().delay();
        let mut two = hal.clone().pin(2);
        let mut three = hal.clone().pin(3);

        zero.set_high().unwrap();
        one.set_high().unwrap();
        delay.delay_ns(10);
        two.set_low().unwrap();
        three.set_high().unwrap();

        hal.clone().done();
    }
}
