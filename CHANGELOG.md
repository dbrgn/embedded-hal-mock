# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).


## Unreleased

### Added

- Support mocked SPI transaction errors (#131)

### Fixed

### Changed

- Drop fixed MSRV policy (#124)
- **Breaking**: the `eh0` feature is no longer part of the default features.
  it still exists as an optional feature and has to be explicitly added when needed.


## 0.11.1 - 2024-06-02

This release only includes a documentation fix (with regards to default-enabled
Cargo features), but no code changes.


## 0.11.0 - 2024-05-30

This release adds support for various async APIs as defined in
`embedded-hal-async`. To use these features, you need to enable the
`embedded-hal-async` Cargo feature.

If you're upgrading from an earlier version, please note that the `pin` module
was renamed to `digital` to match `embedded-hal`.

### Added

- Add `ToggleableOutputPin` support for `eh0::digital::Mock` (#89)
- Add `StatefulOutputPin` support for `eh1::digital::Mock` (#89)
- Async support for `eh1::i2c::Mock` (#119)
- Async support for `eh1::digital::Mock` (#91)
- Async delay for `NoopDelay` / `StdSleep` (#104)
- New `CheckedDelay` mock that supports both sync and async delays (#104)

### Changed

- Rename `pin` module to `digital` to match embedded-hal (#113)
- Improve top level usage docs (#117)


## 0.10.0 - 2024-01-10

This release contains a big change: `embedded-hal-mock` now supports both
`embedded-hal` 0.x and 1.x! The two variants are accessible through
`embedded_hal_mock::eh0::*` and `embedded_hal_mock::eh1::*`. If there are any
issues, leave feedback in the GitHub issue tracker.

Additionally, tests now fail if you forgot to call `.done()`. This should
reduce the chance of accidentally writing a broken test.

This release contains commits by 12 different people, thanks a lot for the
contributions!

### Migrating to 0.10.0

- Update your imports: Change `use embedded_hal_mock::*` to
  `use embedded_hal_mock::eh0::*`
- Rename all `.expect(...)` calls on mocks to `.update_expectations(...)`
- Rename all `eh0::delay::MockNoop` usages to `eh0::delay::NoopDelay`
- Run your tests to ensure that you don't have any missing `.done()` calls in
  your code
- Look through the rest of the changes below and check if they might affect
  your code

### Added

- Support for both `embedded-hal` 0.x and 1.x in the same crate (#75)
- Print a warning to stderr and fail test if a mock is dropped without having
  calling `.done()` on it, or if `.done()` is called twice (#59, #61)
- Implement mock for `eh1::pwm::SetDutyCycle`

### Fixed

- `Generic` mock: Fix a bug that caused the call to `.done()` to fail if
  `.next()` was called on the mock after all expectations have already been
  consumed (#58)
- Fix assertion error message for SPI `transfer` and ` transfer_in_place` (#90)

### Changed

- Renamed `.expect(...)` method to `.update_expectations(...)` to avoid
  confusion with the expect method in `Option` and `Result` (#63)
- When updating expectations on a mock by calling `.expect(...)` /
  `.update_expectations(...)` on it, assert that previous expectations have
  been consumed (#63)
- Rename `delay::MockNoop` to `delay::NoopDelay`.
- Changed the eh1 SPI implementation to be generic over word size
- Updated `nb` dependency from 0.1 to 1.1 (#107)
- Bump minimal supported Rust version (MSRV) to 1.63 (or 1.75 if you use
  embedded-hal 1.0)
- The minimal supported Rust version (MSRV) is specified in the `Cargo.toml` to
  offer clearer error messages to consumers with outdated Rust versions


## 0.9.0 - 2023-01-07

### Added

- Implement `WriteIter` and `WriteIterRead` for i2c mock (#44)
- Implement `PwmPin` for pin mock (#52)
- Add mock for timers using embedded-time with nanosecond precision (#40)

### Changed

- Bump minimal supported Rust version (MSRV) to 1.60
- Switch to Rust 2021 edition (#55)
- Switch from CircleCI to GitHub Actions (#50)


## 0.8.0 - 2021-08-16

### Added

- Add one-shot ADC mock (#38)


## 0.7.2 - 2020-06-02

### Added

- Implement `std::Error` trait for `MockError` (#31)
- serial: Implement error expectations (#32)


## 0.7.1 - 2020-01-03

### Added

- i2c: Implement error expectations (#29)

### Fixed

- Fix link to digital pin docs (#28)


## 0.7.0 - 2019-05-22

### Added

- The serial transaction API now has two new constructor methods: `read_many`
  and `write_many`.

### Changed

- The serial transaction API changed: The `Transaction::write` function now
  expects a single word, not a collection of words. To add a transaction for
  many writes, use `Transaction::write_many` instead.

### Fixed

- Make the serial mock actually cloneable


## 0.6.0 - 2019-05-10

### Added

- Add serial device mock (#21)
- Add InputPin and OutputPin mocks (#18)

### Changed

- `MockError::Io` now wraps an `io::ErrorKind` instance instead of `io::Error`.


## 0.5.0 - 2019-01-07

### Added

- SPI: Add support for non-blocking `FullDuplex` mode (#14)

### Changed

- Require Rust 1.31+
- Apply and enforce rustfmt


## 0.4.1 - 2018-12-26

### Added

- Add `StdSleep` delay implementation based on `std::thread::sleep` (#8)
- Add `new()` methods to `MockNoop` and `StdSleep`

### Fixed

- Fix error messages for unfulfilled I²C expectations (#12)


## 0.4.0 - 2018-10-22

### Changed

- I²C mock has a new transaction based API, matching the SPI mock (#4)


## 0.3.0 - 2018-10-12

### Added

- SPI mock implementation (#2)
- Set up CI (#3)

### Changed

- Restructure crate:
  - `I2cMock` is now at `i2c::Mock`
  - `DelayMockNoop` is now at `delay::MockNoop`
- Move all docs into crate docs (so it can be tested)


## 0.2.0 - 2018-06-18

### Changed

- Upgrade to `embedded-hal` 0.2.


## 0.1.0 - 2018-03-31

Initial release on crates.io.
