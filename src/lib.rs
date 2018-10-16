//! This is a collection of types that implement the embedded-hal traits.
//!
//! The implementations never access real hardware. Instead, the hardware is mocked
//! or no-op implementations are used.
//!
//! The goal of the crate is to be able to test drivers in CI without having access
//! to hardware.
//!
//! ## Usage
//!
//! See module-level docs for more information.
//!
//! ## no\_std
//!
//! Currently this crate is not `no_std`. If you think this is important, let
//! me know.

extern crate embedded_hal as hal;

mod error;
pub use error::MockError;

pub mod mock;
pub mod i2c;
pub mod spi;
pub mod delay;

