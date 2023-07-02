# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Implement mock for `embedded_hal::pwm::SetDutyCycle`

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

-  Fix link to digital pin docs (#28)


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
