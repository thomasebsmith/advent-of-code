use std::error::Error;
use std::io;

pub fn invalid_input<E: Into<Box<dyn Error + Send + Sync>>>(
    error: E,
) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, error)
}
