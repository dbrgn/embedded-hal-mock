# embedded-hal-mock

[![CircleCI][circle-ci-badge]][circle-ci]
![Minimum Rust Version][min-rust-badge]
[![Crates.io Version][version-badge]][crates-io]

This is a collection of types that implement the embedded-hal traits.

The implementations never access real hardware. Instead, the hardware is mocked
or no-op implementations are used.

The goal of the crate is to be able to test drivers in CI without having access
to hardware.

This crate requires Rust 1.31+!

[Docs](https://docs.rs/embedded-hal-mock/)


## Status

- [x] I²C
- [x] SPI
- [x] No-op delay
- [x] Actual delay
- [x] Serial
- [ ] RNG
- [x] I/O pins
- [x] ADC
- [ ] Timers
- [ ] ...

Pull requests for more mock implementations are welcome! :)


## no\_std

Currently this crate is not `no_std`. If you think this is important, let me
know.


## Usage

See [docs](https://docs.rs/embedded-hal-mock/).


## Development Version of `embedded-hal`

If you would like to use the current development version of `embedded-hal` (or any other version), 
so long as they are API compatible you can use a patch field in your `Cargo.toml` file to override 
the dependency version.

```yaml
[patch.crates-io]
embedded-hal = { git = "https://github.com/rust-embedded/embedded-hal" }
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
[circle-ci]: https://circleci.com/gh/dbrgn/embedded-hal-mock/tree/master
[circle-ci-badge]: https://circleci.com/gh/dbrgn/embedded-hal-mock/tree/master.svg?style=shield
[min-rust-badge]: https://img.shields.io/badge/rustc-1.31+-blue.svg
[crates-io]: https://crates.io/crates/embedded-hal-mock
[version-badge]: https://img.shields.io/crates/v/embedded-hal-mock.svg
