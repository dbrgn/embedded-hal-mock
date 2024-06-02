//! This is a collection of types that implement the embedded-hal traits.
//!
//! The implementations never access real hardware. Instead, the hardware is
//! mocked or no-op implementations are used.
//!
//! The goal of the crate is to be able to test drivers in CI without having
//! access to hardware.
//!
//! ## Usage
//!
//! The general approach for testing drivers using mocks:
//!
//! 1. Define the expectations: A list of transactions (e.g. a read or write
//!    operation) that you expect the driver-under-test to invoke on the mocked
//!    hardware
//! 2. Instantiate the mock with the expectations
//! 3. Run the test code
//! 4. At the end of the test code, call the `.done()` method on the mock to
//!    ensure that all expectations were met
//!
//! For more information, see module-level docs.
//!
//! **Note:** Mocks contain an `Arc` internally and can be cloned freely. This
//! means you can clone a mock before passing it to the driver, and then call
//! `.done()` on the second mock instance without having to reclaim the first
//! instance from the driver.
//!
//! ## embedded_hal Version Support
//!
//! This crate supports both version 0.x and version 1.x of embedded-hal.  By
//! default only support for version 0.x is enabled.  To enable support for
//! version 1.x, use the `eh1` feature.
//!
//! ## Cargo Features
//!
//! There are currently the following cargo features:
//!
//! - `eh0`: Provide module [`eh0`] that mocks embedded-hal version 0.x
//!   (enabled by default)
//! - `eh1`: Provide module [`eh1`] that mocks embedded-hal version 1.x
//!   (enabled by default)
//! - `embedded-time`: Enable the [`eh0::timer`] module (enabled by default)
//! - `embedded-hal-async`: Provide mocks for embedded-hal-async in [`eh1`]
#![cfg_attr(docsrs, feature(doc_cfg), feature(doc_auto_cfg))]
#![deny(missing_docs)]

pub mod common;
#[cfg(feature = "eh0")]
pub mod eh0;
#[cfg(feature = "eh1")]
pub mod eh1;
