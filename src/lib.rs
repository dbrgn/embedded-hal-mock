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

#![deny(missing_docs)]

mod error;
pub use crate::error::MockError;

pub mod adc;
pub mod common;
pub mod delay;
pub mod i2c;
pub mod pin;
pub mod serial;
pub mod spi;
