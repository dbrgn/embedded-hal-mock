
use std::io;

#[derive(Debug)]
pub enum MockError {
    Io(io::Error),
}

impl From<io::Error> for MockError {
    fn from(e: io::Error) -> Self {
        MockError::Io(e)
    }
}
