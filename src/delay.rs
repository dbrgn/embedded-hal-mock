//! Delay mock implementations.
//!
//! ## Usage
//!
//! Just create an instance of [`DelayMockNoop`](struct.DelayMockNoop.html).
//! There will be no actual delay. This is useful for fast tests, where you
//! don't actually need to wait for the hardware.

use hal::blocking::delay;

/// A `Delay` implementation that does not actually block.
pub struct DelayMockNoop;

macro_rules! impl_delay_us {
    ($type:ty) => {
        impl delay::DelayUs<$type> for DelayMockNoop {
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
        impl delay::DelayMs<$type> for DelayMockNoop {
            /// A no-op delay implementation.
            fn delay_ms(&mut self, _n: $type) {}
        }
    };
}

impl_delay_ms!(u8);
impl_delay_ms!(u16);
impl_delay_ms!(u32);
impl_delay_ms!(u64);
