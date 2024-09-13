# embedded-hal-mock

[![GitHub Actions][github-actions-badge]][github-actions]
[![Crates.io Version][version-badge]][crates-io]

This is a collection of types that implement the embedded-hal traits.

The implementations never access real hardware. Instead, the hardware is mocked
or no-op implementations are used.

The goal of the crate is to be able to test drivers in CI without having access
to hardware.

[Docs](https://docs.rs/embedded-hal-mock/)

## Usage

See module-level docs for more information.

## embedded_hal version

This crate supports both version 0.x and version 1.x of embedded-hal.  By default only support
for version 0.x is enabled.  To enable support for version 1.x, use the `eh1` feature.

## Cargo Features

There are currently the following cargo features:

- `eh0`: Provide module `eh0` that mocks embedded-hal version 0.x (enabled by default)
- `eh1`: Provide module `eh1` that mocks embedded-hal version 1.x (enabled by default)
- `embedded-time`: Enable the `eh0::timer` module (enabled by default)
- `embedded-hal-async`: Provide mocks for embedded-hal-async in `eh1`

## no\_std

Currently this crate is not `no_std`. If you think this is important, let
me know.

## Status

| Feature                                     | embedded-hal | embeded-hal-async |
|---------------------------------------------|--------------|-------------------|
| I²C                                         | ✅           | ✅               |
| SPI                                         | ✅           | ✅               |
| No-op delay                                 | ✅           | ✅               |
| Actual delay                                | ✅           | ✅               |
| Serial                                      | ✅           | -                |
| RNG                                         | -            | -                |
| I/O pins (including PWM)                    | ✅           | ✅               |
| ADC                                         | ✅           | -                |
| Timers (with `embedded-time` Cargo feature) | ✅           | -                |

Pull requests for more mock implementations are welcome! :)

## Minimum Supported Rust Version (MSRV)

This crate is guaranteed to compile on the latest stable Rust release. It
*might* compile with older versions but that may change in any new patch
release.

## Development Version of `embedded-hal`

If you would like to use the current development version of `embedded-hal` (or any other version),
so long as they are API compatible you can use a patch field in your `Cargo.toml` file to override
the dependency version.

```yaml
[patch.crates-io]
eh1 = { git = "https://github.com/rust-embedded/embedded-hal" }
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT) at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

<!-- Badges -->
[github-actions]: https://github.com/dbrgn/embedded-hal-mock/actions/workflows/ci.yml
[github-actions-badge]: https://github.com/dbrgn/embedded-hal-mock/actions/workflows/ci.yml/badge.svg
[crates-io]: https://crates.io/crates/embedded-hal-mock
[version-badge]: https://img.shields.io/crates/v/embedded-hal-mock.svg
