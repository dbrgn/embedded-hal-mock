use std::io;

/// Errors that may occur during mocking.
#[derive(PartialEq, Clone, Debug)]
pub enum MockError {
    /// An I/O-Error occurred
    Io(io::ErrorKind),
}

impl From<io::Error> for MockError {
    fn from(e: io::Error) -> Self {
        MockError::Io(e.kind())
    }
}
