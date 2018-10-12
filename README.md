# embedded-hal-mock

[![CircleCI][circle-ci-badge]][circle-ci]

This is a collection of types that implement the embedded-hal traits.

The implementations never access real hardware. Instead, the hardware is mocked
or no-op implementations are used.

The goal of the crate is to be able to test drivers in CI without having access
to hardware.

## Status

- [x] Simple I2C implementation
- [x] Transactional SPI implementation
- [x] No-op Delay implementation

Pull requests for more mock implementations are welcome! :)

## no\_std

Currently this crate is not `no_std`. If you think this is important, let me
know.

## Usage

### I2C

```rust
use embedded_hal::blocking::i2c::Read;
use embedded_hal_mock::I2cMock;

let mut i2c = I2cMock::new();

// Reading
let mut buf = [0; 3];
i2c.set_read_data(&[1, 2]);
i2c.read(0, &mut buf).unwrap();
assert_eq!(buf, [1, 2, 0]);
assert_eq!(i2c.get_last_address(), Some(0));

// Writing
let buf = [1, 2, 4];
i2c.write(42, &buf).unwrap();
assert_eq!(i2c.get_last_address(), Some(42));
assert_eq!(i2c.get_write_data(), &[1, 2, 4]);
```

### SPI

```rust
use hal::blocking::spi::{Transfer, Write};
use embedded_hal_mock::{SpiMock, SpiTransaction};

let mut spi = SpiMock::new();

// Configure expectations
spi.expect(vec![
    SpiTransaction::write(vec![1u8, 2u8]),
    SpiTransaction::transfer(vec![3u8, 4u8], vec![5u8, 6u8]),
]);

// Writing
spi.write(&vec![1u8, 2u8]).unwrap();

// Transferring
let mut buf = vec![3u8, 4u8];
spi.transfer(&mut buf).unwrap();
assert_eq!(buf, vec![5u8, 6u8]);

// Finalise expectations
spi.done();
```

### Delay

Just create an instance of `embedded_hal_mock::DelayMockNoop`. There will be no
actual delay. This is useful for fast tests, where you don't actually need to
wait for the hardware.

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
