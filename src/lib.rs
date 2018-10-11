extern crate embedded_hal as hal;

mod error;
pub use error::MockError;

mod i2c;
pub use i2c::I2cMock;

mod spi;
pub use spi::SPIMock;

pub struct DelayMockNoop;

macro_rules! impl_delay_us {
    ($type:ty) => {
        impl hal::blocking::delay::DelayUs<$type> for DelayMockNoop {
            /// A no-op delay implementation.
            fn delay_us(&mut self, _n: $type) {}
        }
    };
}

impl_delay_us!(u8);
impl_delay_us!(u16);
impl_delay_us!(u32);
impl_delay_us!(u64);

macro_rules! impl_delay_ms {
    ($type:ty) => {
        impl hal::blocking::delay::DelayMs<$type> for DelayMockNoop {
            /// A no-op delay implementation.
            fn delay_ms(&mut self, _n: $type) {}
        }
    };
}

impl_delay_ms!(u8);
impl_delay_ms!(u16);
impl_delay_ms!(u32);
impl_delay_ms!(u64);
