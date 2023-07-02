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
#![cfg_attr(
    feature = "embedded-hal-async",
    feature(async_fn_in_trait),
    allow(incomplete_features)
)]
#![deny(missing_docs)]

mod error;
pub use crate::error::MockError;

pub mod common;
pub mod delay;
pub mod i2c;
pub mod pin;
pub mod pwm;
pub mod serial;
pub mod spi;
