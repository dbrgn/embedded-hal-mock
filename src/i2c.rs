use std::io::Read;

use hal::blocking::i2c;

use error::MockError;

const WRITE_BUF_SIZE: usize = 64;

pub struct I2cMock<'a> {
    data: &'a [u8],
    address: Option<u8>,
    buf: [u8; WRITE_BUF_SIZE],
    buf_bytes_written: usize,
}

impl<'a> I2cMock<'a> {
    pub fn new() -> Self {
        I2cMock {
            data: &[],
            address: None,
            buf: [0; WRITE_BUF_SIZE],
            buf_bytes_written: 0,
        }
    }

    /// Set data that will be read by `read()`.
    pub fn set_read_data(&mut self, data: &'a [u8]) {
        self.data = data;
    }

    /// Return the data that was written by the last write command.
    pub fn get_write_data(&self) -> &[u8] {
        &self.buf[0..self.buf_bytes_written]
    }

    /// Return the address that was used by the last read or write command.
    pub fn get_last_address(&self) -> Option<u8> {
        self.address
    }
}

impl<'a> i2c::Read for I2cMock<'a> {
    type Error = MockError;

    fn read(&mut self, address: u8, mut buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.address = Some(address);
        self.data.read(&mut buffer)?;
        Ok(())
    }
}

impl<'a> i2c::Write for I2cMock<'a> {
    type Error = MockError;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        if bytes.len() > WRITE_BUF_SIZE {
            panic!(
                "Write buffer is too small for this number of bytes ({} > {})",
                bytes.len(),
                WRITE_BUF_SIZE
            );
        }
        self.address = Some(address);
        self.buf[0..bytes.len()].copy_from_slice(bytes);
        self.buf_bytes_written = bytes.len();
        Ok(())
    }
}

impl<'a> i2c::WriteRead for I2cMock<'a> {
    type Error = MockError;

    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        mut buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        if bytes.len() > WRITE_BUF_SIZE {
            panic!(
                "Write buffer is too small for this number of bytes ({} > {})",
                bytes.len(),
                WRITE_BUF_SIZE
            );
        }
        self.address = Some(address);
        self.data.read(&mut buffer)?;
        self.buf[0..bytes.len()].copy_from_slice(bytes);
        self.buf_bytes_written = bytes.len();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use hal::blocking::i2c::{Read, Write, WriteRead};

    #[test]
    fn i2c_read_no_data_set() {
        let mut i2c = I2cMock::new();
        let mut buf = [0; 3];
        i2c.read(0, &mut buf).unwrap();
        assert_eq!(buf, [0; 3]);
    }

    #[test]
    fn i2c_read_some_data_set() {
        let mut i2c = I2cMock::new();
        let mut buf = [0; 3];
        i2c.set_read_data(&[1, 2]);
        i2c.read(0, &mut buf).unwrap();
        assert_eq!(buf, [1, 2, 0]);
    }

    #[test]
    fn i2c_read_too_much_data_set() {
        let mut i2c = I2cMock::new();
        let mut buf = [0; 3];
        i2c.set_read_data(&[1, 2, 3, 4]);
        i2c.read(0, &mut buf).unwrap();
        assert_eq!(buf, [1, 2, 3]);
    }

    #[test]
    fn i2c_write_data() {
        let mut i2c = I2cMock::new();
        let buf = [1, 2, 4];
        assert_eq!(i2c.get_last_address(), None);
        i2c.write(42, &buf[..]).unwrap();
        assert_eq!(i2c.get_last_address(), Some(42));
        assert_eq!(i2c.get_write_data(), &[1, 2, 4]);
        i2c.write(23, &buf[1..2]).unwrap();
        assert_eq!(i2c.get_last_address(), Some(23));
        assert_eq!(i2c.get_write_data(), &[2]);
    }

    #[test]
    #[should_panic]
    fn i2c_write_data_too_much() {
        let mut i2c = I2cMock::new();
        let buf = [0; 65];
        i2c.write(42, &buf[..]).unwrap();
    }

    #[test]
    fn i2c_read_write_data() {
        let mut i2c = I2cMock::new();
        let buf = [1, 2, 4];
        let mut buf2 = [0; 3];
        assert_eq!(i2c.get_last_address(), None);
        i2c.write_read(42, &buf[..], &mut buf2).unwrap();
        assert_eq!(i2c.get_last_address(), Some(42));
        assert_eq!(i2c.get_write_data(), &[1, 2, 4]);
    }
}
