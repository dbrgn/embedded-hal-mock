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
//! ## embedded_hal version
//!
//! This crate supports both version 0.x and version 1.x of embedded-hal.  By default only support
//! for version 0.x is enabled.  To enable support for version 1.x, use the `eh1` feature.
//!
//! ## Cargo Features
//!
//! There are currently the following cargo features:
//!
//! - `eh0`: Provide module [`eh0`] that mocks embedded-hal version 0.x (enabled by default)
//! - `eh1`: Provide module [`eh1`] that mocks embedded-hal version 1.x
//! - `embedded-time`: Enable the [`eh0::timer`] module (enabled by default)
//! - `embedded-hal-async`: Provide mocks for embedded-hal-async in [`eh1`]
//!
//! ## no\_std
//!
//! Currently this crate is not `no_std`. If you think this is important, let
//! me know.
#![cfg_attr(docsrs, feature(doc_cfg), feature(doc_auto_cfg))]
#![deny(missing_docs)]

pub mod common;
#[cfg(feature = "eh0")]
pub mod eh0;
#[cfg(feature = "eh1")]
pub mod eh1;
