use strum_macros::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Display, Debug)]
pub enum Error {
    Io(std::io::Error),
    Invalid,
    Empty,
    FileNotFound,
    NotSupported,
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(_err: std::str::Utf8Error) -> Self {
        Error::Invalid
    }
}
