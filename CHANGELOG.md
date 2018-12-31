# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).


## 0.4.0 - 2018-12-26

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
