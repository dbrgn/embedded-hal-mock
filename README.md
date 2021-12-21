# embedded-hal-mock

[![GitHub Actions][github-actions-badge]][github-actions]
![Minimum Rust Version][min-rust-badge]
[![Crates.io Version][version-badge]][crates-io]

_(Note: This create currently targets the latest stable version of embedded-hal.
If you're looking for a version that's compatible with the 1.0.0 alpha of
embedded-hal, check out the [`1-alpha`
branch](https://github.com/dbrgn/embedded-hal-mock/tree/1-alpha).)_

This is a collection of types that implement the embedded-hal traits.

The implementations never access real hardware. Instead, the hardware is mocked
or no-op implementations are used.

The goal of the crate is to be able to test drivers in CI without having access
to hardware.

[Docs](https://docs.rs/embedded-hal-mock/)


## Status

- [x] I²C
- [x] SPI
- [x] No-op delay
- [x] Actual delay
- [x] Serial
- [ ] RNG
- [x] I/O pins (including PWM)
- [x] ADC
- [x] Timers (with `embedded-time` Cargo feature)
- [ ] ...

Pull requests for more mock implementations are welcome! :)


## no\_std

Currently this crate is not `no_std`. If you think this is important, let me
know.


## Usage

See [docs](https://docs.rs/embedded-hal-mock/).


## Minimum Supported Rust Version (MSRV)

This crate is guaranteed to compile on stable Rust 1.60 and up. It *might*
compile with older versions but that may change in any new patch release.


## Development Version of `embedded-hal`

If you would like to use the current development version of `embedded-hal` (or any other version),
so long as they are API compatible you can use a patch field in your `Cargo.toml` file to override
the dependency version.

```yaml
[patch.crates-io]
embedded-hal = { git = "https://github.com/rust-embedded/embedded-hal" }
```


# Minimum Supported Rust Version (MSRV)

This crate is guaranteed to compile on stable Rust 1.46.0 and up. It *might*
compile with older versions but that may change in any new patch release.


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
[min-rust-badge]: https://img.shields.io/badge/rustc-1.31+-blue.svg
[crates-io]: https://crates.io/crates/embedded-hal-mock
[version-badge]: https://img.shields.io/crates/v/embedded-hal-mock.svg
