//! This is a collection of types that implement the embedded-hal version 1.x
//! traits.
//!
//! ## Usage
//!
//! See module-level docs for more information.

mod error;
pub use crate::eh1::error::MockError;

pub mod delay;
pub mod digital;
pub mod i2c;
pub mod io;
pub mod pwm;
pub mod serial;
pub mod spi;
