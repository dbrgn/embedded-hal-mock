//! This is a collection of types that implement the embedded-hal version 0.x
//! traits.
//!
//! ## Usage
//!
//! See module-level docs for more information.

mod error;
pub use error::MockError;

pub mod adc;
pub mod delay;
pub mod digital;
pub mod i2c;
pub mod serial;
pub mod spi;
#[cfg(feature = "embedded-time")]
pub mod timer;
