# embedded-hal-mock

This is a collection of types that implement the embedded-hal traits.

The implementations never access real hardware. Instead, the hardware is mocked
or no-op implementations are used.

The goal of the crate is to be able to test drivers in CI without having access
to hardware.

## Status

- [x] Simple I2C implementation
- [x] No-op Delay implementation

## no\_std

Currently this crate is not `no_std`. If you think this is important, let me
know.

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
